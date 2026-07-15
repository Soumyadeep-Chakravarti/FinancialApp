use std::collections::HashMap;
use std::sync::LazyLock;

use crate::domain::enriched_candidate::EnrichedCandidate;

use super::enricher::Enricher;

/// Canonical merchant names — maps known aliases to a single form.
///
/// Keys must be UPPERCASE (since NormalizedCandidate already uppercases).
/// This is the source of truth for merchant identity resolution.
static ALIAS_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let pairs: Vec<(&str, &str)> = vec![
        // Food
        ("SWIGGY", "SWIGGY"),
        ("SWIGGY LTD", "SWIGGY"),
        ("SWIGGY PAYMENTS", "SWIGGY"),
        ("SWIGGY LIMITED", "SWIGGY"),
        ("ZOMATO", "ZOMATO"),
        ("ZOMATO LIMITED", "ZOMATO"),
        ("ZOMATO LTD", "ZOMATO"),
        // Shopping
        ("AMAZON", "AMAZON"),
        ("AMAZON PAY", "AMAZON"),
        ("AMAZONPAY", "AMAZON"),
        ("FLIPKART", "FLIPKART"),
        ("FLIPKART INTERNET", "FLIPKART"),
        ("FLIPKART PRIVATE", "FLIPKART"),
        ("MYNTRA", "MYNTRA"),
        ("MYNTRA DESIGNS", "MYNTRA"),
        // Transport
        ("OLA", "OLA"),
        ("OLA CABS", "OLA"),
        ("OLA MONEY", "OLA"),
        ("UBER", "UBER"),
        ("UBER INDIA", "UBER"),
        // Groceries
        ("BIGBASKET", "BIGBASKET"),
        ("BIG BASKET", "BIGBASKET"),
        ("BBINSTAMART", "BIGBASKET"),
        ("BLINKIT", "BLINKIT"),
        ("BLINKIT (OPC)", "BLINKIT"),
        ("ZEPTO", "ZEPTO"),
        ("DMART", "DMART"),
        ("DMART READY", "DMART"),
        // Bills
        ("JIO", "JIO"),
        ("RELIANCE JIO", "JIO"),
        ("AIRTEL", "AIRTEL"),
        ("BHARTI AIRTEL", "AIRTEL"),
        // Entertainment
        ("NETFLIX", "NETFLIX"),
        ("HOTSTAR", "HOTSTAR"),
        ("DISNEY HOTSTAR", "HOTSTAR"),
        ("BOOKMYSHOW", "BOOKMYSHOW"),
        ("BMS", "BOOKMYSHOW"),
        // Healthcare
        ("APOLLO", "APOLLO"),
        ("APOLLO PHARMACY", "APOLLO"),
        ("APOLLO HOSPITALS", "APOLLO"),
        ("MEDPLUS", "MEDPLUS"),
        ("MEDPLUS PHARMACY", "MEDPLUS"),
        // Investment
        ("GROWW", "GROWW"),
        ("GROWW INV", "GROWW"),
        ("ZERODHA", "ZERODHA"),
        ("ZERODHA BROKING", "ZERODHA"),
    ];

    pairs.into_iter().collect()
});

/// Strips trailing transaction reference suffixes from merchant names.
///
/// Common patterns in SMS: `SWIGGY*123456`, `ZOMATO#TXN789`, `AMAZON@UPI`.
/// These are transaction IDs, not part of the merchant name.
fn strip_transaction_suffix(name: &str) -> &str {
    // Strip trailing non-alphanumeric suffix starting with * # @ ! ~
    if let Some(pos) = name.rfind(|c: char| c == '*' || c == '#' || c == '@' || c == '!' || c == '~')
    {
        let suffix = &name[pos..];
        // Only strip if suffix is purely reference-like (no spaces, all after the delimiter)
        if !suffix.contains(' ') && suffix.len() > 1 {
            return &name[..pos];
        }
    }
    name
}

/// Collapse multiple whitespace into single spaces and trim.
fn collapse_whitespace(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut prev_space = false;

    for c in name.chars() {
        if c.is_ascii_whitespace() {
            if !prev_space {
                result.push(' ');
                prev_space = true;
            }
        } else {
            result.push(c);
            prev_space = false;
        }
    }

    result.trim().to_string()
}

/// Normalize a merchant name: strip suffixes, collapse whitespace, uppercase, resolve alias.
fn normalize_merchant_name(raw: &str) -> String {
    let stripped = strip_transaction_suffix(raw);
    let collapsed = collapse_whitespace(stripped);
    let upper = collapsed.to_uppercase();

    // Look up canonical form
    if let Some(&canonical) = ALIAS_MAP.get(upper.as_str()) {
        canonical.to_string()
    } else {
        upper
    }
}

/// Normalizes merchant names to a canonical form.
///
/// This is a mutating enricher — it modifies the merchant name in place.
/// It must run **before** `CategoryInferencer` so category inference
/// sees the canonical merchant name.
///
/// Processing order:
/// 1. Strip transaction suffixes (`*123456`, `#TXN789`)
/// 2. Collapse whitespace, trim
/// 3. Uppercase
/// 4. Resolve known aliases
pub struct MerchantNormalizer;

