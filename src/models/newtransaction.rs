use crate::models::Kind;
use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct NewTransaction {
    pub date: NaiveDate,
    pub title: String,
    pub amount: f64,
    pub kind: Kind,
}
