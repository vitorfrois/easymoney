use rusqlite::{
    Result, ToSql,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    CreditPurchase,
    DebitPurchase,
    Income,
    CreditBillPayment,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Kind::CreditPurchase => write!(f, "CreditPurchase"),
            Kind::DebitPurchase => write!(f, "DebitPurchase"),
            Kind::Income => write!(f, "Income"),
            Kind::CreditBillPayment => write!(f, "CreditBillPayment"),
        }
    }
}

impl ToSql for Kind {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok(self.to_string().into())
    }
}

impl FromStr for Kind {
    type Err = FromSqlError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CreditPurchase" => Ok(Kind::CreditPurchase),
            "DebitPurchase" => Ok(Kind::DebitPurchase),
            "Income" => Ok(Kind::Income),
            "CreditBillPayment" => Ok(Kind::CreditBillPayment),
            _ => Err(FromSqlError::Other(format!("UnknownEnum {}", s).into())),
        }
    }
}

impl FromSql for Kind {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_str()?
            .parse()
            .map_err(|e| FromSqlError::Other(Box::new(e)))
    }
}
