use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransactionType {
    Credit,
    Debit,
}

impl Default for TransactionType {
    fn default() -> Self {
        Self::Credit
    }
}

impl TransactionType {
    pub fn is_credit(self) -> bool {
        matches!(self, Self::Credit)
    }

    pub fn is_debit(self) -> bool {
        matches!(self, Self::Debit)
    }
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TransactionType::Credit => "Credit",
            TransactionType::Debit => "Debit",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for TransactionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Credit" => Ok(TransactionType::Credit),
            "Debit" => Ok(TransactionType::Debit),
            _ => Err(format!("unknown transaction type: {}", s)),
        }
    }
}
