use crate::domain::{
    category::Category, enriched_candidate::EnrichedCandidate,
    validated_candidate::ValidatedCandidate,
};

use super::enricher::Enricher;
use super::merchant_normalizer::MerchantNormalizer;
use super::stage::{PipelineError, PipelineStage};

/// Orchestrates a sequence of enrichers.
///
/// Each enricher has a single responsibility. `EnrichStage` simply
/// runs them in order, passing the output of one as input to the next.
pub struct EnrichStage {
    enrichers: Vec<Box<dyn Enricher>>,
}

impl EnrichStage {
    pub fn new() -> Self {
        Self {
            enrichers: Vec::new(),
        }
    }

    pub fn with_enricher(mut self, enricher: Box<dyn Enricher>) -> Self {
        self.enrichers.push(enricher);
        self
    }
}

impl Default for EnrichStage {
    fn default() -> Self {
        Self::new()
            .with_enricher(Box::new(MerchantNormalizer))
            .with_enricher(Box::new(CategoryInferencer))
    }
}

impl PipelineStage for EnrichStage {
    type Input = ValidatedCandidate;
    type Output = EnrichedCandidate;

    fn run(&self, candidate: ValidatedCandidate) -> Result<EnrichedCandidate, PipelineError> {
        // Wrap in EnrichedCandidate with default category first
        let mut enriched = EnrichedCandidate::new(candidate, Category::Other);

        for enricher in &self.enrichers {
            enriched = enricher.enrich(enriched);
        }

        Ok(enriched)
    }
}

/// Infers category from merchant name using keyword matching.
pub struct CategoryInferencer;

impl Enricher for CategoryInferencer {
    fn enrich(&self, mut candidate: EnrichedCandidate) -> EnrichedCandidate {
        candidate.category = infer_category(&candidate.inner().inner().merchant.name);
        candidate
    }
}

