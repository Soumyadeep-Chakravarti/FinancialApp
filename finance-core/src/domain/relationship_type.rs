use std::fmt;
use std::str::FromStr;

/// Defines the semantic relationship between a transaction and a raw import.
///
/// This is the provenance model: every link between data carries meaning
/// beyond "these two rows are connected."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationshipType {
    /// Transaction was directly parsed from this raw import.
    ParsedFrom,

    /// Transaction was derived from another transaction (e.g., refund reversal).
    DerivedFrom,

    /// Transaction was manually corrected from this raw import.
    CorrectedBy,

    /// Transaction was split from a larger raw import or transaction.
    SplitFrom,

    /// Transaction was merged from multiple raw imports or transactions.
    MergedFrom,

    /// Transaction was imported alongside others in a batch.
    ImportedWith,
}

impl RelationshipType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ParsedFrom => "parsed_from",
            Self::DerivedFrom => "derived_from",
            Self::CorrectedBy => "corrected_by",
            Self::SplitFrom => "split_from",
            Self::MergedFrom => "merged_from",
            Self::ImportedWith => "imported_with",
        }
    }
}

impl fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for RelationshipType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "parsed_from" => Ok(Self::ParsedFrom),
            "derived_from" => Ok(Self::DerivedFrom),
            "corrected_by" => Ok(Self::CorrectedBy),
            "split_from" => Ok(Self::SplitFrom),
            "merged_from" => Ok(Self::MergedFrom),
            "imported_with" => Ok(Self::ImportedWith),
            _ => Err(format!("unknown relationship type: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_display() {
        for rt in [
            RelationshipType::ParsedFrom,
            RelationshipType::DerivedFrom,
            RelationshipType::CorrectedBy,
            RelationshipType::SplitFrom,
            RelationshipType::MergedFrom,
            RelationshipType::ImportedWith,
        ] {
            let s = rt.to_string();
            let parsed: RelationshipType = s.parse().unwrap();
            assert_eq!(parsed, rt);
        }
    }

    #[test]
    fn from_str_rejects_unknown() {
        assert!("bogus".parse::<RelationshipType>().is_err());
    }
}
