use crate::models::Kind;
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

    pub fn check_kind(&self, df: &DataFrame) -> Option<Kind> {
        match df.shape().1 {
            3 => Some(Kind::Credit),
            4 => Some(Kind::Debit),
            _ => None,
        }
    }

    pub fn concat(&mut self, df: DataFrame) {
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

pub struct AccountFormatter {
    df: DataFrame,
}

impl AccountFormatter {
    pub fn new(df: DataFrame) -> Self {
        AccountFormatter { df }
    }

    fn rename_columns(mut self) -> Self {
        self.df
            .set_column_names(&["date", "amount", "index", "description"])
            .unwrap();
        self
    }

    fn add_aux_columns(mut self) -> Self {
        let split_name_series = col("description").str().split(lit("-")).alias("split");
        let length_series = col("description")
            .str()
            .split(lit("-"))
            .len()
            .alias("length");

        self.df = self
            .df
            .lazy()
            .with_columns(vec![split_name_series, length_series])
            .collect()
            .expect("DataFrame error");

        self
    }

    fn add_short_description(mut self) -> Self {
        self = self.add_aux_columns();

        let short_description = when(col("length").gt_eq(2))
            .then(col("split").list().get(1.into(), true))
            .otherwise(col("split").list().get(2.into(), true))
            .alias("description");

        self.df = self
            .df
            .lazy()
            .with_column(short_description)
            .collect()
            .expect("DataFrame new columns error");
        self
    }

    fn select_columns(mut self) -> Self {
        self.df = self
            .df
            .lazy()
            .select([
                col("date"),
                col("description").alias("title"),
                col("amount"),
                lit("account").alias("kind"),
            ])
            .collect()
            .expect("DataFrame select error");
        self
    }

    pub fn format(self) -> PolarsResult<DataFrame> {
        self.rename_columns()
            .add_short_description()
            .select_columns()
            .build()
    }

    pub fn build(self) -> PolarsResult<DataFrame> {
        Ok(self.df)
    }
}

pub struct CreditFormatter {
    df: DataFrame,
}

impl CreditFormatter {
    pub fn new(df: DataFrame) -> Self {
        CreditFormatter { df }
    }

    fn select_columns(mut self) -> Self {
        self.df = self
            .df
            .lazy()
            .with_columns([lit("credit").alias("kind"), col("amount") * lit(-1)])
            .collect()
            .expect("DataFrame select error");
        self
    }

    pub fn format(self) -> PolarsResult<DataFrame> {
        self.select_columns().build()
    }

    pub fn build(self) -> PolarsResult<DataFrame> {
        Ok(self.df)
    }
}
