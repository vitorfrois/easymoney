use crate::labeling::CategoryMap;
use crate::models::Category;
use crate::models::{NewTransaction, Transaction};
use dirs_next::data_dir;
use rusqlite::{Connection, Result};
use std::fs;
use std::str::FromStr;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let database_dir = data_dir().unwrap().join("easymoney");
        let database_path = database_dir.join("database.db");

        if !database_dir.exists() {
            fs::create_dir_all(&database_dir).ok();
        }

        let conn = Connection::open(database_path)?;
        let database = Database { conn };
        database.initialize_database().ok();
        Ok(database)
    }

    pub fn initialize_database(&self) -> Result<()> {
        let _ = self.create_transactions();
        let _ = self.create_categories();
        Ok(())
    }

    fn create_transactions(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                date    DATE,
                title   TEXT,
                amount  REAL NOT NULL,
                kind    TEXT NOT NULL,
                tgroup  TEXT NULL,
                UNIQUE(date, title)
                )",
            (),
        )?;
        Ok(())
    }

    pub fn insert_transactions(&self, transactions: Vec<NewTransaction>) -> Result<()> {
        for transaction in transactions {
            self.insert_transaction(transaction);
        }
        Ok(())
    }

    pub fn insert_transaction(&self, transaction: NewTransaction) {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO transactions (date, title, amount, kind) VALUES (?1, ?2, ?3, ?4)",
                (
                    &transaction.date,
                    &transaction.title,
                    &transaction.amount,
                    &transaction.kind,
                ),
            )
            .ok();
    }

    pub fn get_transactions(&self) -> Result<Vec<Transaction>> {
        let mut statement = self
            .conn
            .prepare("SELECT rowid, date, title, amount, kind, tgroup FROM transactions")?;

        let rows = statement.query_map([], |row| {
            Ok(Transaction {
                id: row.get(0)?,
                date: row.get(1)?,
                title: row.get(2)?,
                amount: row.get(3)?,
                kind: row.get(4)?,
                group: row.get(5)?,
            })
        })?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(row?);
        }
        Ok(transactions)
    }

    fn create_categories(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS categories (
                title       TEXT,
                category    TEXT, 
                PRIMARY KEY (title, category)
            )",
            (),
        )?;
        Ok(())
    }

    pub fn insert_categories(&self, categories: CategoryMap) -> Result<()> {
        for (title, category) in categories.map {
            self.insert_category(title, category);
        }
        Ok(())
    }

    pub fn insert_category(&self, title: String, category: Category) {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO categories (title, category) VALUES (?1, ?2)",
                (&title, &category),
            )
            .ok();
    }

    pub fn get_categories(&self) -> Result<CategoryMap> {
        let mut statement = self
            .conn
            .prepare("SELECT title, category FROM categories")?;

        let map = statement
            .query_map([], |row| {
                let category_str: String = row.get(1)?;
                Ok((row.get(0)?, Category::from_str(&category_str)?))
            })?
            .collect::<Result<Vec<(String, Category)>, _>>()?;

        Ok(CategoryMap { map })
    }
}
