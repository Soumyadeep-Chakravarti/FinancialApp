use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Money {
    amount: Decimal,
}

impl Money {
    pub fn new(amount: Decimal) -> Self {
        Self { amount }
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }
}

impl Default for Money {
    fn default() -> Self {
        Self { amount: Decimal::ZERO }
    }
}
