use std::process::Command;

use crate::domain::{GhCliStatus, ReviewFeedback};
use serde::Deserialize;

pub fn check_status() -> GhCliStatus {
    let version = Command::new("gh").arg("--version").output();
    let Ok(version_output) = version else {
        return GhCliStatus {
            installed: false,
            authenticated: false,
            message: "gh CLI was not found on PATH.".to_string(),
        };
    };

    if !version_output.status.success() {
        return GhCliStatus {
            installed: false,
            authenticated: false,
            message: stderr_or_default(&version_output.stderr, "gh CLI could not run."),
        };
    }

    let auth = Command::new("gh").args(["auth", "status"]).output();
    let Ok(auth_output) = auth else {
        return GhCliStatus {
            installed: true,
            authenticated: false,
            message: "gh CLI is installed, but auth status could not be checked.".to_string(),
        };
    };

    if auth_output.status.success() {
        GhCliStatus {
            installed: true,
            authenticated: true,
            message: "gh CLI is installed and authenticated.".to_string(),
        }
    } else {
        GhCliStatus {
            installed: true,
            authenticated: false,
            message: stderr_or_default(
                &auth_output.stderr,
                "gh CLI is installed, but no authenticated GitHub account was found.",
            ),
        }
    }
}

pub fn publish_review_feedback(
    repository_path: &str,
    feedback: &ReviewFeedback,
) -> Result<(), String> {
    let location = feedback
        .code_location
        .as_ref()
        .ok_or_else(|| "Only inline feedback with a code location can be published.".to_string())?;
    let line = feedback.line.unwrap_or(location.start_line);
    let path = if location.file_path.trim().is_empty() {
        feedback.file.as_str()
    } else {
        location.file_path.as_str()
    };

    let pr = current_pull_request(repository_path)?;
    let repo = current_repository(repository_path)?;
    let body = if feedback.editable_comment.trim().is_empty() {
        feedback.body.trim()
    } else {
        feedback.editable_comment.trim()
    };

    if body.is_empty() {
        return Err("Feedback comment is empty.".to_string());
    }

    let output = Command::new("gh")
        .current_dir(repository_path)
        .args([
            "api",
            &format!("repos/{repo}/pulls/{}/comments", pr.number),
            "-f",
            &format!("body={body}"),
            "-f",
            &format!("commit_id={}", pr.head_ref_oid),
            "-f",
            &format!("path={path}"),
            "-F",
            &format!("line={line}"),
            "-f",
            "side=RIGHT",
        ])
        .output()
        .map_err(|error| format!("Could not run gh api: {error}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(stderr_or_default(
            &output.stderr,
            "gh could not publish the pull request comment.",
        ))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurrentPullRequest {
    number: u32,
    head_ref_oid: String,
}

fn current_pull_request(repository_path: &str) -> Result<CurrentPullRequest, String> {
    let output = Command::new("gh")
        .current_dir(repository_path)
        .args(["pr", "view", "--json", "number,headRefOid"])
        .output()
        .map_err(|error| format!("Could not run gh pr view: {error}"))?;

    if !output.status.success() {
        return Err(stderr_or_default(
            &output.stderr,
            "Could not find a pull request for the current branch.",
        ));
    }

    serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())
}

fn current_repository(repository_path: &str) -> Result<String, String> {
    let output = Command::new("gh")
        .current_dir(repository_path)
        .args(["repo", "view", "--json", "nameWithOwner", "-q", ".nameWithOwner"])
        .output()
        .map_err(|error| format!("Could not run gh repo view: {error}"))?;

    if !output.status.success() {
        return Err(stderr_or_default(
            &output.stderr,
            "Could not resolve the current GitHub repository.",
        ));
    }

    let repo = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if repo.is_empty() {
        Err("Could not resolve the current GitHub repository.".to_string())
    } else {
        Ok(repo)
    }
}

fn stderr_or_default(stderr: &[u8], fallback: &str) -> String {
    let message = String::from_utf8_lossy(stderr).trim().to_string();
    if message.is_empty() {
        fallback.to_string()
    } else {
        message
    }
}
