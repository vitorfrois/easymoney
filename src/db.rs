use crate::models::Transaction;
use dirs_next::data_dir;
use rusqlite::Connection;
use rusqlite::Result;
use std::fs;

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
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                date    DATE,
                title   TEXT,
                amount  REAL NOT NULL,
                tgroup  TEXT NULL,
                PRIMARY KEY (date, title)
                )",
            (),
        )?;
        Ok(())
    }

    pub fn insert_transactions(&self, transactions: Vec<Transaction>) -> Result<()> {
        for transaction in transactions {
            self.insert_transaction(transaction);
        }
        Ok(())
    }

    pub fn insert_transaction(&self, transaction: Transaction) {
        self.conn
            .execute(
                "INSERT INTO transactions (date, title, amount, tgroup) VALUES (?1, ?2, ?3, ?4)",
                (
                    &transaction.date,
                    &transaction.title,
                    &transaction.amount,
                    &transaction.group,
                ),
            )
            .ok();
    }

    pub fn get_transactions(&self) -> Result<Vec<Transaction>> {
        let mut statement = self
            .conn
            .prepare("SELECT date, title, amount, tgroup FROM transactions")?;

        let rows = statement.query_map([], |row| {
            Ok(Transaction {
                date: row.get(0)?,
                title: row.get(1)?,
                amount: row.get(2)?,
                group: None,
                // kind: row.get(4)?,
            })
        })?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(row?);
        }
        Ok(transactions)
    }
}
