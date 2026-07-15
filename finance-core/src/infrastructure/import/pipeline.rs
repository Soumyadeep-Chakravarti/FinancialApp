use crate::domain::{parsed_candidate::ParsedCandidate, raw_import::RawImportRecord};
use super::parser::{ParseError, SourceParser};

/// Iterates registered parsers against a raw import,
/// returning the first successful parse.
pub struct ImportPipeline {
    parsers: Vec<Box<dyn SourceParser + Send + Sync>>,
}

impl ImportPipeline {
    pub fn new() -> Self {
        Self {
            parsers: Vec::new(),
        }
    }

    pub fn register(&mut self, parser: Box<dyn SourceParser + Send + Sync>) {
        self.parsers.push(parser);
    }

    pub fn parse(&self, record: &RawImportRecord) -> Result<ParsedCandidate, ParseError> {
        for parser in &self.parsers {
            match parser.parse(record) {
                Ok(candidate) => return Ok(candidate),
                Err(ParseError::NotRecognized) => continue,
                Err(e) => return Err(e),
            }
        }
        Err(ParseError::NotRecognized)
    }
}

impl Default for ImportPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{money::Money, source::ImportSource, transaction_type::TransactionType};
    use rust_decimal::Decimal;

    struct FakeParser {
        keyword: &'static str,
    }

    impl SourceParser for FakeParser {
        fn parse(&self, record: &RawImportRecord) -> Result<ParsedCandidate, ParseError> {
            let text = record.as_text().unwrap_or("");
            if !text.contains(self.keyword) {
                return Err(ParseError::NotRecognized);
            }
            Ok(ParsedCandidate::new(self.info())
                .with_amount(Money::new(Decimal::new(100, 0)))
                .with_kind(TransactionType::Debit))
        }

        fn info(&self) -> crate::domain::parser_info::ParserInfo {
            crate::domain::parser_info::ParserInfo::new("FakeParser", 100)
        }
    }

    fn make_record(payload: &str) -> RawImportRecord {
        RawImportRecord::from_text(ImportSource::Sms, payload)
    }

    #[test]
    fn returns_first_matching_parser() {
        let mut pipeline = ImportPipeline::new();
        pipeline.register(Box::new(FakeParser { keyword: "SBI" }));
        pipeline.register(Box::new(FakeParser { keyword: "HDFC" }));

        let record = make_record("SBI: Rs.100 debited");
        let result = pipeline.parse(&record).unwrap();
        assert_eq!(result.amount.unwrap().amount(), Decimal::new(100, 0));
    }

    #[test]
    fn skips_non_matching_parser() {
        let mut pipeline = ImportPipeline::new();
        pipeline.register(Box::new(FakeParser { keyword: "SBI" }));
        pipeline.register(Box::new(FakeParser { keyword: "HDFC" }));

        let record = make_record("HDFC: Rs.200 debited");
        let result = pipeline.parse(&record).unwrap();
        assert_eq!(result.amount.unwrap().amount(), Decimal::new(100, 0));
    }

    #[test]
    fn returns_not_recognized_when_no_parser_matches() {
        let mut pipeline = ImportPipeline::new();
        pipeline.register(Box::new(FakeParser { keyword: "SBI" }));

        let record = make_record("random text message");
        let result = pipeline.parse(&record);
        assert!(matches!(result, Err(ParseError::NotRecognized)));
    }

    #[test]
    fn propagates_parse_errors() {
        struct ErrorParser;
        impl SourceParser for ErrorParser {
            fn parse(&self, record: &RawImportRecord) -> Result<ParsedCandidate, ParseError> {
                let text = record.as_text().unwrap_or("");
                if text.contains("AMBIGUOUS") {
                    Err(ParseError::InvalidFormat("ambiguous format".into()))
                } else {
                    Err(ParseError::NotRecognized)
                }
            }

            fn info(&self) -> crate::domain::parser_info::ParserInfo {
                crate::domain::parser_info::ParserInfo::new("ErrorParser", 100)
            }
        }

        let mut pipeline = ImportPipeline::new();
        pipeline.register(Box::new(ErrorParser));
        pipeline.register(Box::new(FakeParser { keyword: "SBI" }));

        // ErrorParser returns InvalidFormat, should propagate
        let record = make_record("AMBIGUOUS message");
        let result = pipeline.parse(&record);
        assert!(matches!(result, Err(ParseError::InvalidFormat(_))));
    }
}
