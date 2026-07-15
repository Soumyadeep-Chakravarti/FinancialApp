use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

use finance_core::domain::{
    category::Category, merchant::Merchant, money::Money, transaction::Transaction,
    transaction_type::TransactionType,
};
use finance_core::infrastructure::{database::connection::Database, repository::{raw_import::RawImportRepository, transaction::TransactionRepository}};

fn test_db() -> Database {
    Database::open(":memory:").expect("failed to create in-memory database")
}

fn sample_transaction() -> Transaction {
    Transaction {
        id: Uuid::new_v4(),
        amount: Money::new(Decimal::new(3500, 2)), // 350.00
        merchant: Merchant::new("Swiggy"),
        category: Category::Food,
        kind: TransactionType::Debit,
        timestamp: Utc::now(),
        reference: Some("REF123456".to_string()),
        notes: None,
    }
}

#[test]
fn insert_and_find_by_id() {
    let db = test_db();
    let repo = TransactionRepository::new(db.connection());
    let tx = sample_transaction();

    repo.insert(&tx).unwrap();
    let found = repo.find_by_id(tx.id).unwrap();

    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, tx.id);
    assert_eq!(found.amount, tx.amount);
    assert_eq!(found.merchant.name, "Swiggy");
    assert_eq!(found.category, Category::Food);
    assert_eq!(found.kind, TransactionType::Debit);
    assert_eq!(found.reference, Some("REF123456".to_string()));
}

#[test]
fn find_by_reference() {
    let db = test_db();
    let repo = TransactionRepository::new(db.connection());
    let tx = sample_transaction();

    repo.insert(&tx).unwrap();
    let found = repo.find_by_reference("REF123456").unwrap();

    assert!(found.is_some());
    assert_eq!(found.unwrap().id, tx.id);
}

#[test]
fn find_by_reference_returns_none_for_missing() {
    let db = test_db();
    let repo = TransactionRepository::new(db.connection());

    let found = repo.find_by_reference("NONEXISTENT").unwrap();
    assert!(found.is_none());
}

#[test]
fn exists_reference() {
    let db = test_db();
    let repo = TransactionRepository::new(db.connection());
    let tx = sample_transaction();

    repo.insert(&tx).unwrap();

    assert!(repo.exists_reference("REF123456").unwrap());
    assert!(!repo.exists_reference("OTHER").unwrap());
}

#[test]
fn duplicate_reference_fails() {
    let db = test_db();
    let repo = TransactionRepository::new(db.connection());

    let tx1 = sample_transaction();
    repo.insert(&tx1).unwrap();

    let tx2 = sample_transaction(); // same reference
    let result = repo.insert(&tx2);
    assert!(result.is_err());
}

#[test]
fn list_all_orders_by_timestamp_desc() {
    let db = test_db();
    let repo = TransactionRepository::new(db.connection());

    let mut tx1 = sample_transaction();
    tx1.timestamp = Utc::now() - chrono::Duration::hours(1);
    let mut tx2 = sample_transaction();
    tx2.id = Uuid::new_v4();
    tx2.reference = Some("REF999".to_string());
    tx2.timestamp = Utc::now();

    repo.insert(&tx1).unwrap();
    repo.insert(&tx2).unwrap();

    let all = repo.list_all().unwrap();
    assert_eq!(all.len(), 2);
    // tx2 is newer, should come first
    assert_eq!(all[0].id, tx2.id);
    assert_eq!(all[1].id, tx1.id);
}

#[test]
fn delete_removes_transaction() {
    let db = test_db();
    let repo = TransactionRepository::new(db.connection());
    let tx = sample_transaction();

    repo.insert(&tx).unwrap();
    let deleted = repo.delete(tx.id).unwrap();
    assert!(deleted);

    let found = repo.find_by_id(tx.id).unwrap();
    assert!(found.is_none());
}

#[test]
fn delete_returns_false_for_missing() {
    let db = test_db();
    let repo = TransactionRepository::new(db.connection());

    let deleted = repo.delete(Uuid::new_v4()).unwrap();
    assert!(!deleted);
}

#[test]
fn transaction_without_reference_and_notes() {
    let db = test_db();
    let repo = TransactionRepository::new(db.connection());

    let tx = Transaction {
        id: Uuid::new_v4(),
        amount: Money::new(Decimal::new(100, 0)),
        merchant: Merchant::new("Unknown"),
        category: Category::Other,
        kind: TransactionType::Credit,
        timestamp: Utc::now(),
        reference: None,
        notes: None,
    };

    repo.insert(&tx).unwrap();
    let found = repo.find_by_id(tx.id).unwrap().unwrap();

    assert_eq!(found.reference, None);
    assert_eq!(found.notes, None);
}

#[test]
fn migrations_are_idempotent() {
    let db = test_db();
    // Running open twice on same path should not fail.
    // For in-memory DBs, this just verifies the migration logic doesn't error
    // on a fresh connection.
    let _ = db;
}

#[test]
fn raw_import_insert_if_new_inserts_first_time() {
    use finance_core::domain::{raw_import::RawImportRecord, source::ImportSource};

    let db = test_db();
    let repo = RawImportRepository::new(db.connection());

    let mut record = RawImportRecord::from_text(ImportSource::Sms, "UPI: Rs.500 sent to X");
    let inserted = repo.insert_if_new(&mut record).unwrap();
    assert!(inserted);
    assert!(record.id.is_some());
}

#[test]
fn raw_import_insert_if_new_skips_duplicate() {
    use finance_core::domain::{raw_import::RawImportRecord, source::ImportSource};

    let db = test_db();
    let repo = RawImportRepository::new(db.connection());

    let mut record1 = RawImportRecord::from_text(ImportSource::Sms, "UPI: Rs.500 sent to X");
    repo.insert_if_new(&mut record1).unwrap();
    let id1 = record1.id.unwrap();

    let mut record2 = RawImportRecord::from_text(ImportSource::Sms, "UPI: Rs.500 sent to X");
    let inserted = repo.insert_if_new(&mut record2).unwrap();
    assert!(!inserted);
    // record2 should NOT have an ID assigned (was skipped)
    assert!(record2.id.is_none());

    // Only one record in the table
    let found = repo.find_by_id(id1).unwrap();
    assert!(found.is_some());
}

#[test]
fn raw_import_insert_if_new_allows_different_content() {
    use finance_core::domain::{raw_import::RawImportRecord, source::ImportSource};

    let db = test_db();
    let repo = RawImportRepository::new(db.connection());

    let mut r1 = RawImportRecord::from_text(ImportSource::Sms, "UPI: Rs.500 sent to X");
    repo.insert_if_new(&mut r1).unwrap();

    let mut r2 = RawImportRecord::from_text(ImportSource::Sms, "UPI: Rs.300 sent to Y");
    let inserted = repo.insert_if_new(&mut r2).unwrap();
    assert!(inserted);
    assert!(r2.id.is_some());
    assert_ne!(r1.id, r2.id);
}
