use crate::error::AppError;
use std::path::{Path, PathBuf};

/// Safely resolves a subpath against a shared root directory.
/// Returns the canonicalized target path if it is safe and resides inside the root directory.
pub fn safe_resolve_path(root_dir: &Path, relative_subpath: &str) -> Result<PathBuf, AppError> {
    // 1. Canonicalize the root directory first to resolve symlinks
    let canonical_root = root_dir
        .canonicalize()
        .map_err(|e| AppError::NotFound(format!("Shared folder root could not be found: {}", e)))?;

    // 2. Decode the subpath (URL decoding) and build the target path
    let decoded_subpath = percent_encoding::percent_decode_str(relative_subpath)
        .decode_utf8()
        .map_err(|_| AppError::BadRequest("Invalid URL encoding in path".to_string()))?;

    // Trim leading slash to make it relative to the root
    let clean_subpath = decoded_subpath.trim_start_matches('/');

    let mut target = canonical_root.clone();
    for component in Path::new(clean_subpath).components() {
        match component {
            std::path::Component::Normal(c) => target.push(c),
            std::path::Component::ParentDir => {
                // Explictly block ".." components to prevent traversal
                return Err(AppError::Forbidden(
                    "Directory traversal sequence disallowed".to_string(),
                ));
            }
            std::path::Component::Prefix(_) | std::path::Component::RootDir => {
                // Ignore prefix or absolute root transitions to keep it relative
            }
            std::path::Component::CurDir => {
                // Ignore "."
            }
        }
    }

    // 3. Canonicalize the target path to resolve any symlinks
    let canonical_target = target
        .canonicalize()
        .map_err(|_| AppError::NotFound("File or directory not found".to_string()))?;

    // 4. Ensure target still starts with the root path
    if !canonical_target.starts_with(&canonical_root) {
        return Err(AppError::Forbidden(
            "Access denied: path traverses outside root".to_string(),
        ));
    }

    Ok(canonical_target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_safe_resolve_path() {
        let temp_dir = std::env::temp_dir().join(format!("qrshare-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.txt");
        File::create(&file_path).unwrap();

        let sub_dir = temp_dir.join("sub");
        std::fs::create_dir_all(&sub_dir).unwrap();
        let nested_file = sub_dir.join("nested.txt");
        File::create(&nested_file).unwrap();

        // 1. Test valid paths
        let resolved = safe_resolve_path(&temp_dir, "test.txt").unwrap();
        assert_eq!(
            resolved.canonicalize().unwrap(),
            file_path.canonicalize().unwrap()
        );

        let resolved_nested = safe_resolve_path(&temp_dir, "sub/nested.txt").unwrap();
        assert_eq!(
            resolved_nested.canonicalize().unwrap(),
            nested_file.canonicalize().unwrap()
        );

        // 2. Test traversal attempts
        assert!(safe_resolve_path(&temp_dir, "../").is_err());
        assert!(safe_resolve_path(&temp_dir, "sub/../../").is_err());

        // 3. Test URL encoding resolution
        let resolved_encoded = safe_resolve_path(&temp_dir, "sub%2Fnested.txt").unwrap();
        assert_eq!(
            resolved_encoded.canonicalize().unwrap(),
            nested_file.canonicalize().unwrap()
        );

        // Clean up
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
