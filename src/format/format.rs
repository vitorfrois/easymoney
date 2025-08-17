use crate::format::{CreditFormatter, DebitFormatter};
use crate::models::CsvType;
use polars::prelude::*;

#[derive(Clone)]
pub struct Formatter {
    pub df: LazyFrame,
    empty: bool,
}

impl Formatter {
    pub fn new(df: DataFrame) -> Self {
        Formatter {
            df: df.lazy(),
            empty: true,
        }
    }

    fn check_kind(&self, df: &DataFrame) -> Option<CsvType> {
        match df.shape().1 {
            3 => Some(CsvType::Credit),
            4 => Some(CsvType::Debit),
            _ => None,
        }
    }

    pub fn add(&mut self, df: DataFrame) {
        match self.check_kind(&df) {
            Some(CsvType::Credit) => {
                self.concat(CreditFormatter::new(df).format().expect("Concat error"))
            }
            Some(CsvType::Debit) => {
                self.concat(DebitFormatter::new(df).format().expect("Concat error"))
            }
            None => {}
        };
    }

    fn concat(&mut self, df: DataFrame) {
        if self.empty {
            self.df = df.lazy();
            self.empty = false;
        } else {
            self.df =
                concat([df.lazy(), self.df.clone()], UnionArgs::default()).expect("Concat error")
        }
    }

    pub fn build(self) -> PolarsResult<DataFrame> {
        Ok(self
            .df
            .with_columns([col("title").str().strip_chars(lit(" "))])
            .collect()?
            .drop_nulls::<String>(None)?)
    }
}
