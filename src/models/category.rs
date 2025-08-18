use rusqlite::{
    Result, ToSql,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Category {
    Housing,
    Transportation,
    Food,
    Savings,
    Health,
    Personal,
    Other,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Category::Housing => write!(f, "Housing"),
            Category::Transportation => write!(f, "Transportation"),
            Category::Food => write!(f, "Food"),
            Category::Savings => write!(f, "Savings"),
            Category::Health => write!(f, "Health"),
            Category::Personal => write!(f, "Personal"),
            Category::Other => write!(f, "Other"),
        }
    }
}

impl ToSql for Category {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok(self.to_string().into())
    }
}

impl FromStr for Category {
    type Err = FromSqlError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Housing" => Ok(Category::Housing),
            "Transportation" => Ok(Category::Transportation),
            "Food" => Ok(Category::Food),
            "Savings" => Ok(Category::Savings),
            "Health" => Ok(Category::Health),
            "Personal" => Ok(Category::Personal),
            "Other" => Ok(Category::Other),
            _ => Err(FromSqlError::Other(format!("UnknownEnum {}", s).into())),
        }
    }
}

impl FromSql for Category {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_str()?
            .parse()
            .map_err(|e| FromSqlError::Other(Box::new(e)))
    }
}
