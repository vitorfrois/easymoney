use polars::prelude::*;

pub struct CreditFormatter {
    df: DataFrame,
}

impl CreditFormatter {
    pub fn new(df: DataFrame) -> Self {
        CreditFormatter { df }
    }

    fn select_columns(mut self) -> Self {
        let kind = when(col("amount").gt(0))
            .then(lit("CreditBillPayment"))
            .otherwise(lit("CreditPurchase"));

        self.df = self
            .df
            .lazy()
            .with_columns([kind.alias("kind"), col("amount") * lit(-1)])
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
