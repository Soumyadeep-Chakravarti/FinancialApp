use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Food,
    Groceries,
    Shopping,
    Transport,
    Bills,
    Entertainment,
    Healthcare,
    Education,
    Salary,
    Investment,
    Transfer,
    Other,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Category::Food => "Food",
            Category::Groceries => "Groceries",
            Category::Shopping => "Shopping",
            Category::Transport => "Transport",
            Category::Bills => "Bills",
            Category::Entertainment => "Entertainment",
            Category::Healthcare => "Healthcare",
            Category::Education => "Education",
            Category::Salary => "Salary",
            Category::Investment => "Investment",
            Category::Transfer => "Transfer",
            Category::Other => "Other",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Category {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Food" => Ok(Category::Food),
            "Groceries" => Ok(Category::Groceries),
            "Shopping" => Ok(Category::Shopping),
            "Transport" => Ok(Category::Transport),
            "Bills" => Ok(Category::Bills),
            "Entertainment" => Ok(Category::Entertainment),
            "Healthcare" => Ok(Category::Healthcare),
            "Education" => Ok(Category::Education),
            "Salary" => Ok(Category::Salary),
            "Investment" => Ok(Category::Investment),
            "Transfer" => Ok(Category::Transfer),
            "Other" => Ok(Category::Other),
            _ => Err(format!("Unknown category: {s}")),
        }
    }
}
