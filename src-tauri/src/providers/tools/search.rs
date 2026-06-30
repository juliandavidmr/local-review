use std::{
    fs,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

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
        if let Some(rg_result) =
            search_with_ripgrep(&root, query, args.file_glob.as_deref(), max_results)
        {
            return render_search_result(query, rg_result);
        }

        let mut matches = Vec::new();
        let mut budget = SearchBudget::new();
        search_repository_files(
            &root,
            &root,
            query,
            args.file_glob.as_deref(),
            max_results,
            &mut matches,
            &mut budget,
        );

        if budget.exhausted {
            matches.push(
                "Search stopped after a bounded scan. Use a narrower query or fileGlob for more context."
                    .to_string(),
            );
        }

        render_search_result(query, matches)
    }
}

fn search_with_ripgrep(
    root: &Path,
    query: &str,
    file_glob: Option<&str>,
    max_results: usize,
) -> Option<Vec<String>> {
    let mut command = Command::new("rg");
    command.current_dir(root).args([
        "--files-with-matches",
        "--fixed-strings",
        "--no-messages",
        "--color",
        "never",
        "--max-filesize",
        "512K",
        "--glob",
        "!.git/**",
        "--glob",
        "!node_modules/**",
        "--glob",
        "!**/node_modules/**",
        "--glob",
        "!target/**",
        "--glob",
        "!**/target/**",
        "--glob",
        "!dist/**",
        "--glob",
        "!**/dist/**",
        "--glob",
        "!build/**",
        "--glob",
        "!**/build/**",
        "--glob",
        "!.next/**",
        "--glob",
        "!**/.next/**",
    ]);

    if let Some(glob) = file_glob.filter(|glob| !glob.trim().is_empty() && *glob != "*") {
        command.args(["--glob", glob]);
    }
    command.arg("--").arg(query);

    let output = command.output().ok()?;
    if !output.status.success() {
        return if output.status.code() == Some(1) {
            Some(Vec::new())
        } else {
            None
        };
    }

    let candidates = String::from_utf8_lossy(&output.stdout);
    let mut matches = Vec::new();
    for relative in candidates.lines().take(max_results.saturating_mul(4)) {
        if matches.len() >= max_results {
            break;
        }
        collect_matching_lines(root, relative, query, max_results, &mut matches);
    }

    Some(matches)
}

fn search_repository_files(
    root: &Path,
    directory: &Path,
    query: &str,
    file_glob: Option<&str>,
    max_results: usize,
    matches: &mut Vec<String>,
    budget: &mut SearchBudget,
) {
    if matches.len() >= max_results || budget.should_stop() {
        return;
    }
    budget.visited_dirs += 1;

    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    let query_lower = query.to_lowercase();

    for entry in entries.flatten() {
        if matches.len() >= max_results || budget.should_stop() {
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
            search_repository_files(root, &path, query, file_glob, max_results, matches, budget);
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
        budget.visited_files += 1;

        collect_matching_lines_in_file(&path, relative, &query_lower, max_results, matches);
    }
}

fn collect_matching_lines(
    root: &Path,
    relative: &str,
    query: &str,
    max_results: usize,
    matches: &mut Vec<String>,
) {
    if should_skip_repository_path(relative, false) {
        return;
    }
    let path = root.join(relative);
    let query_lower = query.to_lowercase();
    collect_matching_lines_in_file(&path, relative, &query_lower, max_results, matches);
}

fn collect_matching_lines_in_file(
    path: &Path,
    relative: &str,
    query_lower: &str,
    max_results: usize,
    matches: &mut Vec<String>,
) {
    let Ok(raw) = fs::read_to_string(path) else {
        return;
    };
    for (index, line) in raw.lines().enumerate() {
        if matches.len() >= max_results {
            return;
        }
        if line.to_lowercase().contains(query_lower) {
            matches.push(format!("{}:{}: {}", relative, index + 1, line.trim()));
        }
    }
}

fn render_search_result(query: &str, matches: Vec<String>) -> Result<String, ReviewToolError> {
    if matches.is_empty() {
        Ok(format!("No matches for {query:?}."))
    } else {
        Ok(matches.join("\n"))
    }
}

struct SearchBudget {
    started_at: Instant,
    visited_files: usize,
    visited_dirs: usize,
    exhausted: bool,
}

impl SearchBudget {
    fn new() -> Self {
        Self {
            started_at: Instant::now(),
            visited_files: 0,
            visited_dirs: 0,
            exhausted: false,
        }
    }

    fn should_stop(&mut self) -> bool {
        let stop = self.started_at.elapsed() > Duration::from_secs(3)
            || self.visited_files > 2_000
            || self.visited_dirs > 500;
        self.exhausted |= stop;
        stop
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
