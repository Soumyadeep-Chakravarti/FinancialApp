use crate::domain::parsed_candidate::ParsedCandidate;
use crate::domain::parser_info::ParserInfo;
use crate::domain::raw_import::RawImportRecord;
use crate::infrastructure::import::parser::{ParseError, SourceParser};

use super::common;

/// Generic UPI SMS parser (fallback).
///
/// Handles common UPI SMS formats:
/// - "UPI: Rs.500.00 sent to SWIGGY UPI Ref 123456789012"
/// - "UPI: Rs.1500.00 received from John UPI Ref 987654321012"
///
/// This parser is always last in the pipeline.
/// It matches messages containing UPI keywords that weren't caught
/// by more specific bank parsers.
pub struct GenericUpiParser;

impl SourceParser for GenericUpiParser {
    fn parse(&self, record: &RawImportRecord) -> Result<ParsedCandidate, ParseError> {
        let text = match record.as_text() {
            Some(t) => t,
            None => return Err(ParseError::NotRecognized),
        };

        // Must contain UPI-related keywords
        if !text.contains("UPI") && !text.contains("debited") && !text.contains("credited") {
            return Err(ParseError::NotRecognized);
        }

        let amount = common::extract_amount(text)?;
        let kind = common::extract_kind(text)?;
        let merchant = common::extract_merchant(text);
        let reference = common::extract_reference(text);

        let mut candidate = ParsedCandidate::new(self.info())
            .with_amount(amount)
            .with_kind(kind);

        if let Some(m) = merchant {
            candidate.merchant = Some(m);
        }

        if let Some(r) = reference {
            candidate = candidate.with_reference(r);
        }

        Ok(candidate)
    }

    fn info(&self) -> ParserInfo {
        ParserInfo::new("GenericUpiParser", 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::source::ImportSource;
    use rust_decimal::Decimal;

    fn make_record(text: &str) -> RawImportRecord {
        RawImportRecord::from_text(ImportSource::Sms, text)
    }

    #[test]
    fn parses_generic_debit() {
        let parser = GenericUpiParser;
        let record = make_record("UPI: Rs.500.00 sent to SWIGGY UPI Ref 123456789012");
        let result = parser.parse(&record).unwrap();

        assert_eq!(result.amount.unwrap().amount(), Decimal::new(50000, 2));
        assert_eq!(result.merchant.unwrap().name, "SWIGGY");
        assert_eq!(result.reference.unwrap(), "123456789012");
    }

    #[test]
    fn parses_generic_credit() {
        let parser = GenericUpiParser;
        let record = make_record("UPI: Rs.1500.00 received from John UPI Ref 987654321012");
        let result = parser.parse(&record).unwrap();

        assert_eq!(result.amount.unwrap().amount(), Decimal::new(150000, 2));
        assert_eq!(result.merchant.unwrap().name, "John");
    }

    #[test]
    fn parses_inr_amount() {
        let parser = GenericUpiParser;
        let record = make_record("UPI: INR 1200.00 paid to Amazon Pay UPI Ref 987654321012");
        let result = parser.parse(&record).unwrap();

        assert_eq!(result.amount.unwrap().amount(), Decimal::new(120000, 2));
    }

    #[test]
    fn parses_amount_with_commas() {
        let parser = GenericUpiParser;
        let record = make_record("UPI: Rs.10,500.00 sent to landlord UPI Ref 111111111111");
        let result = parser.parse(&record).unwrap();

        assert_eq!(result.amount.unwrap().amount(), Decimal::new(1050000, 2));
    }

    #[test]
    fn rejects_non_upi_message() {
        let parser = GenericUpiParser;
        let record = make_record("Your OTP is 123456. Valid for 5 minutes.");
        assert!(matches!(parser.parse(&record), Err(ParseError::NotRecognized)));
    }

    #[test]
    fn parses_all_generic_samples() {
        let parser = GenericUpiParser;
        let samples = load_samples("sms/generic/valid.txt");
        let (successes, failures): (Vec<_>, Vec<_>) = samples.iter().map(|s| {
            let record = make_record(s);
            match parser.parse(&record) {
                Ok(c) => Ok((s.clone(), c)),
                Err(e) => Err((s.clone(), e.to_string())),
            }
        }).partition(|r| r.is_ok());

        let successes: Vec<_> = successes.into_iter().map(|r| r.unwrap()).collect();
        let failures: Vec<_> = failures.into_iter().map(|r| r.unwrap_err()).collect();

        for (sample, err) in &failures {
            eprintln!("FAILED: {} -> {}", sample, err);
        }
        assert_eq!(successes.len(), samples.len(), "all generic samples should parse");
    }

    fn load_samples(path: &str) -> Vec<String> {
        let full_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/samples")
            .join(path);
        let content = std::fs::read_to_string(&full_path)
            .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", full_path, e));
        content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect()
    }
}
