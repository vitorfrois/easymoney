use rusqlite::{
    Result, ToSql,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
};
use std::fmt;
use std::str::FromStr;
use strum::EnumIter;

#[derive(Default, PartialEq, Eq, Debug, Clone, EnumIter)]
pub enum Category {
    #[default]
    Housing,
    Transportation,
    Food,
    Supermarket,
    Savings,
    Health,
    Personal,
    Trips,
    Other,
}

impl Category {
    pub fn next(&self) -> Self {
        match self {
            Self::Housing => Self::Transportation,
            Self::Transportation => Self::Food,
            Self::Food => Self::Supermarket,
            Self::Supermarket => Self::Savings,
            Self::Savings => Self::Health,
            Self::Health => Self::Personal,
            Self::Personal => Self::Trips,
            Self::Trips => Self::Other,
            Self::Other => Self::Housing,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Self::Housing => Self::Transportation,
            Self::Transportation => Self::Food,
            Self::Food => Self::Transportation,
            Self::Supermarket => Self::Food,
            Self::Savings => Self::Supermarket,
            Self::Health => Self::Savings,
            Self::Personal => Self::Health,
            Self::Trips => Self::Personal,
            Self::Other => Self::Housing,
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Category::Housing => write!(f, "Housing"),
            Category::Supermarket => write!(f, "Supermarket"),
            Category::Transportation => write!(f, "Transportation"),
            Category::Food => write!(f, "Food"),
            Category::Savings => write!(f, "Savings"),
            Category::Health => write!(f, "Health"),
            Category::Personal => write!(f, "Personal"),
            Category::Trips => write!(f, "Trips"),
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
            "Supermarket" => Ok(Category::Supermarket),
            "Trips" => Ok(Category::Trips),
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