impl Enricher for MerchantNormalizer {
    fn enrich(&self, mut candidate: EnrichedCandidate) -> EnrichedCandidate {
        let canonical = normalize_merchant_name(&candidate.inner().inner().merchant.name);
        candidate.set_merchant_name(canonical);
        candidate
    }
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

    fn make_candidate(merchant: &str) -> EnrichedCandidate {
        let parsed = ParsedCandidate::new(ParserInfo::new("Test", 1))
            .with_amount(Money::new(Decimal::new(50000, 2)))
            .with_merchant(merchant)
            .with_kind(TransactionType::Debit);

        let normalized = NormalizedCandidate::from_parsed(parsed);
        let validated = crate::domain::validated_candidate::ValidatedCandidate::new(normalized);
        crate::domain::enriched_candidate::EnrichedCandidate::new(
            validated,
            crate::domain::category::Category::Other,
        )
    }

    #[test]
    fn strips_asterisk_suffix() {
        let result = normalize_merchant_name("SWIGGY*123456");
        assert_eq!(result, "SWIGGY");
    }

    #[test]
    fn strips_hash_suffix() {
        let result = normalize_merchant_name("ZOMATO#TXN789");
        assert_eq!(result, "ZOMATO");
    }

    #[test]
    fn strips_at_suffix() {
        let result = normalize_merchant_name("AMAZON@UPI");
        assert_eq!(result, "AMAZON");
    }

    #[test]
    fn does_not_strip_mid_name_special_chars() {
        // "V#IJAY SALES" has a # but not at the end — should NOT strip
        let result = normalize_merchant_name("V#IJAY SALES");
        assert_eq!(result, "V#IJAY SALES");
    }

    #[test]
    fn collapses_whitespace() {
        let result = normalize_merchant_name("  SWIGGY   PAYMENTS  ");
        // Alias resolves "SWIGGY PAYMENTS" → "SWIGGY"
        assert_eq!(result, "SWIGGY");
    }

    #[test]
    fn resolves_alias_swiggy_ltd() {
        let result = normalize_merchant_name("SWIGGY LTD");
        assert_eq!(result, "SWIGGY");
    }

    #[test]
    fn resolves_alias_flipkart_private() {
        let result = normalize_merchant_name("FLIPKART PRIVATE");
        assert_eq!(result, "FLIPKART");
    }

    #[test]
    fn resolves_alias_uber_india() {
        let result = normalize_merchant_name("UBER INDIA");
        assert_eq!(result, "UBER");
    }

    #[test]
    fn resolves_alias_ola_cabs() {
        let result = normalize_merchant_name("OLA CABS");
        assert_eq!(result, "OLA");
    }

    #[test]
    fn resolves_alias_netflix() {
        let result = normalize_merchant_name("NETFLIX");
        assert_eq!(result, "NETFLIX");
    }

    #[test]
    fn preserves_unknown_merchant() {
        let result = normalize_merchant_name("RANDOM SHOP 123");
        assert_eq!(result, "RANDOM SHOP 123");
    }

    #[test]
    fn empty_becomes_empty() {
        let result = normalize_merchant_name("");
        assert_eq!(result, "");
    }

    #[test]
    fn enricher_updates_candidate() {
        let candidate = make_candidate("Swiggy*123456");
        let enricher = MerchantNormalizer;
        let enriched = enricher.enrich(candidate);
        assert_eq!(enriched.inner().inner().merchant.name, "SWIGGY");
    }

    #[test]
    fn enricher_preserves_upi_id() {
        let parsed = ParsedCandidate::new(ParserInfo::new("Test", 1))
            .with_amount(Money::new(Decimal::new(50000, 2)))
            .with_merchant("SWIGGY*123456")
            .with_kind(TransactionType::Debit);
        let mut normalized = NormalizedCandidate::from_parsed(parsed);
        normalized.merchant.upi_id = Some("swiggy@ybl".to_string());
        let validated = crate::domain::validated_candidate::ValidatedCandidate::new(normalized);
        let candidate = crate::domain::enriched_candidate::EnrichedCandidate::new(
            validated,
            crate::domain::category::Category::Other,
        );

        let enricher = MerchantNormalizer;
        let enriched = enricher.enrich(candidate);

        assert_eq!(enriched.inner().inner().merchant.name, "SWIGGY");
        assert_eq!(
            enriched.inner().inner().merchant.upi_id,
            Some("swiggy@ybl".to_string())
        );
    }

    #[test]
    fn alias_map_has_no_duplicate_keys() {
        // Each key should be unique — no two aliases for the same input
        let mut seen = std::collections::HashSet::new();
        for (&key, _) in ALIAS_MAP.iter() {
            assert!(seen.insert(key), "duplicate key in alias map: {}", key);
        }
    }

    #[test]
    fn alias_map_keys_are_uppercase() {
        for (&key, _) in ALIAS_MAP.iter() {
            assert_eq!(key, key.to_uppercase(), "key '{}' is not uppercase", key);
        }
    }
}
