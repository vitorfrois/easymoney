use chrono::Datelike;
use itertools::multizip;
use polars::prelude::*;
use rusqlite::Result;
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;

use crate::models::Category;

pub mod app;
pub mod db;
pub mod event;
pub mod format;
pub mod labeling;
pub mod models;
pub mod tui;

fn read_csv(path: &fs::DirEntry) -> PolarsResult<DataFrame> {
    CsvReadOptions::default()
        .with_has_header(true)
        .map_parse_options(|parse_options| parse_options.with_try_parse_dates(true))
        .try_into_reader_with_file_path(Some(path.path()))?
        .finish()
}

fn read_folder(path: String) -> PolarsResult<DataFrame> {
    let paths = fs::read_dir(path)?;

    let mut formatter = format::Formatter::new(DataFrame::default());

    for path in paths {
        let df = read_csv(&path?)?;
        formatter.add(df);
    }

    let full_df = formatter.build()?;

    Ok(full_df)
}

fn convert_df(df: DataFrame) -> Vec<models::NewTransaction> {
    let date_series = df
        .column("date")
        .expect("CSV Date error")
        .date()
        .unwrap()
        .as_date_iter();
    let title_series = df
        .column("title")
        .expect("CSV Str error")
        .str()
        .unwrap()
        .iter();
    let amount_series = df
        .column("amount")
        .expect("CSV Float error")
        .f64()
        .unwrap()
        .iter();
    let group_series = df
        .column("kind")
        .expect("CSV Str error")
        .str()
        .unwrap()
        .iter();

    let combined = multizip((date_series, title_series, amount_series, group_series));
    let res: Vec<models::NewTransaction> = combined
        .map(|(date, title, amount, kind)| models::NewTransaction {
            date: date.unwrap(),
            title: title.expect("DateErr").to_string(),
            amount: amount.unwrap(),
            kind: models::Kind::from_str(kind.expect("Kind not found")).unwrap(),
        })
        .collect();
    res
}

fn _get_transactions_by_month(transactions: &Vec<models::Transaction>) -> HashMap<(i32, u32), f64> {
    let mut count: HashMap<(i32, u32), f64> = HashMap::new();
    let monthly_totals = transactions.iter().fold(&mut count, |acc, transaction| {
        *acc.entry((transaction.date.year(), transaction.date.month()))
            .or_insert(0.0) += transaction.amount;
        acc // Return the updated accumulator for the next iteration
    });
    monthly_totals.clone()
}

fn _get_transactions_by_category(transactions: &Vec<models::Transaction>) -> HashMap<String, f64> {
    let mut count: HashMap<String, f64> = HashMap::new();
    let category_totals = transactions.iter().fold(&mut count, |acc, transaction| {
        match &transaction.group {
            Some(category) => *acc.entry(category.to_string()).or_insert(0.0) += transaction.amount,
            None => (),
        }
        acc // Return the updated accumulator for the next iteration
    });
    category_totals.clone()
}

#[tokio::main]
async fn main() -> Result<()> {
    let df = read_folder("../data".into()).expect("PathError");
    println!("{df}");

    let transactions = convert_df(df);

    let _ = app::init_app(transactions).await;

    Ok(())
}
