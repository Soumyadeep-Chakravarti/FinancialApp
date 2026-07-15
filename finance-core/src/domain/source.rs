/// The origin of an imported record.
///
/// Every imported record knows where it came from.
/// The source link table tracks which raw imports contributed to each canonical transaction.
///
/// Note: This is distinct from TransactionSource (salary, cash, UPI, card, etc.)
/// which describes how a payment was made, not where the data was imported from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImportSource {
    Sms,
    Notification,
    Csv,
    Pdf,
    Email,
    Api,
    Manual,
}

impl std::fmt::Display for ImportSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sms => write!(f, "SMS"),
            Self::Notification => write!(f, "Notification"),
            Self::Csv => write!(f, "CSV"),
            Self::Pdf => write!(f, "PDF"),
            Self::Email => write!(f, "Email"),
            Self::Api => write!(f, "API"),
            Self::Manual => write!(f, "Manual"),
        }
    }
}

impl std::str::FromStr for ImportSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sms" => Ok(Self::Sms),
            "notification" => Ok(Self::Notification),
            "csv" => Ok(Self::Csv),
            "pdf" => Ok(Self::Pdf),
            "email" => Ok(Self::Email),
            "api" => Ok(Self::Api),
            "manual" => Ok(Self::Manual),
            _ => Err(format!("unknown source type: {}", s)),
        }
    }
}
