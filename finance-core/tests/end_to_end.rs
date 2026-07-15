use finance_core::application::ingestion::enrich::EnrichStage;
use finance_core::application::ingestion::normalize::NormalizeStage;
use finance_core::application::ingestion::stage::PipelineStage;
use finance_core::application::ingestion::validate::ValidateStage;
use finance_core::domain::{
    category::Category,
    raw_import::RawImportRecord,
    relationship_type::RelationshipType,
    source::ImportSource,
    source_link::SourceLink,
    transaction::Transaction,
};
use finance_core::infrastructure::{
    database::connection::Database,
    import::{pipeline::ImportPipeline, sms::generic::GenericUpiParser},
    repository::{
        raw_import::RawImportRepository, source_link::SourceLinkRepository,
        transaction::TransactionRepository,
    },
};

/// Run the full typed pipeline: Parse -> Normalize -> Validate -> Enrich -> Transaction
fn ingest(
    candidate: finance_core::domain::parsed_candidate::ParsedCandidate,
) -> Transaction {
    let normalized = NormalizeStage.run(candidate).unwrap();
    let validated = ValidateStage.run(normalized).unwrap();
    let enriched = EnrichStage::default().run(validated).unwrap();
    Transaction::new(enriched)
}

#[test]
fn lineage_integrity() {
    // 1. Setup
    let db = Database::open(":memory:").expect("failed to create in-memory database");
    let raw_repo = RawImportRepository::new(db.connection());
    let tx_repo = TransactionRepository::new(db.connection());
    let link_repo = SourceLinkRepository::new(db.connection());

    let mut pipeline = ImportPipeline::new();
    pipeline.register(Box::new(GenericUpiParser));

    // 2. Raw SMS input
    let sms = "UPI: Rs.500.00 sent to SWIGGY UPI Ref 123456789012";
    let original_payload = sms.as_bytes().to_vec();

    // 3. Create raw import record and persist it
    let mut record = RawImportRecord::from_text(ImportSource::Sms, sms)
        .with_metadata("phone_number", "+919876543210");
    let original_checksum = record.checksum;

    raw_repo
        .insert(&mut record)
        .expect("failed to insert raw import");
    let raw_import_id = record.id.expect("raw import should have id");

    // 4. Parse
    let candidate = pipeline.parse(&record).expect("failed to parse SMS");
    let parser_info = candidate.parser.clone();

    // 5. Full pipeline: Normalize -> Validate -> Enrich -> Transaction
    let tx = ingest(candidate);
    let tx_id = tx.id;

    // 6. Store transaction
    tx_repo.insert(&tx).expect("failed to insert transaction");

    // 7. Create source link with parser lineage
    let link = SourceLink::new(tx_id, raw_import_id, RelationshipType::ParsedFrom, parser_info);
    link_repo
        .insert(&link)
        .expect("failed to insert source link");

    // 8. Verify lineage integrity

    // raw_imports row exists with correct data
    let stored_raw = raw_repo
        .find_by_id(raw_import_id)
        .unwrap()
        .expect("raw import not found");
    assert_eq!(stored_raw.id, Some(raw_import_id));
    assert_eq!(stored_raw.checksum, original_checksum);
    assert_eq!(stored_raw.payload, original_payload);
    assert_eq!(stored_raw.source, ImportSource::Sms);
    assert_eq!(
        stored_raw.metadata.get("phone_number").unwrap(),
        "+919876543210"
    );

    // transaction row exists with correct data
    let stored_tx = tx_repo
        .find_by_id(tx_id)
        .unwrap()
        .expect("transaction not found");
    assert_eq!(stored_tx.id, tx_id);
    assert_eq!(
        stored_tx.amount.amount(),
        rust_decimal::Decimal::new(50000, 2)
    );
    assert_eq!(stored_tx.merchant.name, "SWIGGY");
    assert_eq!(stored_tx.category, Category::Food);
    assert_eq!(
        stored_tx.reference,
        Some("123456789012".to_string())
    );

    // source_link connects them with parser info and relationship
    let links = link_repo.find_by_transaction(tx_id).unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].transaction_id, tx_id);
    assert_eq!(links[0].raw_import_id, raw_import_id);
    assert_eq!(links[0].relationship, RelationshipType::ParsedFrom);
    assert_eq!(links[0].parser.name, "GenericUpiParser");
    assert_eq!(links[0].parser.version, 100);

    // reverse lookup works
    let links = link_repo.find_by_raw_import(raw_import_id).unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].transaction_id, tx_id);

    // parser lineage lookup works
    let links = link_repo
        .find_by_parser("GenericUpiParser", 100)
        .unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].transaction_id, tx_id);

    // relationship type lookup works
    let links = link_repo
        .find_by_relationship(RelationshipType::ParsedFrom)
        .unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].transaction_id, tx_id);

    // checksum lookup works
    let found = raw_repo.find_by_checksum(&original_checksum).unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, Some(raw_import_id));
}

