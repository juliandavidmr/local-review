use std::fs;

use serde::Deserialize;
use serde_json::json;

use super::{safety::safe_repository_file, ReviewToolError};

#[derive(Clone)]
pub(in crate::providers) struct ReadRepositoryFileTool {
    repository_path: String,
}

impl ReadRepositoryFileTool {
    pub fn new(repository_path: impl Into<String>) -> Self {
        Self {
            repository_path: repository_path.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadRepositoryFileArgs {
    path: String,
    start_line: Option<usize>,
    end_line: Option<usize>,
    context_lines: Option<usize>,
}

impl rig::tool::Tool for ReadRepositoryFileTool {
    const NAME: &'static str = "read_repository_file";
    type Error = ReviewToolError;
    type Args = ReadRepositoryFileArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Read a bounded line range from a non-sensitive repository file. Use this for surrounding function, caller, type, or test context before making a review claim.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Repository-relative file path."
                    },
                    "startLine": {
                        "type": "integer",
                        "description": "1-based starting line. Omit to start at the beginning."
                    },
                    "endLine": {
                        "type": "integer",
                        "description": "1-based ending line. Omit to read from startLine through a bounded window."
                    },
                    "contextLines": {
                        "type": "integer",
                        "description": "Extra lines to include before and after the requested range, capped by the tool."
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let file_path = safe_repository_file(&self.repository_path, &args.path)?;
        let raw = fs::read_to_string(&file_path).map_err(|_| ReviewToolError::ReadFailed)?;
        let lines = raw.lines().collect::<Vec<_>>();
        if lines.is_empty() {
            return Ok(format!("{} is empty.", args.path));
        }

        let requested_start = args.start_line.unwrap_or(1).max(1);
        let requested_end = args
            .end_line
            .unwrap_or_else(|| requested_start.saturating_add(80))
            .max(requested_start);
        let context = args.context_lines.unwrap_or(8).min(30);
        let start = requested_start.saturating_sub(context).max(1);
        let end = requested_end.saturating_add(context).min(lines.len());
        let max_lines = 220usize;
        let end = end.min(start.saturating_add(max_lines).saturating_sub(1));

        let body = lines[start - 1..end]
            .iter()
            .enumerate()
            .map(|(index, line)| format!("{:>5}: {}", start + index, line))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(format!("{}:{}-{}\n{}", args.path, start, end, body))
    }
}
