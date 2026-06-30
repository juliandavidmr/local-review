use std::{fs, path::Path};

use serde::Deserialize;
use serde_json::json;

use super::{
    safety::{canonical_repository_root, should_skip_repository_path},
    ReviewToolError,
};

#[derive(Clone)]
pub(in crate::providers) struct SearchRepositoryTool {
    repository_path: String,
}

impl SearchRepositoryTool {
    pub fn new(repository_path: impl Into<String>) -> Self {
        Self {
            repository_path: repository_path.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRepositoryArgs {
    query: String,
    file_glob: Option<String>,
    max_results: Option<usize>,
}

impl rig::tool::Tool for SearchRepositoryTool {
    const NAME: &'static str = "search_repository";
    type Error = ReviewToolError;
    type Args = SearchRepositoryArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Search non-sensitive repository files for a literal string. Use this to find callers, definitions, tests, configuration, and repeated patterns before writing review feedback.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Literal text to search for. Keep it specific."
                    },
                    "fileGlob": {
                        "type": "string",
                        "description": "Optional simple glob such as *.ts, **/*.rs, src/**, or an exact path."
                    },
                    "maxResults": {
                        "type": "integer",
                        "description": "Maximum matches to return, capped by the tool."
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let query = args.query.trim();
        if query.len() < 2 {
            return Err(ReviewToolError::Rejected(
                "Search query must contain at least two non-space characters.".to_string(),
            ));
        }

        let root = canonical_repository_root(&self.repository_path)?;
        let max_results = args.max_results.unwrap_or(24).clamp(1, 50);
        let mut matches = Vec::new();
        search_repository_files(
            &root,
            &root,
            query,
            args.file_glob.as_deref(),
            max_results,
            &mut matches,
        );

        if matches.is_empty() {
            Ok(format!("No matches for {query:?}."))
        } else {
            Ok(matches.join("\n"))
        }
    }
}

fn search_repository_files(
    root: &Path,
    directory: &Path,
    query: &str,
    file_glob: Option<&str>,
    max_results: usize,
    matches: &mut Vec<String>,
) {
    if matches.len() >= max_results {
        return;
    }

    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    let query_lower = query.to_lowercase();

    for entry in entries.flatten() {
        if matches.len() >= max_results {
            return;
        }
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .ok()
            .and_then(|path| path.to_str())
            .unwrap_or("");
        if relative.is_empty() || should_skip_repository_path(relative, path.is_dir()) {
            continue;
        }

        if path.is_dir() {
            search_repository_files(root, &path, query, file_glob, max_results, matches);
            continue;
        }

        if file_glob
            .map(|glob| !path_matches_simple_glob(relative, glob))
            .unwrap_or(false)
        {
            continue;
        }

        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if metadata.len() > 512 * 1024 {
            continue;
        }

        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        for (index, line) in raw.lines().enumerate() {
            if matches.len() >= max_results {
                return;
            }
            if line.to_lowercase().contains(&query_lower) {
                matches.push(format!("{}:{}: {}", relative, index + 1, line.trim()));
            }
        }
    }
}

fn path_matches_simple_glob(path: &str, glob: &str) -> bool {
    if glob.trim().is_empty() || glob == "*" {
        return true;
    }
    if glob.ends_with("/**") {
        return path.starts_with(glob.trim_end_matches("/**"));
    }
    if glob.starts_with("**/*.") {
        return path.ends_with(glob.trim_start_matches("**/*"));
    }
    if glob.starts_with("*.") {
        return path.ends_with(glob.trim_start_matches('*'));
    }
    path == glob
}
