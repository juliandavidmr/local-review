use std::{
    fs,
    path::{Path, PathBuf},
};

use super::ReviewToolError;

pub(in crate::providers) fn safe_repository_file(
    repository_path: &str,
    relative_path: &str,
) -> Result<PathBuf, ReviewToolError> {
    if relative_path.trim().is_empty() || Path::new(relative_path).is_absolute() {
        return Err(ReviewToolError::Rejected(
            "Use a non-empty repository-relative path.".to_string(),
        ));
    }
    if is_sensitive_path(relative_path) {
        return Err(ReviewToolError::Rejected(
            "Sensitive files are not available to review tools.".to_string(),
        ));
    }

    let root = canonical_repository_root(repository_path)?;
    let path = root.join(relative_path);
    let canonical = fs::canonicalize(path).map_err(|_| {
        ReviewToolError::Rejected("Requested file is not readable in the repository.".to_string())
    })?;

    if !canonical.starts_with(&root) || !canonical.is_file() {
        return Err(ReviewToolError::Rejected(
            "Requested path is outside the repository or is not a file.".to_string(),
        ));
    }

    Ok(canonical)
}

pub(super) fn canonical_repository_root(repository_path: &str) -> Result<PathBuf, ReviewToolError> {
    fs::canonicalize(repository_path)
        .map_err(|_| ReviewToolError::Rejected("Repository path is not readable.".to_string()))
}

pub(super) fn should_skip_repository_path(relative_path: &str, is_dir: bool) -> bool {
    let normalized = relative_path.replace('\\', "/");
    let first = normalized.split('/').next().unwrap_or("");
    if is_sensitive_path(&normalized) {
        return true;
    }
    if is_dir {
        matches!(
            first,
            ".git" | "node_modules" | "target" | "dist" | "build" | ".next" | ".turbo" | ".cache"
        )
    } else {
        false
    }
}

pub(in crate::providers) fn is_sensitive_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/").to_lowercase();
    let file_name = normalized.rsplit('/').next().unwrap_or(normalized.as_str());
    file_name == ".env"
        || file_name.starts_with(".env.")
        || file_name.ends_with(".pem")
        || file_name.ends_with(".key")
        || file_name.ends_with(".p12")
        || file_name.ends_with(".pfx")
        || file_name.ends_with(".crt")
        || file_name.ends_with(".cer")
        || file_name.ends_with(".dump")
        || file_name.ends_with(".sql")
        || file_name.contains("id_rsa")
        || file_name.contains("secret")
        || file_name.contains("credential")
}
