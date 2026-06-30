use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryDescriptor {
    pub path: String,
    pub name: String,
    pub current_branch: Option<String>,
    pub head_sha: Option<String>,
}
