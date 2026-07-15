/// finance-core/src/domain/budget.rs
use uuid::Uuid;

use crate::domain::{budget_period::BudgetPeriod, category::Category, money::Money};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Budget {
    pub id: Uuid,

    pub name: String,

    /// Maximum amount that may be spent during the budget period.
    pub limit: Money,

    /// None = overall budget.
    /// Some(Category::Food) = food budget.
    pub category: Option<Category>,

    /// Monthly, Weekly, etc.
    pub period: BudgetPeriod,
}

impl Budget {
    pub fn new(
        name: String,
        limit: Money,
        category: Option<Category>,
        period: BudgetPeriod,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            limit,
            category,
            period,
        }
    }
}
