use crate::domain::{category::Category, validated_candidate::ValidatedCandidate};

/// An enriched candidate — all business rules have been applied.
///
/// This is the final form before constructing a `Transaction`.
#[derive(Debug, Clone)]
pub struct EnrichedCandidate {
    pub(crate) inner: ValidatedCandidate,
    pub category: Category,
}

impl EnrichedCandidate {
    /// Wrap a validated candidate with enrichment data. Called only by EnrichStage.
    pub(crate) fn new(inner: ValidatedCandidate, category: Category) -> Self {
        Self { inner, category }
    }

    /// Access the inner validated candidate.
    pub fn inner(&self) -> &ValidatedCandidate {
        &self.inner
    }

    /// Unwrap into the validated candidate.
    pub fn into_inner(self) -> ValidatedCandidate {
        self.inner
    }

    /// Replace the merchant name. Used by mutating enrichers (e.g. MerchantNormalizer).
    pub fn set_merchant_name(&mut self, name: String) {
        self.inner.inner.merchant.name = name;
    }
}
