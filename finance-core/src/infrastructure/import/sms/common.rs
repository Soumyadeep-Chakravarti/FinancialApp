use once_cell::sync::Lazy;
use regex::Regex;

use crate::domain::{merchant::Merchant, money::Money, transaction_type::TransactionType};
use crate::infrastructure::import::parser::ParseError;

use rust_decimal::Decimal;

/// Matches: "Rs.500.00" or "INR 500.00" or "Rs. 500.00" or "INR500.00" or "₹500"
pub(crate) static AMOUNT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:Rs\.?|INR|₹)\s*(\d[\d,]*\.?\d*)").unwrap()
});

/// Matches: "UPI Ref Y" or "Ref: Y" or "UPI Ref: Y" or "Ref No Y"
pub(crate) static REF_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:UPI\s+)?Ref[:\s]*(?:No[:\s]*)?(\d+)").unwrap()
});

/// Matches: "sent to X" or "paid to X"
pub(crate) static TO_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:sent|paid)\s+to\s+(.+?)(?:\s+UPI|\s+Ref|\s*$)").unwrap()
});

/// Matches: "received from X"
pub(crate) static FROM_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:received|credited)\s+from\s+(.+?)(?:\s+UPI|\s+Ref|\s*$)").unwrap()
});

/// Matches: "debited from"
pub(crate) static DEBIT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"debited\s+from").unwrap()
});

/// Matches: "credited to"
pub(crate) static CREDIT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"credited\s+to").unwrap()
});

/// Extract amount from text using common regex.
pub(crate) fn extract_amount(text: &str) -> Result<Money, ParseError> {
    AMOUNT_RE
        .captures(text)
        .and_then(|cap| {
            let raw = cap[1].replace(',', "");
            raw.parse::<Decimal>().ok()
        })
        .map(Money::new)
        .ok_or(ParseError::NoAmount)
}

/// Extract transaction type from text using common keywords.
pub(crate) fn extract_kind(text: &str) -> Result<TransactionType, ParseError> {
    if DEBIT_RE.is_match(text) || text.contains("sent to") || text.contains("paid to") {
        Ok(TransactionType::Debit)
    } else if CREDIT_RE.is_match(text) || text.contains("received from") {
        Ok(TransactionType::Credit)
    } else {
        Err(ParseError::NotRecognized)
    }
}

/// Extract merchant from "sent to X" or "received from X" patterns.
pub(crate) fn extract_merchant(text: &str) -> Option<Merchant> {
    TO_RE
        .captures(text)
        .or_else(|| FROM_RE.captures(text))
        .map(|cap| Merchant::new(cap[1].trim()))
}

/// Extract UPI reference number.
pub(crate) fn extract_reference(text: &str) -> Option<String> {
    REF_RE
        .captures(text)
        .map(|cap| cap[1].to_string())
}
