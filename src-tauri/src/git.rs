use std::{collections::HashMap, path::Path, process::Command};

use crate::domain::{
    now_iso, ChangeHunk, ChangeLine, ChangeLineKind, ChangeSetSnapshot, ChangeSource, ChangedFile,
    ChangedFileStatus, RepositoryDescriptor,
};

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

pub fn build_change_set(source: ChangeSource) -> Result<ChangeSetSnapshot, String> {
    let repository_path = source_repository_path(&source).to_string();
    let diff_args = diff_args(&source);
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

fn source_repository_path(source: &ChangeSource) -> &str {
    match source {
        ChangeSource::WorkingTree { repository_path } => repository_path,
        ChangeSource::Commit {
            repository_path, ..
        } => repository_path,
        ChangeSource::CompareRefs {
            repository_path, ..
        } => repository_path,
    }
}

fn diff_args(source: &ChangeSource) -> Vec<String> {
    match source {
        ChangeSource::WorkingTree { .. } => vec!["diff".to_string(), "HEAD".to_string()],
        ChangeSource::Commit { commit_sha, .. } => vec![
            "diff".to_string(),
            format!("{commit_sha}^"),
            commit_sha.to_string(),
        ],
        ChangeSource::CompareRefs {
            base_ref, head_ref, ..
        } => vec![
            "diff".to_string(),
            base_ref.to_string(),
            head_ref.to_string(),
        ],
    }
}

fn run_git(repository_path: &str, args: &[&str]) -> Result<String, String> {
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

fn run_git_with_extra(
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

fn parse_numstat(output: &str) -> HashMap<String, (u32, u32)> {
    output
        .lines()
        .filter_map(|line| {
            let mut parts = line.split('\t');
            let additions = parts.next()?.parse::<u32>().ok().unwrap_or(0);
            let deletions = parts.next()?.parse::<u32>().ok().unwrap_or(0);
            let path = parts.next()?.to_string();
            Some((path, (additions, deletions)))
        })
        .collect()
}

fn parse_statuses(output: &str) -> HashMap<String, (ChangedFileStatus, Option<String>)> {
    let mut statuses = HashMap::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let status = status_from_code(parts[0]);
        if parts[0].starts_with('R') && parts.len() >= 3 {
            statuses.insert(parts[2].to_string(), (status, Some(parts[1].to_string())));
        } else {
            statuses.insert(parts[1].to_string(), (status, None));
        }
    }

    statuses
}

fn parse_name_status_only(output: &str, stats: &HashMap<String, (u32, u32)>) -> Vec<ChangedFile> {
    parse_statuses(output)
        .into_iter()
        .map(|(path, (status, previous_path))| {
            let (additions, deletions) = stats.get(&path).copied().unwrap_or((0, 0));
            ChangedFile {
                path: path.clone(),
                previous_path,
                status,
                additions,
                deletions,
                hunks: vec![],
                is_generated: is_generated_file(&path),
            }
        })
        .collect()
}

fn parse_patch(output: &str, stats: &HashMap<String, (u32, u32)>) -> Vec<ChangedFile> {
    let mut files = Vec::new();
    let mut current_file: Option<ChangedFile> = None;
    let mut current_hunk: Option<ChangeHunk> = None;
    let mut old_line = 0;
    let mut new_line = 0;

    for line in output.lines() {
        if line.starts_with("diff --git ") {
            flush_hunk(&mut current_file, &mut current_hunk);
            if let Some(file) = current_file.take() {
                files.push(file);
            }

            let path = line.split(" b/").nth(1).unwrap_or("unknown").to_string();
            let (additions, deletions) = stats.get(&path).copied().unwrap_or((0, 0));
            current_file = Some(ChangedFile {
                path: path.clone(),
                previous_path: None,
                status: ChangedFileStatus::Modified,
                additions,
                deletions,
                hunks: vec![],
                is_generated: is_generated_file(&path),
            });
            continue;
        }

        if line.starts_with("new file mode") {
            if let Some(file) = &mut current_file {
                file.status = ChangedFileStatus::Added;
            }
            continue;
        }

        if line.starts_with("deleted file mode") {
            if let Some(file) = &mut current_file {
                file.status = ChangedFileStatus::Deleted;
            }
            continue;
        }

        if line.starts_with("@@") {
            flush_hunk(&mut current_file, &mut current_hunk);
            if let Some((old_start, new_start)) = parse_hunk_header(line) {
                old_line = old_start;
                new_line = new_start;
                current_hunk = Some(ChangeHunk {
                    id: format!("hunk-{old_start}-{new_start}"),
                    old_start_line: old_start,
                    new_start_line: new_start,
                    lines: vec![],
                });
            }
            continue;
        }

        let Some(hunk) = &mut current_hunk else {
            continue;
        };

        if line.starts_with("+++") || line.starts_with("---") || line.starts_with("\\ No newline") {
            continue;
        }

        if let Some(content) = line.strip_prefix('+') {
            hunk.lines.push(ChangeLine {
                kind: ChangeLineKind::Added,
                content: content.to_string(),
                old_line_number: None,
                new_line_number: Some(new_line),
            });
            new_line += 1;
        } else if let Some(content) = line.strip_prefix('-') {
            hunk.lines.push(ChangeLine {
                kind: ChangeLineKind::Removed,
                content: content.to_string(),
                old_line_number: Some(old_line),
                new_line_number: None,
            });
            old_line += 1;
        } else {
            let content = line.strip_prefix(' ').unwrap_or(line).to_string();
            hunk.lines.push(ChangeLine {
                kind: ChangeLineKind::Context,
                content,
                old_line_number: Some(old_line),
                new_line_number: Some(new_line),
            });
            old_line += 1;
            new_line += 1;
        }
    }

    flush_hunk(&mut current_file, &mut current_hunk);
    if let Some(file) = current_file.take() {
        files.push(file);
    }

    files
}

fn flush_hunk(file: &mut Option<ChangedFile>, hunk: &mut Option<ChangeHunk>) {
    if let (Some(file), Some(hunk)) = (file, hunk.take()) {
        file.hunks.push(hunk);
    }
}

fn parse_hunk_header(line: &str) -> Option<(u32, u32)> {
    let header = line.split("@@").nth(1)?.trim();
    let mut parts = header.split_whitespace();
    let old_part = parts.next()?.trim_start_matches('-');
    let new_part = parts.next()?.trim_start_matches('+');
    let old_start = old_part.split(',').next()?.parse::<u32>().ok()?;
    let new_start = new_part.split(',').next()?.parse::<u32>().ok()?;
    Some((old_start, new_start))
}

fn status_from_code(code: &str) -> ChangedFileStatus {
    match code.chars().next().unwrap_or('M') {
        'A' => ChangedFileStatus::Added,
        'D' => ChangedFileStatus::Deleted,
        'R' => ChangedFileStatus::Renamed,
        'C' => ChangedFileStatus::Copied,
        _ => ChangedFileStatus::Modified,
    }
}

fn is_generated_file(path: &str) -> bool {
    path.ends_with(".lock")
        || path.ends_with(".min.js")
        || path.ends_with(".generated.ts")
        || path.ends_with(".generated.tsx")
        || path.contains("/dist/")
}
