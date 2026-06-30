use std::path::Path;

use crate::domain::RepositoryDescriptor;

use super::commands::run_git;

pub fn open_repository(repository_path: &str) -> Result<RepositoryDescriptor, String> {
    let path = Path::new(repository_path);
    if !path.exists() {
        return Err("Repository folder does not exist.".to_string());
    }

    run_git(repository_path, &["rev-parse", "--is-inside-work-tree"])?;
    let branch = run_git(repository_path, &["branch", "--show-current"]).ok();
    let head = run_git(repository_path, &["rev-parse", "HEAD"]).ok();
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("repository")
        .to_string();

    Ok(RepositoryDescriptor {
        path: repository_path.to_string(),
        name,
        current_branch: branch.filter(|value| !value.trim().is_empty()),
        head_sha: head.filter(|value| !value.trim().is_empty()),
    })
}