fn infer_category(merchant: &str) -> Category {
    let lower = merchant.to_lowercase();

    if lower.contains("swiggy")
        || lower.contains("zomato")
        || lower.contains("food")
        || lower.contains("pizza")
        || lower.contains("burger")
        || lower.contains("restaurant")
        || lower.contains("cafe")
    {
        return Category::Food;
    }

    if lower.contains("bigbasket")
        || lower.contains("grocery")
        || lower.contains("dmart")
        || lower.contains("blinkit")
        || lower.contains("zepto")
        || lower.contains("instamart")
    {
        return Category::Groceries;
    }

    if lower.contains("amazon")
        || lower.contains("flipkart")
        || lower.contains("myntra")
        || lower.contains("ajio")
    {
        return Category::Shopping;
    }

    if lower.contains("ola")
        || lower.contains("uber")
        || lower.contains("rapido")
        || lower.contains("metro")
        || lower.contains("parking")
        || lower.contains("auto")
        || lower.contains("rickshaw")
    {
        return Category::Transport;
    }

    if lower.contains("pharmacy")
        || lower.contains("medical")
        || lower.contains("hospital")
        || lower.contains("doctor")
        || lower.contains("apollo")
        || lower.contains("medplus")
    {
        return Category::Healthcare;
    }

    if lower.contains("electricity")
        || lower.contains("bill")
        || lower.contains("recharge")
        || lower.contains("jio")
        || lower.contains("airtel")
        || lower.contains("broadband")
    {
        return Category::Bills;
    }

    if lower.contains("netflix")
        || lower.contains("hotstar")
        || lower.contains("prime")
        || lower.contains("movie")
        || lower.contains("bookmyshow")
    {
        return Category::Entertainment;
    }

    if lower.contains("donation")
        || lower.contains("university")
        || lower.contains("college")
        || lower.contains("school")
        || lower.contains("iisc")
        || lower.contains("iit")
    {
        return Category::Education;
    }

    if lower.contains("salary") || lower.contains("payroll") {
        return Category::Salary;
    }

    if lower.contains("mutual")
        || lower.contains("sip")
        || lower.contains("stock")
        || lower.contains("groww")
        || lower.contains("zerodha")
    {
        return Category::Investment;
    }

    if lower.contains("rent") || lower.contains("friend") || lower.contains("transfer") {
        return Category::Transfer;
    }

    if lower.contains("lic") || lower.contains("insurance") {
        return Category::Bills;
    }

    if lower.contains("mobile")
        || lower.contains("electronics")
        || lower.contains("croma")
        || lower.contains("vijay sales")
    {
        return Category::Shopping;
    }

    Category::Other
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        money::Money, normalized_candidate::NormalizedCandidate,
        parsed_candidate::ParsedCandidate, parser_info::ParserInfo,
        transaction_type::TransactionType,
    };
    use rust_decimal::Decimal;

    fn make_candidate(merchant: &str) -> ValidatedCandidate {
        let parsed = ParsedCandidate::new(ParserInfo::new("Test", 1))
            .with_amount(Money::new(Decimal::new(50000, 2)))
            .with_merchant(merchant)
            .with_kind(TransactionType::Debit);

        let normalized = NormalizedCandidate::from_parsed(parsed);
        ValidatedCandidate::new(normalized)
    }

    #[test]
    fn infers_food_category() {
        let stage = EnrichStage::default();
        for name in &["Swiggy", "Zomato", "Pizza Hut", "Burger King"] {
            let enriched = stage.run(make_candidate(name)).unwrap();
            assert_eq!(enriched.category, Category::Food, "merchant: {}", name);
        }
    }

    #[test]
    fn infers_groceries_category() {
        let stage = EnrichStage::default();
        for name in &["BigBasket", "Blinkit", "Zepto", "DMart"] {
            let enriched = stage.run(make_candidate(name)).unwrap();
            assert_eq!(enriched.category, Category::Groceries, "merchant: {}", name);
        }
    }

    #[test]
    fn infers_shopping_category() {
        let stage = EnrichStage::default();
        for name in &["Amazon", "Flipkart", "Myntra"] {
            let enriched = stage.run(make_candidate(name)).unwrap();
            assert_eq!(enriched.category, Category::Shopping, "merchant: {}", name);
        }
    }

    #[test]
    fn infers_transport_category() {
        let stage = EnrichStage::default();
        for name in &["Ola", "Uber", "Rapido", "metro parking"] {
            let enriched = stage.run(make_candidate(name)).unwrap();
            assert_eq!(
                enriched.category,
                Category::Transport,
                "merchant: {}",
                name
            );
        }
    }

    #[test]
    fn infers_bills_category() {
        let stage = EnrichStage::default();
        for name in &["electricity board", "Jio recharge", "LIC Premium"] {
            let enriched = stage.run(make_candidate(name)).unwrap();
            assert_eq!(enriched.category, Category::Bills, "merchant: {}", name);
        }
    }

    #[test]
    fn infers_entertainment_category() {
        let stage = EnrichStage::default();
        for name in &["Netflix", "Hotstar", "BookMyShow"] {
            let enriched = stage.run(make_candidate(name)).unwrap();
            assert_eq!(
                enriched.category,
                Category::Entertainment,
                "merchant: {}",
                name
            );
        }
    }

    #[test]
    fn infers_healthcare_category() {
        let stage = EnrichStage::default();
        for name in &["Apollo Pharmacy", "Medplus", "hospital bill"] {
            let enriched = stage.run(make_candidate(name)).unwrap();
            assert_eq!(
                enriched.category,
                Category::Healthcare,
                "merchant: {}",
                name
            );
        }
    }

    #[test]
    fn infers_salary_category() {
        let stage = EnrichStage::default();
        let enriched = stage.run(make_candidate("salary payment")).unwrap();
        assert_eq!(enriched.category, Category::Salary);
    }

    #[test]
    fn infers_investment_category() {
        let stage = EnrichStage::default();
        for name in &["Groww", "Zerodha", "mutual fund sip"] {
            let enriched = stage.run(make_candidate(name)).unwrap();
            assert_eq!(
                enriched.category,
                Category::Investment,
                "merchant: {}",
                name
            );
        }
    }

    #[test]
    fn infers_other_for_unknown_merchant() {
        let stage = EnrichStage::default();
        let enriched = stage.run(make_candidate("Unknown Shop 123")).unwrap();
        assert_eq!(enriched.category, Category::Other);
    }

    #[test]
    fn enrich_stage_returns_ok() {
        use crate::application::ingestion::stage::PipelineStage;

        let stage = EnrichStage::default();
        let enriched = stage.run(make_candidate("Swiggy")).unwrap();
        assert_eq!(enriched.category, Category::Food);
    }

    #[test]
    fn empty_enrichers_preserves_other_category() {
        let stage = EnrichStage::new();
        let enriched = stage.run(make_candidate("Swiggy")).unwrap();
        assert_eq!(enriched.category, Category::Other);
    }

    #[test]
    fn multiple_enrichers_compose() {
        // A trivial enricher that always sets category to Investment
        struct ForcedInvestment;
        impl Enricher for ForcedInvestment {
            fn enrich(&self, mut candidate: EnrichedCandidate) -> EnrichedCandidate {
                candidate.category = Category::Investment;
                candidate
            }
        }

        let stage = EnrichStage::new()
            .with_enricher(Box::new(CategoryInferencer))
            .with_enricher(Box::new(ForcedInvestment));

        // ForcedInvestment runs after CategoryInferencer, so it wins
        let enriched = stage.run(make_candidate("Swiggy")).unwrap();
        assert_eq!(enriched.category, Category::Investment);
    }
}
