use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChangeSource {
    WorkingTree {
        repository_path: String,
    },
    CurrentBranch {
        repository_path: String,
    },
    StagedChanges {
        repository_path: String,
    },
    UnstagedChanges {
        repository_path: String,
    },
    Commit {
        repository_path: String,
        commit_sha: String,
    },
    CompareRefs {
        repository_path: String,
        base_ref: String,
        head_ref: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangedFileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeLineKind {
    Added,
    Removed,
    Context,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeLine {
    pub kind: ChangeLineKind,
    pub content: String,
    pub old_line_number: Option<u32>,
    pub new_line_number: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeHunk {
    pub id: String,
    pub old_start_line: u32,
    pub new_start_line: u32,
    pub lines: Vec<ChangeLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangedFile {
    pub path: String,
    pub previous_path: Option<String>,
    pub status: ChangedFileStatus,
    pub additions: u32,
    pub deletions: u32,
    pub hunks: Vec<ChangeHunk>,
    pub is_generated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetSnapshot {
    pub id: String,
    pub repository_path: String,
    pub source: ChangeSource,
    pub base_ref: Option<String>,
    pub head_ref: Option<String>,
    pub files: Vec<ChangedFile>,
    pub created_at: String,
    pub fingerprint: String,
}
