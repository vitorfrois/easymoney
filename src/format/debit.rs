use polars::prelude::*;

pub struct DebitFormatter {
    df: DataFrame,
}

impl DebitFormatter {
    pub fn new(df: DataFrame) -> Self {
        DebitFormatter { df }
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

    fn add_kind(mut self) -> Self {
        let kind = when(col("amount").gt(0))
            .then(lit("Income"))
            .otherwise(lit("DebitPurchase"))
            .alias("kind");

        self.df = self
            .df
            .lazy()
            .with_column(kind)
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
                col("kind"),
            ])
            .collect()
            .expect("DataFrame select error");
        self
    }

    pub fn format(self) -> PolarsResult<DataFrame> {
        self.rename_columns()
            .add_short_description()
            .add_kind()
            .select_columns()
            .build()
    }

    pub fn build(self) -> PolarsResult<DataFrame> {
        Ok(self.df)
    }
}
