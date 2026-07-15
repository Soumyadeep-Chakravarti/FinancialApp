use crate::domain::parsed_candidate::ParsedCandidate;
use crate::domain::parser_info::ParserInfo;
use crate::domain::raw_import::RawImportRecord;
use crate::infrastructure::import::parser::{ParseError, SourceParser};

use super::common;

/// SBI UPI SMS parser.
///
/// Handles messages like:
/// - "SBI: Rs.500.00 debited from A/c XX1234 on 15-Jan-24. UPI Ref: 123456789012"
/// - "SBI: Rs.2500.00 credited to A/c XX1234 on 15-Jan-24. UPI Ref: 667788990011"
pub struct SbiParser;

impl SourceParser for SbiParser {
    fn parse(&self, record: &RawImportRecord) -> Result<ParsedCandidate, ParseError> {
        let text = match record.as_text() {
            Some(t) => t,
            None => return Err(ParseError::NotRecognized),
        };

        // Must start with SBI prefix
        if !text.starts_with("SBI") {
            return Err(ParseError::NotRecognized);
        }

        let amount = common::extract_amount(text)?;
        let kind = common::extract_kind(text)?;
        let reference = common::extract_reference(text);

        let mut candidate = ParsedCandidate::new(self.info())
            .with_amount(amount)
            .with_kind(kind);

        if let Some(r) = reference {
            candidate = candidate.with_reference(r);
        }

        Ok(candidate)
    }

    fn info(&self) -> ParserInfo {
        ParserInfo::new("SbiParser", 100)
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
    fn parses_sbi_debit() {
        let parser = SbiParser;
        let record = make_record("SBI: Rs.500.00 debited from A/c XX1234 on 15-Jan-24. UPI Ref: 123456789012");
        let result = parser.parse(&record).unwrap();

        assert_eq!(result.amount.unwrap().amount(), Decimal::new(50000, 2));
        assert_eq!(result.reference.unwrap(), "123456789012");
    }

    #[test]
    fn parses_sbi_credit() {
        let parser = SbiParser;
        let record = make_record("SBI: Rs.2500.00 credited to A/c XX1234 on 15-Jan-24. UPI Ref: 667788990011");
        let result = parser.parse(&record).unwrap();

        assert_eq!(result.amount.unwrap().amount(), Decimal::new(250000, 2));
        assert_eq!(result.reference.unwrap(), "667788990011");
    }

    #[test]
    fn rejects_non_sbi_message() {
        let parser = SbiParser;
        let record = make_record("UPI: Rs.500.00 sent to SWIGGY UPI Ref 123");
        assert!(matches!(parser.parse(&record), Err(ParseError::NotRecognized)));
    }

    #[test]
    fn parses_all_sbi_samples() {
        let parser = SbiParser;
        let samples = load_samples("sms/sbi/valid.txt");
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
        assert_eq!(successes.len(), samples.len(), "all SBI samples should parse");
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
