use super::{
    ExecutionCapacitySettings, LocalModelProviderKind, McpSourceSettings, ModelProviderSettings,
    ProfileScopeKind, ProviderSettings, ReviewProfileItem,
};

pub fn default_provider_settings() -> ProviderSettings {
    ProviderSettings {
        model_providers: vec![
            ModelProviderSettings {
                id: "ollama".to_string(),
                kind: LocalModelProviderKind::Ollama,
                name: "Ollama".to_string(),
                base_url: "http://localhost:11434".to_string(),
                enabled: false,
                selected_model_id: None,
                use_for_human_tone_rewrite: false,
            },
            ModelProviderSettings {
                id: "lm-studio".to_string(),
                kind: LocalModelProviderKind::LmStudio,
                name: "LM Studio".to_string(),
                base_url: "http://localhost:1234/v1".to_string(),
                enabled: true,
                selected_model_id: None,
                use_for_human_tone_rewrite: false,
            },
        ],
        mcp_sources: vec![
            McpSourceSettings {
                id: "filesystem".to_string(),
                name: "Filesystem context".to_string(),
                description: Some("Guarded repository exploration.".to_string()),
                enabled: true,
            },
            McpSourceSettings {
                id: "github".to_string(),
                name: "GitHub context".to_string(),
                description: Some("Future configured MCP and gh context.".to_string()),
                enabled: false,
            },
        ],
        execution: ExecutionCapacitySettings {
            max_parallel_review_passes: 2,
            adaptive_parallelism_enabled: true,
        },
    }
}

pub fn default_profiles() -> Vec<ReviewProfileItem> {
    vec![
        ReviewProfileItem {
            id: "correctness".to_string(),
            name: "Correctness".to_string(),
            scope: "Global default".to_string(),
            scope_kind: ProfileScopeKind::Global,
            selected: true,
            enabled_by_default: true,
            criteria: vec![
                "Correctness".to_string(),
                "Regression risk".to_string(),
                "Edge cases".to_string(),
            ],
            file_globs: vec!["*".to_string()],
            prompt: "Review for concrete behavior regressions only. Name the failing scenario, the invariant or validation that changed, the user-visible or data-impacting consequence, and the smallest code change that would prevent it. Skip comments that cannot cite exact changed lines and supporting context.".to_string(),
        },
        ReviewProfileItem {
            id: "architecture".to_string(),
            name: "Architecture".to_string(),
            scope: "Global default".to_string(),
            scope_kind: ProfileScopeKind::Global,
            selected: true,
            enabled_by_default: true,
            criteria: vec![
                "Hexagonal boundaries".to_string(),
                "Domain purity".to_string(),
                "Adapter isolation".to_string(),
            ],
            file_globs: vec!["*".to_string()],
            prompt: "Review architecture only when the diff crosses a documented boundary or makes a module harder to reason about. Cite the boundary, the dependency direction or domain term being violated, and the future change that becomes riskier. Skip broad design preferences without repository evidence.".to_string(),
        },
    ]
}
