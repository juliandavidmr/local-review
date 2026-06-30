use crate::domain::{now_iso, ChangeSetSnapshot, ChangeSource};

use super::{
    commands::{diff_args, run_git, run_git_with_extra, source_repository_path},
    parser::{parse_name_status_only, parse_numstat, parse_patch, parse_statuses},
};

pub fn build_change_set(source: ChangeSource) -> Result<ChangeSetSnapshot, String> {
    let repository_path = source_repository_path(&source).to_string();
    let diff_args = diff_args(&repository_path, &source)?;
    let name_status = run_git_with_extra(&repository_path, &diff_args, &["--name-status"])?;
    let numstat = run_git_with_extra(&repository_path, &diff_args, &["--numstat"])?;
    let patch = run_git_with_extra(&repository_path, &diff_args, &["--unified=3"])?;
    let stats = parse_numstat(&numstat);
    let mut files = parse_patch(&patch, &stats);

    if files.is_empty() {
        files = parse_name_status_only(&name_status, &stats);
    } else {
        let statuses = parse_statuses(&name_status);
        for file in &mut files {
            if let Some((status, previous_path)) = statuses.get(&file.path) {
                file.status = status.clone();
                file.previous_path = previous_path.clone();
            }
        }
    }

    let created_at = now_iso();
    let head = run_git(&repository_path, &["rev-parse", "HEAD"]).unwrap_or_default();
    let fingerprint = format!(
        "{}:{}:{}:{}",
        repository_path,
        head.trim(),
        files.len(),
        files
            .iter()
            .map(|file| file.additions + file.deletions)
            .sum::<u32>()
    );

    Ok(ChangeSetSnapshot {
        id: format!("changeset-{}", chrono::Utc::now().timestamp_millis()),
        repository_path,
        source,
        base_ref: None,
        head_ref: None,
        files,
        created_at,
        fingerprint,
    })
}
