use crate::domain::{
    now_iso, ChangedFile, LocalModelProviderKind, ModelProviderSettings, ReviewProfileItem,
};

use super::{
    agent::{parse_json_from_model, repair_model_json, run_rig_agent},
    context::model_prompt_budget,
    feedback_mapping::feedback_from_agent_item,
    feedback_quality::{agent_item_quality_issue, feedback_requires_repository_exploration},
    models::openai_base_url,
    prompt::{repository_grounding_prompt, review_prompt},
    types::{AgentFeedbackOutput, AgentProgressContext, ReviewPassResult},
};

pub(crate) async fn run_review_pass(
    provider: &ModelProviderSettings,
    profile: &ReviewProfileItem,
    change_set: &crate::domain::ChangeSetSnapshot,
    file: &ChangedFile,
    pass_index: usize,
    repository_tools_enabled: bool,
    progress: AgentProgressContext,
) -> Result<ReviewPassResult, String> {
    let model = provider
        .selected_model_id
        .clone()
        .ok_or_else(|| "No model selected.".to_string())?;
    let base_url = openai_base_url(provider);
    let prompt_budget = model_prompt_budget(provider, &model, repository_tools_enabled).await;
    let structured_output_enabled = structured_output_enabled(provider);
    eprintln!(
        "[local-review-pass] prompt_budget file={} profile={} model={} tools_enabled={} structured_output={} context_tokens={} max_prompt_chars={}",
        file.path,
        profile.name,
        model,
        repository_tools_enabled,
        structured_output_enabled,
        prompt_budget.context_tokens,
        prompt_budget.max_prompt_chars
    );
    let prompt = review_prompt(
        profile,
        change_set,
        file,
        repository_tools_enabled,
        prompt_budget,
    );
    let (mut parsed, mut exploration_requests, mut raw_response) = run_review_agent_and_parse(
        &base_url,
        &model,
        &prompt,
        &change_set.repository_path,
        repository_tools_enabled,
        structured_output_enabled,
        Some(progress.clone()),
    )
    .await?;

    if repository_tools_enabled
        && exploration_requests == 0
        && parsed
            .feedback
            .iter()
            .any(feedback_requires_repository_exploration)
    {
        eprintln!(
            "[local-review-pass] grounding_retry_start file={} profile={} reason=external_claim_without_tool_use",
            file.path, profile.name
        );
        let grounding_prompt =
            repository_grounding_prompt(profile, change_set, file, &raw_response, prompt_budget);
        let (grounded, retry_exploration_requests, retry_raw_response) =
            run_review_agent_and_parse(
                &base_url,
                &model,
                &grounding_prompt,
                &change_set.repository_path,
                true,
                structured_output_enabled,
                Some(AgentProgressContext {
                    current_phase: "Grounding external claims with repository tools".to_string(),
                    ..progress.clone()
                }),
            )
            .await?;
        exploration_requests += retry_exploration_requests;
        parsed = grounded;
        raw_response = retry_raw_response;
        eprintln!(
            "[local-review-pass] grounding_retry_finish file={} profile={} exploration_requests={} response_bytes={}",
            file.path,
            profile.name,
            retry_exploration_requests,
            raw_response.len()
        );
    }

    let pass_id = format!("pass-{}-{}", profile.id, pass_index + 1);
    let created_at = now_iso();

    let used_repository_exploration = exploration_requests > 0;
    let feedback = parsed
        .feedback
        .into_iter()
        .enumerate()
        .filter_map(|(index, item)| {
            if let Some(reason) = agent_item_quality_issue(&item, file, used_repository_exploration)
            {
                eprintln!(
                    "[local-review-pass] feedback_rejected file={} profile={} reason={}",
                    file.path, profile.name, reason
                );
                None
            } else {
                Some(feedback_from_agent_item(
                    item,
                    provider,
                    profile,
                    file,
                    &model,
                    &pass_id,
                    &created_at,
                    index,
                    used_repository_exploration,
                ))
            }
        })
        .collect();

    Ok(ReviewPassResult {
        feedback,
        exploration_requests,
    })
}

fn structured_output_enabled(provider: &ModelProviderSettings) -> bool {
    matches!(&provider.kind, LocalModelProviderKind::LmStudio)
}

async fn run_review_agent_and_parse(
    base_url: &str,
    model: &str,
    prompt: &str,
    repository_path: &str,
    repository_tools_enabled: bool,
    structured_output_enabled: bool,
    progress: Option<AgentProgressContext>,
) -> Result<(AgentFeedbackOutput, u32, String), String> {
    let agent_result = run_rig_agent(
        base_url,
        model,
        prompt,
        repository_path,
        repository_tools_enabled,
        structured_output_enabled,
        progress,
    )
    .await?;
    let parsed = match parse_json_from_model(&agent_result.raw) {
        Ok(parsed) => parsed,
        Err(parse_error) => {
            eprintln!("[local-review-pass] json_repair_start error={parse_error}");
            let repaired = repair_model_json(base_url, model, &agent_result.raw).await?;
            parse_json_from_model(&repaired).map_err(|repair_error| {
                format!(
                    "{parse_error}; repair also failed: {repair_error}; repaired raw: {repaired}"
                )
            })?
        }
    };

    Ok((parsed, agent_result.exploration_requests, agent_result.raw))
}

#[cfg(test)]
mod tests {
    use super::structured_output_enabled;
    use crate::domain::{LocalModelProviderKind, ModelProviderSettings};

    fn provider(kind: LocalModelProviderKind) -> ModelProviderSettings {
        ModelProviderSettings {
            id: "provider".to_string(),
            kind,
            name: "Provider".to_string(),
            base_url: "http://localhost".to_string(),
            enabled: true,
            selected_model_id: Some("model".to_string()),
            use_for_human_tone_rewrite: false,
        }
    }

    #[test]
    fn structured_output_is_enabled_for_lm_studio_only() {
        assert!(structured_output_enabled(&provider(
            LocalModelProviderKind::LmStudio
        )));
        assert!(!structured_output_enabled(&provider(
            LocalModelProviderKind::Ollama
        )));
    }
}
