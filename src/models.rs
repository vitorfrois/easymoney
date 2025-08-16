use chrono::NaiveDate;
use rusqlite::{
    Result, ToSql,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
};

use std::{fmt, str::FromStr};

#[derive(Debug, Clone)]
pub enum Group {
    Fixed,
    Savings,
    Variable,
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Group::Fixed => write!(f, "Fixed"),
            Group::Savings => write!(f, "Savings"),
            Group::Variable => write!(f, "Variable"),
        }
    }
}

impl ToSql for Group {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok(self.to_string().into())
    }
}

impl FromStr for Group {
    type Err = FromSqlError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Fixed" => Ok(Group::Fixed),
            "Savings" => Ok(Group::Savings),
            "Variable" => Ok(Group::Variable),
            _ => Err(FromSqlError::Other(format!("UnknownEnum {}", s).into())),
        }
    }
}

impl FromSql for Group {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_str()?
            .parse()
            .map_err(|e| FromSqlError::Other(Box::new(e)))
    }
}

#[derive(Debug)]
pub enum Kind {
    Credit,
    Debit,
}

// impl fmt::Display for Kind {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             Kind::Credit => write!(f, "Credit"),
//             Kind::Debit => write!(f, "Debit"),
//         }
//     }
// }
//
// impl ToSql for Kind {
//     fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
//         Ok(self.to_string().into())
//     }
// }

#[derive(Debug, Clone)]
pub struct Transaction {
    pub date: NaiveDate,
    pub title: String,
    pub amount: f64,
    pub group: Option<Group>,
    // pub kind: Option<Kind>,
}
