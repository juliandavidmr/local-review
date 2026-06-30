use crate::domain::{now_iso, ChangedFile, ModelProviderSettings, ReviewProfileItem};

use super::{
    agent::{parse_json_from_model, repair_model_json, run_rig_agent},
    feedback_mapping::feedback_from_agent_item,
    feedback_quality::agent_item_quality_issue,
    models::openai_base_url,
    prompt::review_prompt,
    types::ReviewPassResult,
};

pub(crate) async fn run_review_pass(
    provider: &ModelProviderSettings,
    profile: &ReviewProfileItem,
    change_set: &crate::domain::ChangeSetSnapshot,
    file: &ChangedFile,
    pass_index: usize,
    repository_tools_enabled: bool,
) -> Result<ReviewPassResult, String> {
    let model = provider
        .selected_model_id
        .clone()
        .ok_or_else(|| "No model selected.".to_string())?;
    let base_url = openai_base_url(provider);
    let prompt = review_prompt(profile, change_set, file, repository_tools_enabled);
    let agent_result = run_rig_agent(
        &base_url,
        &model,
        &prompt,
        &change_set.repository_path,
        repository_tools_enabled,
    )
    .await?;
    let parsed = match parse_json_from_model(&agent_result.raw) {
        Ok(parsed) => parsed,
        Err(parse_error) => {
            eprintln!(
                "[local-review-pass] json_repair_start file={} profile={} error={}",
                file.path, profile.name, parse_error
            );
            let repaired = repair_model_json(&base_url, &model, &agent_result.raw).await?;
            parse_json_from_model(&repaired).map_err(|repair_error| {
                format!(
                    "{parse_error}; repair also failed: {repair_error}; repaired raw: {repaired}"
                )
            })?
        }
    };
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
                    agent_result.exploration_requests > 0,
                ))
            }
        })
        .collect();

    Ok(ReviewPassResult {
        feedback,
        exploration_requests: agent_result.exploration_requests,
    })
}