#[test]
fn lineage_multiple_transactions_one_raw_import() {
    let db = Database::open(":memory:").expect("failed to create in-memory database");
    let raw_repo = RawImportRepository::new(db.connection());
    let tx_repo = TransactionRepository::new(db.connection());
    let link_repo = SourceLinkRepository::new(db.connection());

    // A batch SMS import (multiple messages concatenated)
    let batch =
        "UPI: Rs.500.00 sent to SWIGGY UPI Ref 111\nUPI: Rs.300.00 sent to ZOMATO UPI Ref 222";
    let mut record = RawImportRecord::from_text(ImportSource::Sms, batch);
    raw_repo
        .insert(&mut record)
        .expect("failed to insert raw import");
    let raw_import_id = record.id.unwrap();

    // Parse each line separately, but they share the same raw import
    let mut pipeline = ImportPipeline::new();
    pipeline.register(Box::new(GenericUpiParser));

    for line in batch.lines() {
        let single_record = RawImportRecord::from_text(ImportSource::Sms, line);
        let candidate = pipeline.parse(&single_record).expect("failed to parse");
        let parser_info = candidate.parser.clone();
        let tx = ingest(candidate);
        tx_repo.insert(&tx).expect("failed to insert");
        link_repo
            .insert(&SourceLink::new(
                tx.id,
                raw_import_id,
                RelationshipType::ParsedFrom,
                parser_info,
            ))
            .expect("failed to link");
    }

    // Both transactions link to same raw import
    let links = link_repo.find_by_raw_import(raw_import_id).unwrap();
    assert_eq!(links.len(), 2);
}

#[test]
fn raw_import_checksum_dedup_check() {
    let db = Database::open(":memory:").expect("failed to create in-memory database");
    let raw_repo = RawImportRepository::new(db.connection());

    let sms = "UPI: Rs.500.00 sent to SWIGGY UPI Ref 123456789012";
    let mut record1 = RawImportRecord::from_text(ImportSource::Sms, sms);
    let checksum = record1.checksum;

    raw_repo.insert(&mut record1).expect("failed to insert");

    // Same checksum already exists
    assert!(raw_repo.exists_checksum(&checksum).unwrap());

    // A completely different message that was never imported
    let never_imported =
        RawImportRecord::from_text(ImportSource::Sms, "UPI: Rs.999.00 sent to UNKNOWN UPI Ref 999");
    assert!(!raw_repo.exists_checksum(&never_imported.checksum).unwrap());
}

#[test]
fn source_link_relationship_type_roundtrip() {
    use finance_core::domain::parser_info::ParserInfo;

    let db = Database::open(":memory:").expect("failed to create in-memory database");
    let link_repo = SourceLinkRepository::new(db.connection());
    let raw_repo = RawImportRepository::new(db.connection());
    let tx_repo = TransactionRepository::new(db.connection());

    // Create a transaction via the typed pipeline
    let parsed = finance_core::domain::parsed_candidate::ParsedCandidate::new(ParserInfo::new("Test", 1))
        .with_amount(finance_core::domain::money::Money::new(rust_decimal::Decimal::new(1000, 2)))
        .with_merchant("Zomato")
        .with_kind(finance_core::domain::transaction_type::TransactionType::Debit);
    let tx = ingest(parsed);
    tx_repo.insert(&tx).unwrap();

    // Create 6 raw imports (one for each relationship type)
    let mut raw_ids = Vec::new();
    for i in 0..6 {
        let mut record = RawImportRecord::from_text(ImportSource::Sms, &format!("raw {i}"));
        raw_repo.insert(&mut record).unwrap();
        raw_ids.push(record.id.unwrap());
    }

    // Insert with each relationship type
    let parser = ParserInfo::new("TestParser", 1);
    let relationships = [
        RelationshipType::ParsedFrom,
        RelationshipType::DerivedFrom,
        RelationshipType::CorrectedBy,
        RelationshipType::SplitFrom,
        RelationshipType::MergedFrom,
        RelationshipType::ImportedWith,
    ];

    for (i, &rel) in relationships.iter().enumerate() {
        let link = SourceLink::new(tx.id, raw_ids[i], rel, parser.clone());
        link_repo.insert(&link).unwrap();
    }

    // All 6 links exist for this transaction
    let links = link_repo.find_by_transaction(tx.id).unwrap();
    assert_eq!(links.len(), 6);

    // Lookup by each relationship type
    for &rel in &relationships {
        let links = link_repo.find_by_relationship(rel).unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].relationship, rel);
        assert_eq!(links[0].transaction_id, tx.id);
    }
}

#[test]
fn typed_pipeline_rejects_invalid_data() {
    use finance_core::domain::money::Money;
    use finance_core::domain::parser_info::ParserInfo;

    // Zero amount should be rejected by ValidateStage
    let parsed = finance_core::domain::parsed_candidate::ParsedCandidate::new(ParserInfo::new("Test", 1))
        .with_amount(Money::new(rust_decimal::Decimal::ZERO))
        .with_merchant("Swiggy");

    let normalized = NormalizeStage.run(parsed).unwrap();
    let result = ValidateStage.run(normalized);
    assert!(result.is_err());
}

#[test]
fn transaction_can_only_be_created_from_enriched_candidate() {
    use finance_core::domain::parser_info::ParserInfo;

    let parsed = finance_core::domain::parsed_candidate::ParsedCandidate::new(ParserInfo::new("Test", 1))
        .with_amount(finance_core::domain::money::Money::new(rust_decimal::Decimal::new(500, 2)))
        .with_merchant("Swiggy");

    // Must go through full pipeline
    let normalized = NormalizeStage.run(parsed).unwrap();
    let validated = ValidateStage.run(normalized).unwrap();
    let enriched = EnrichStage::default().run(validated).unwrap();

    // Only EnrichedCandidate -> Transaction is allowed
    let tx = Transaction::new(enriched);
    assert_eq!(tx.amount.amount(), rust_decimal::Decimal::new(500, 2));
    assert_eq!(tx.merchant.name, "SWIGGY");
    assert_eq!(tx.category, Category::Food);
}
