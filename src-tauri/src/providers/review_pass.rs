use crate::domain::{now_iso, ChangedFile, ModelProviderSettings, ReviewProfileItem};

use super::{
    agent::{parse_json_from_model, repair_model_json, run_rig_agent},
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
    let prompt = review_prompt(profile, change_set, file, repository_tools_enabled);
    let (mut parsed, mut exploration_requests, mut raw_response) = run_review_agent_and_parse(
        &base_url,
        &model,
        &prompt,
        &change_set.repository_path,
        repository_tools_enabled,
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
            repository_grounding_prompt(profile, change_set, file, &raw_response);
        let (grounded, retry_exploration_requests, retry_raw_response) =
            run_review_agent_and_parse(
                &base_url,
                &model,
                &grounding_prompt,
                &change_set.repository_path,
                true,
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

    let feedback = parsed
        .feedback
        .into_iter()
        .enumerate()
        .filter_map(|(index, item)| {
            if let Some(reason) = agent_item_quality_issue(&item, file) {
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
                    exploration_requests > 0,
                ))
            }
        })
        .collect();

    Ok(ReviewPassResult {
        feedback,
        exploration_requests,
    })
}

async fn run_review_agent_and_parse(
    base_url: &str,
    model: &str,
    prompt: &str,
    repository_path: &str,
    repository_tools_enabled: bool,
    progress: Option<AgentProgressContext>,
) -> Result<(AgentFeedbackOutput, u32, String), String> {
    let agent_result = run_rig_agent(
        base_url,
        model,
        prompt,
        repository_path,
        repository_tools_enabled,
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
