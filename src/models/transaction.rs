use crate::models::{Group, Kind};
use chrono::NaiveDate;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: u32,
    pub date: NaiveDate,
    pub title: String,
    pub amount: f64,
    pub kind: Kind,
    pub group: Option<Group>,
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} - {}, R${}",
            self.id, self.date, self.title, self.amount
        )
    }
}
