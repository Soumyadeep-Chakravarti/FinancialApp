use crate::domain::normalized_candidate::NormalizedCandidate;

/// A validated candidate — guaranteed complete and internally consistent.
///
/// This type can only be created by `ValidateStage`, making it impossible
/// to skip validation accidentally.
#[derive(Debug, Clone)]
pub struct ValidatedCandidate {
    pub(crate) inner: NormalizedCandidate,
}

impl ValidatedCandidate {
    /// Wrap a normalized candidate. Called only by ValidateStage.
    pub(crate) fn new(inner: NormalizedCandidate) -> Self {
        Self { inner }
    }

    /// Access the inner normalized candidate.
    pub fn inner(&self) -> &NormalizedCandidate {
        &self.inner
    }

    /// Unwrap into the inner normalized candidate.
    pub fn into_inner(self) -> NormalizedCandidate {
        self.inner
    }
}
