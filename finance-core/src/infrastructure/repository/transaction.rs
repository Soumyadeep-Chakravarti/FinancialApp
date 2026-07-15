use std::str::FromStr;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::domain::{
    category::Category, merchant::Merchant, money::Money, transaction::Transaction,
    transaction_type::TransactionType,
};
use crate::errors::FinanceError;

pub struct TransactionRepository<'a> {
    conn: &'a Connection,
}

impl<'a> TransactionRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn insert(&self, transaction: &Transaction) -> Result<(), FinanceError> {
        self.conn.execute(
            "INSERT INTO transactions (id, amount, merchant, category, kind, timestamp, reference, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                transaction.id.to_string(),
                transaction.amount.amount().to_string(),
                transaction.merchant.name,
                transaction.category.to_string(),
                transaction.kind.to_string(),
                transaction.timestamp.to_rfc3339(),
                transaction.reference,
                transaction.notes,
            ],
        )?;
        Ok(())
    }

    pub fn find_by_id(&self, id: Uuid) -> Result<Option<Transaction>, FinanceError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, amount, merchant, category, kind, timestamp, reference, notes
             FROM transactions WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_transaction(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn find_by_reference(&self, reference: &str) -> Result<Option<Transaction>, FinanceError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, amount, merchant, category, kind, timestamp, reference, notes
             FROM transactions WHERE reference = ?1",
        )?;

        let mut rows = stmt.query(params![reference])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_transaction(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn exists_reference(&self, reference: &str) -> Result<bool, FinanceError> {
        let mut stmt = self
            .conn
            .prepare("SELECT 1 FROM transactions WHERE reference = ?1 LIMIT 1")?;
        let mut rows = stmt.query(params![reference])?;
        Ok(rows.next()?.is_some())
    }

    pub fn list_all(&self) -> Result<Vec<Transaction>, FinanceError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, amount, merchant, category, kind, timestamp, reference, notes
             FROM transactions ORDER BY timestamp DESC",
        )?;

        let mut rows = stmt.query([])?;
        let mut transactions = Vec::new();
        while let Some(row) = rows.next()? {
            transactions.push(row_to_transaction(row)?);
        }
        Ok(transactions)
    }

    pub fn delete(&self, id: Uuid) -> Result<bool, FinanceError> {
        let rows = self
            .conn
            .execute("DELETE FROM transactions WHERE id = ?1", params![id.to_string()])?;
        Ok(rows > 0)
    }
}

fn row_to_transaction(row: &rusqlite::Row<'_>) -> Result<Transaction, FinanceError> {
    let id_str: String = row.get(0).map_err(|e| FinanceError::CorruptData(e.to_string()))?;
    let amount_str: String = row.get(1).map_err(|e| FinanceError::CorruptData(e.to_string()))?;
    let merchant_name: String = row.get(2).map_err(|e| FinanceError::CorruptData(e.to_string()))?;
    let category_str: String = row.get(3).map_err(|e| FinanceError::CorruptData(e.to_string()))?;
    let kind_str: String = row.get(4).map_err(|e| FinanceError::CorruptData(e.to_string()))?;
    let timestamp_str: String = row.get(5).map_err(|e| FinanceError::CorruptData(e.to_string()))?;
    let reference: Option<String> = row.get(6).map_err(|e| FinanceError::CorruptData(e.to_string()))?;
    let notes: Option<String> = row.get(7).map_err(|e| FinanceError::CorruptData(e.to_string()))?;

    let id = Uuid::parse_str(&id_str)
        .map_err(|e| FinanceError::CorruptData(format!("invalid UUID: {e}")))?;
    let amount = Money::new(
        rust_decimal::Decimal::from_str(&amount_str)
            .map_err(|e| FinanceError::CorruptData(format!("invalid amount: {e}")))?,
    );
    let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
        .map_err(|e| FinanceError::CorruptData(format!("invalid timestamp: {e}")))?
        .with_timezone(&Utc);
    let category = Category::from_str(&category_str)
        .map_err(|e| FinanceError::CorruptData(format!("invalid category: {e}")))?;
    let kind = TransactionType::from_str(&kind_str)
        .map_err(|e| FinanceError::CorruptData(format!("invalid kind: {e}")))?;

    Ok(Transaction {
        id,
        amount,
        merchant: Merchant::new(merchant_name),
        category,
        kind,
        timestamp,
        reference,
        notes,
    })
}
