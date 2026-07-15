// finance-core/src/domain/budget_period.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BudgetPeriod {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}
