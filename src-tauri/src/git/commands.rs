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
        ChangeSource::CurrentBranch { .. } => current_branch_diff_args(repository_path),
        ChangeSource::StagedChanges { .. } => Ok(vec!["diff".to_string(), "--cached".to_string()]),
        ChangeSource::UnstagedChanges { .. } => Ok(vec!["diff".to_string()]),
        ChangeSource::Commit { commit_sha, .. } => Ok(vec![
            "diff".to_string(),
            format!("{commit_sha}^"),
            commit_sha.to_string(),
        ]),
        ChangeSource::CompareRefs {
            base_ref, head_ref, ..
        } => Ok(vec!["diff".to_string(), format!("{base_ref}...{head_ref}")]),
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
    let mut args: Vec<String> = Vec::new();
    if let Some(pathspec_index) = base_args.iter().position(|value| value == "--") {
        args.extend_from_slice(&base_args[..pathspec_index]);
        args.extend(extra_args.iter().map(|value| value.to_string()));
        args.extend_from_slice(&base_args[pathspec_index..]);
    } else {
        args.extend_from_slice(base_args);
        args.extend(extra_args.iter().map(|value| value.to_string()));
    }

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

fn current_branch_diff_args(repository_path: &str) -> Result<Vec<String>, String> {
    let base_ref = current_branch_base_ref(repository_path)?;
    let owned_paths = current_branch_owned_paths(repository_path, &base_ref)?;

    if owned_paths.is_empty() {
        return Ok(vec![
            "diff".to_string(),
            "HEAD".to_string(),
            "HEAD".to_string(),
        ]);
    }

    let mut args = vec![
        "diff".to_string(),
        format!("{base_ref}...HEAD"),
        "--".to_string(),
    ];
    args.extend(owned_paths);
    Ok(args)
}

fn current_branch_owned_paths(
    repository_path: &str,
    base_ref: &str,
) -> Result<Vec<String>, String> {
    let revision_range = format!("{base_ref}..HEAD");
    let output = run_git(
        repository_path,
        &[
            "log",
            "--first-parent",
            "--no-merges",
            "--name-only",
            "--format=",
            &revision_range,
        ],
    )?;

    let mut paths = Vec::new();
    for path in output
        .lines()
        .map(str::trim)
        .filter(|path| !path.is_empty())
    {
        if !paths.iter().any(|existing| existing == path) {
            paths.push(path.to_string());
        }
    }

    Ok(paths)
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::{
        fs,
        path::PathBuf,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn current_branch_limits_diff_to_first_parent_non_merge_paths() {
        let repo = TestRepo::new();
        repo.git(&["init", "--initial-branch=main"]);
        repo.git(&["config", "user.email", "test@example.com"]);
        repo.git(&["config", "user.name", "Test User"]);
        repo.write("owned.txt", "base owned\n");
        repo.write("external.txt", "base external\n");
        repo.git(&["add", "."]);
        repo.git(&["commit", "-m", "initial"]);
        repo.git(&["checkout", "-b", "external"]);
        repo.write("external.txt", "base external\nexternal change\n");
        repo.git(&["commit", "-am", "external change"]);
        repo.git(&["checkout", "main"]);
        repo.git(&["checkout", "-b", "feature"]);
        repo.write("owned.txt", "base owned\nowned change\n");
        repo.git(&["commit", "-am", "owned change"]);
        repo.git(&["merge", "--no-ff", "external", "-m", "merge external"]);

        let args = diff_args(
            repo.path_str(),
            &ChangeSource::CurrentBranch {
                repository_path: repo.path_str().to_string(),
            },
        )
        .expect("current branch diff args should resolve");

        assert_eq!(args, vec!["diff", "main...HEAD", "--", "owned.txt"]);

        let name_status = run_git_with_extra(repo.path_str(), &args, &["--name-status"])
            .expect("name-status diff should run with pathspec-limited args");
        assert_eq!(name_status.trim(), "M\towned.txt");
    }

    #[test]
    fn compare_refs_uses_merge_base_diff() {
        let args = diff_args(
            "/repo",
            &ChangeSource::CompareRefs {
                repository_path: "/repo".to_string(),
                base_ref: "origin/main".to_string(),
                head_ref: "feature".to_string(),
            },
        )
        .expect("compare refs diff args should not inspect the repository");

        assert_eq!(args, vec!["diff", "origin/main...feature"]);
    }

    struct TestRepo {
        path: PathBuf,
    }

    impl TestRepo {
        fn new() -> Self {
            let mut path = std::env::temp_dir();
            let id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be valid")
                .as_nanos();
            path.push(format!("local-review-git-test-{id}"));
            fs::create_dir_all(&path).expect("temp repo should be created");
            Self { path }
        }

        fn path_str(&self) -> &str {
            self.path.to_str().expect("temp path should be valid utf-8")
        }

        fn write(&self, path: &str, content: &str) {
            fs::write(self.path.join(path), content).expect("fixture file should be written");
        }

        fn git(&self, args: &[&str]) {
            let output = Command::new("git")
                .args(args)
                .current_dir(&self.path)
                .output()
                .expect("git should run");

            assert!(
                output.status.success(),
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    impl Drop for TestRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
