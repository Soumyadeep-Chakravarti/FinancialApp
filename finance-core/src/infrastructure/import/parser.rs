use thiserror::Error;

use crate::domain::{parsed_candidate::ParsedCandidate, parser_info::ParserInfo, raw_import::RawImportRecord};

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Data not recognized by any parser")]
    NotRecognized,

    #[error("Could not parse amount")]
    NoAmount,

    #[error("Parsing failed: {0}")]
    InvalidFormat(String),
}

/// Trait for parsing raw source data into structured candidates.
///
/// Each source type (SMS provider, bank CSV, notification) implements this trait.
/// The parser receives a RawImportRecord and extracts what it can.
/// Return `Err(ParseError::NotRecognized)` if the data doesn't
/// match this parser's format.
pub trait SourceParser {
    fn parse(&self, record: &RawImportRecord) -> Result<ParsedCandidate, ParseError>;
    fn info(&self) -> ParserInfo;
}
