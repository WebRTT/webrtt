use std::time::{Duration, Instant};
use uuid::Uuid;

#[allow(dead_code)]
pub const MIN_INTERVAL: Duration = Duration::from_millis(200);
#[allow(dead_code)]
pub const MIN_CONFIDENCE: f32 = 0.6;
#[allow(dead_code)]
pub const HIGH_CONFIDENCE: f32 = 0.9;
#[allow(dead_code)]
pub const REVERT_THRESHOLD: f32 = 0.2;

#[allow(dead_code)]
pub struct SpeculationEngine {
    last_hypothesis_at: Option<Instant>,
    last_hypothesis_tokens: Vec<String>,
    last_checkpoint_id: Option<Uuid>,
    hypothesis_count: u64,
    revert_count: u64,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum HypothesisDecision {
    Emit(Uuid),
    Skip,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum CommitDecision {
    CommitExisting(Uuid),
    CommitFresh(Uuid),
    RevertThenCommit {
        revert_checkpoint: Uuid,
        new_checkpoint: Uuid,
    },
}

#[allow(dead_code)]
impl SpeculationEngine {
    pub fn new() -> Self {
        Self {
            last_hypothesis_at: None,
            last_hypothesis_tokens: vec![],
            last_checkpoint_id: None,
            hypothesis_count: 0,
            revert_count: 0,
        }
    }

    pub fn on_partial_tokens(&mut self, tokens: &[String], confidence: f32) -> HypothesisDecision {
        if confidence < MIN_CONFIDENCE {
            return HypothesisDecision::Skip;
        }

        let now = Instant::now();
        let elapsed = self
            .last_hypothesis_at
            .map(|t| now.duration_since(t))
            .unwrap_or(Duration::MAX);

        let should_emit = confidence >= HIGH_CONFIDENCE || elapsed >= MIN_INTERVAL;

        if should_emit {
            let checkpoint_id = Uuid::new_v4();
            self.last_hypothesis_at = Some(now);
            self.last_hypothesis_tokens = tokens.to_vec();
            self.last_checkpoint_id = Some(checkpoint_id);
            self.hypothesis_count += 1;
            HypothesisDecision::Emit(checkpoint_id)
        } else {
            HypothesisDecision::Skip
        }
    }

    pub fn on_final_tokens(&mut self, final_tokens: &[String]) -> CommitDecision {
        match self.last_checkpoint_id {
            None => CommitDecision::CommitFresh(Uuid::new_v4()),

            Some(checkpoint_id) => {
                let distance = edit_distance_ratio(&self.last_hypothesis_tokens, final_tokens);

                if distance <= REVERT_THRESHOLD {
                    CommitDecision::CommitExisting(checkpoint_id)
                } else {
                    self.revert_count += 1;
                    CommitDecision::RevertThenCommit {
                        revert_checkpoint: checkpoint_id,
                        new_checkpoint: Uuid::new_v4(),
                    }
                }
            }
        }
    }

    pub fn revert_rate(&self) -> f32 {
        if self.hypothesis_count == 0 {
            return 0.0;
        }
        self.revert_count as f32 / self.hypothesis_count as f32
    }
}

#[allow(dead_code)]
pub fn edit_distance_ratio(a: &[String], b: &[String]) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let max_len = a.len().max(b.len()) as f32;
    levenshtein(a, b) as f32 / max_len
}

#[allow(dead_code, clippy::needless_range_loop)]
fn levenshtein(a: &[String], b: &[String]) -> usize {
    let m = a.len();
    let n = b.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for (i, row) in dp.iter_mut().enumerate().take(m + 1) {
        row[0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            dp[i][j] = if a[i - 1] == b[j - 1] {
                dp[i - 1][j - 1]
            } else {
                1 + dp[i - 1][j].min(dp[i][j - 1]).min(dp[i - 1][j - 1])
            };
        }
    }
    dp[m][n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_below_min_confidence() {
        let mut e = SpeculationEngine::new();
        assert!(matches!(
            e.on_partial_tokens(&["hello".into()], 0.5),
            HypothesisDecision::Skip
        ));
    }

    #[test]
    fn emit_on_high_confidence() {
        let mut e = SpeculationEngine::new();
        assert!(matches!(
            e.on_partial_tokens(&["hello".into()], 0.95),
            HypothesisDecision::Emit(_)
        ));
    }

    #[test]
    fn skip_second_hypothesis_too_soon() {
        let mut e = SpeculationEngine::new();
        e.on_partial_tokens(&["hello".into()], 0.7);
        assert!(matches!(
            e.on_partial_tokens(&["hello".into(), "world".into()], 0.75),
            HypothesisDecision::Skip
        ));
    }

    #[test]
    fn commit_existing_when_close() {
        let mut e = SpeculationEngine::new();
        e.on_partial_tokens(&["hello", "how", "are", "you"].map(String::from), 0.95);
        let d = e.on_final_tokens(&["hello", "how", "are", "you"].map(String::from));
        assert!(matches!(d, CommitDecision::CommitExisting(_)));
    }

    #[test]
    fn revert_on_large_difference() {
        let mut e = SpeculationEngine::new();
        e.on_partial_tokens(&["hello".into(), "world".into()], 0.95);
        let decision = e.on_final_tokens(&["goodbye", "see", "you", "later"].map(String::from));
        assert!(matches!(decision, CommitDecision::RevertThenCommit { .. }));
    }

    #[test]
    fn fresh_commit_with_no_prior_hypothesis() {
        let mut e = SpeculationEngine::new();
        let decision = e.on_final_tokens(&["hello".into()]);
        assert!(matches!(decision, CommitDecision::CommitFresh(_)));
    }

    #[test]
    fn revert_rate_tracks_correctly() {
        let mut e = SpeculationEngine::new();
        e.on_partial_tokens(&["a".into()], 0.95);
        std::thread::sleep(std::time::Duration::from_millis(210));
        e.on_partial_tokens(&["b".into()], 0.95);

        e.on_final_tokens(&["completely".into(), "different".into()]);
        e.on_final_tokens(&["totally".into(), "wrong".into()]);

        assert!(e.revert_rate() > 0.0);
    }

    #[test]
    fn edit_distance_ratio_identical() {
        let a = ["a", "b", "c"].map(String::from);
        assert_eq!(edit_distance_ratio(&a, &a), 0.0);
    }

    #[test]
    fn edit_distance_ratio_completely_different() {
        let a = ["a", "b"].map(String::from);
        let b = ["x", "y", "z"].map(String::from);
        let ratio = edit_distance_ratio(&a, &b);
        assert!(ratio > 0.5);
    }
}
