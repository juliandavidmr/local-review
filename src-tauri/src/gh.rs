use std::process::Command;

use crate::domain::GhCliStatus;

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

fn stderr_or_default(stderr: &[u8], fallback: &str) -> String {
    let message = String::from_utf8_lossy(stderr).trim().to_string();
    if message.is_empty() {
        fallback.to_string()
    } else {
        message
    }
}
