use crate::error::AppError;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Recursively gathers all files within a folder.
fn walk_dir(dir: &Path, current_dir: &Path) -> Result<Vec<(PathBuf, String)>, AppError> {
    let mut files = Vec::new();
    let entries = std::fs::read_dir(current_dir)
        .map_err(|e| AppError::Internal(format!("Failed to read directory for zip: {}", e)))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path
            .strip_prefix(dir)
            .map_err(|_| AppError::Internal("Failed to calculate relative path".to_string()))?
            .to_string_lossy()
            .into_owned();

        if path.is_dir() {
            files.extend(walk_dir(dir, &path)?);
        } else if path.is_file() {
            files.push((path, relative_path));
        }
    }
    Ok(files)
}

/// Generates a zip archive of the directory at a temporary file path.
/// Returns the path to the temporary zip file.
pub fn generate_zip_file(dir_path: &Path) -> Result<PathBuf, AppError> {
    let temp_path = std::env::temp_dir().join(format!("qrshare-{}.zip", uuid::Uuid::new_v4()));
    let file = File::create(&temp_path)
        .map_err(|e| AppError::Internal(format!("Failed to create temporary zip file: {}", e)))?;

    let mut zip = zip::ZipWriter::new(file);

    let files = walk_dir(dir_path, dir_path)?;

    for (file_path, zip_name) in files {
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o644);

        zip.start_file(zip_name, options)
            .map_err(|e| AppError::Internal(format!("Failed to start zip file entry: {}", e)))?;

        let mut f = File::open(file_path)?;
        std::io::copy(&mut f, &mut zip)?;
    }

    zip.finish()
        .map_err(|e| AppError::Internal(format!("Failed to finalize zip file: {}", e)))?;

    Ok(temp_path)
}

/// A wrapper structure that deletes the file when dropped.
struct Cleanup {
    path: PathBuf,
    file: Option<tokio::fs::File>,
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        // Release the file handle first before deletion
        self.file = None;
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Streams a file from disk and deletes it when the stream is completed or dropped.
pub fn stream_zip_file(
    temp_path: PathBuf,
    file: tokio::fs::File,
) -> impl futures_util::Stream<Item = Result<bytes::Bytes, std::io::Error>> {
    let state = Cleanup {
        path: temp_path,
        file: Some(file),
    };

    futures_util::stream::unfold(state, |mut state| async move {
        let mut buf = vec![0u8; 64 * 1024]; // 64KB chunks
        if let Some(ref mut file) = state.file {
            use tokio::io::AsyncReadExt;
            match file.read(&mut buf).await {
                Ok(0) => None,
                Ok(n) => {
                    let bytes = bytes::Bytes::copy_from_slice(&buf[..n]);
                    Some((Ok(bytes), state))
                }
                Err(e) => Some((Err(e), state)),
            }
        } else {
            None
        }
    })
}
