use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileScopeKind {
    Global,
    Repository,
    Folder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewProfileItem {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub scope_kind: ProfileScopeKind,
    pub selected: bool,
    pub enabled_by_default: bool,
    pub criteria: Vec<String>,
    pub file_globs: Vec<String>,
    pub prompt: String,
}
