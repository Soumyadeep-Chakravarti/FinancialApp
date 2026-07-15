mod harness;

use harness::load_samples;

struct StubParser;

impl finance_core::infrastructure::import::parser::SourceParser for StubParser {
    fn parse(&self, _record: &finance_core::domain::raw_import::RawImportRecord) -> Result<finance_core::domain::parsed_candidate::ParsedCandidate, finance_core::infrastructure::import::parser::ParseError> {
        use finance_core::infrastructure::import::parser::ParseError;
        Err(ParseError::NotRecognized)
    }

    fn info(&self) -> finance_core::domain::parser_info::ParserInfo {
        finance_core::domain::parser_info::ParserInfo::new("StubParser", 100)
    }
}

#[test]
fn golden_files_exist() {
    let files = [
        "shared/non_upi.txt",
        "shared/bank_transfer.txt",
        "sms/sbi/valid.txt",
        "sms/generic/valid.txt",
    ];
    for f in files {
        let samples = load_samples(f);
        assert!(!samples.is_empty(), "golden file {} is empty or missing", f);
    }
}

#[test]
fn non_upi_messages_are_rejected() {
    let parser = StubParser;
    let samples = load_samples("shared/non_upi.txt");
    let (successes, failures) = harness::parse_all(&parser, &samples);
    assert_eq!(successes.len(), 0, "non-UPI messages should not parse");
    assert_eq!(failures.len(), samples.len());
}

#[test]
fn bank_transfer_messages_are_rejected() {
    let parser = StubParser;
    let samples = load_samples("shared/bank_transfer.txt");
    let (successes, _failures) = harness::parse_all(&parser, &samples);
    assert_eq!(successes.len(), 0, "bank transfer messages should not parse");
}

#[test]
fn sample_counts() {
    assert_eq!(load_samples("shared/non_upi.txt").len(), 8);
    assert_eq!(load_samples("shared/bank_transfer.txt").len(), 3);
    assert_eq!(load_samples("sms/sbi/valid.txt").len(), 10);
    assert_eq!(load_samples("sms/generic/valid.txt").len(), 20);
}
