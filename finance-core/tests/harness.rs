use std::path::Path;
use std::fs;
use finance_core::infrastructure::import::parser::SourceParser;
use finance_core::domain::parsed_candidate::ParsedCandidate;

/// Load all samples from a golden test file.
/// Each non-empty line is treated as one sample.
pub fn load_samples(path: &str) -> Vec<String> {
    let full_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/samples")
        .join(path);
    let content = fs::read_to_string(&full_path)
        .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", full_path, e));
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect()
}

/// Parse all samples through a parser, returning (successes, failures).
pub fn parse_all(
    parser: &(impl SourceParser + ?Sized),
    samples: &[String],
) -> (Vec<(String, ParsedCandidate)>, Vec<(String, String)>) {
    let mut successes = Vec::new();
    let mut failures = Vec::new();
    for sample in samples {
        let record = finance_core::domain::raw_import::RawImportRecord::from_text(
            finance_core::domain::source::ImportSource::Sms,
            sample,
        );
        match parser.parse(&record) {
            Ok(candidate) => successes.push((sample.clone(), candidate)),
            Err(e) => failures.push((sample.clone(), e.to_string())),
        }
    }
    (successes, failures)
}

/// Assert that all samples parsed successfully and return the candidates.
#[allow(dead_code)]
pub fn assert_all_parsed(
    parser: &(impl SourceParser + ?Sized),
    samples: &[String],
) -> Vec<ParsedCandidate> {
    let (successes, failures) = parse_all(parser, samples);
    for (sample, err) in &failures {
        eprintln!("FAILED: {} -> {}", sample, err);
    }
    assert!(
        failures.is_empty(),
        "{} of {} samples failed to parse",
        failures.len(),
        samples.len()
    );
    successes.into_iter().map(|(_, c)| c).collect()
}
