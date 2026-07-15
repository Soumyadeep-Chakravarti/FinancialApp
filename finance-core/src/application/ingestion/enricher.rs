use crate::domain::enriched_candidate::EnrichedCandidate;

/// An enrichment step that derives or modifies data on an `EnrichedCandidate`.
///
/// Enrichers are composable: `EnrichStage` runs them in sequence.
/// Each enricher has a single responsibility — e.g., category inference,
/// merchant normalization, recurring detection.
///
/// # Ordering matters
///
/// Some enrichers mutate existing data (MerchantNormalizer),
/// others derive new information (CategoryInferencer).
/// Mutating enrichers should run before annotating ones.
pub trait Enricher {
    fn enrich(&self, candidate: EnrichedCandidate) -> EnrichedCandidate;
}
