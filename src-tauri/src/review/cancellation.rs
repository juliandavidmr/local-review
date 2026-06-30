use std::{
    collections::HashSet,
    sync::{Mutex, OnceLock},
};

static CANCELLED_REVIEWS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

pub(super) fn cancel_review_session(review_id: String) -> Result<(), String> {
    cancelled_reviews()
        .lock()
        .map_err(|_| "Could not lock cancellation registry.".to_string())?
        .insert(review_id);
    Ok(())
}

pub(super) fn review_cancelled(review_id: &str) -> bool {
    cancelled_reviews()
        .lock()
        .map(|reviews| reviews.contains(review_id))
        .unwrap_or(false)
}

pub(super) fn clear_review_cancellation(review_id: &str) {
    if let Ok(mut reviews) = cancelled_reviews().lock() {
        reviews.remove(review_id);
    }
}

fn cancelled_reviews() -> &'static Mutex<HashSet<String>> {
    CANCELLED_REVIEWS.get_or_init(|| Mutex::new(HashSet::new()))
}
