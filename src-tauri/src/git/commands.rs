use std::process::Command;

use crate::domain::ChangeSource;

pub fn source_repository_path(source: &ChangeSource) -> &str {
    match source {
        ChangeSource::WorkingTree { repository_path } => repository_path,
        ChangeSource::CurrentBranch { repository_path } => repository_path,
        ChangeSource::StagedChanges { repository_path } => repository_path,
        ChangeSource::UnstagedChanges { repository_path } => repository_path,
        ChangeSource::Commit {
            repository_path, ..
        } => repository_path,
        ChangeSource::CompareRefs {
            repository_path, ..
        } => repository_path,
    }
}

pub fn diff_args(repository_path: &str, source: &ChangeSource) -> Result<Vec<String>, String> {
    match source {
        ChangeSource::WorkingTree { .. } => Ok(vec!["diff".to_string(), "HEAD".to_string()]),
        ChangeSource::CurrentBranch { .. } => Ok(vec![
            "diff".to_string(),
            format!("{}...HEAD", current_branch_base_ref(repository_path)?),
        ]),
        ChangeSource::StagedChanges { .. } => Ok(vec!["diff".to_string(), "--cached".to_string()]),
        ChangeSource::UnstagedChanges { .. } => Ok(vec!["diff".to_string()]),
        ChangeSource::Commit { commit_sha, .. } => Ok(vec![
            "diff".to_string(),
            format!("{commit_sha}^"),
            commit_sha.to_string(),
        ]),
        ChangeSource::CompareRefs {
            base_ref, head_ref, ..
        } => Ok(vec![
            "diff".to_string(),
            base_ref.to_string(),
            head_ref.to_string(),
        ]),
    }
}

pub fn run_git(repository_path: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repository_path)
        .output()
        .map_err(|error| format!("Could not run git: {error}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn run_git_with_extra(
    repository_path: &str,
    base_args: &[String],
    extra_args: &[&str],
) -> Result<String, String> {
    let mut args: Vec<String> = base_args.to_vec();
    args.extend(extra_args.iter().map(|value| value.to_string()));

    let output = Command::new("git")
        .args(args)
        .current_dir(repository_path)
        .output()
        .map_err(|error| format!("Could not run git diff: {error}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn current_branch_base_ref(repository_path: &str) -> Result<String, String> {
    for candidate in ["origin/main", "origin/master", "main", "master"] {
        if run_git(repository_path, &["rev-parse", "--verify", candidate]).is_ok() {
            return Ok(candidate.to_string());
        }
    }

    let current_branch = run_git(repository_path, &["branch", "--show-current"]).ok();
    if let Ok(upstream) = run_git(
        repository_path,
        &[
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
    ) {
        let upstream = upstream.trim();
        let tracks_same_branch = current_branch
            .as_deref()
            .map(|branch| upstream.ends_with(&format!("/{branch}")) || upstream == branch)
            .unwrap_or(false);

        if !upstream.is_empty() && !tracks_same_branch {
            return Ok(upstream.trim().to_string());
        }
    }

    Err("Current branch review needs an upstream branch or a main/master base ref.".to_string())
}
