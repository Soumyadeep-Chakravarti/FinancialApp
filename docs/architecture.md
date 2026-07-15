# Architecture

## Vision

A personal financial data platform. Ingests financial events from any source, normalizes them through an ETL pipeline, and stores canonical transactions for analytics.

## Data Flow

```
                        Raw Sources
──────────────────────────────────────────────────────

   SMS            Notifications       Bank Statements
   Manual Entry   Future Bank APIs    Future UPI APIs
   Email Receipts Credit Card CSVs    Salary Slips

                    │
                    ▼
              Source Adapters
         (parse raw source format)
                    │
                    ▼
           RawImportRecord
         (immutable landing zone)
                    │
                    ▼
              Import Pipeline
         (SourceParser registry)
                    │
                    ▼
            ParsedCandidate
         (partial, source-agnostic)
                    │
                    ▼
           Ingestion Pipeline
──────────────────────────────────────────────────────
   Normalize → Dedup → Enrich → Validate → Load
                    │
                    ▼
           Canonical Transaction
                    │
                    ▼
              SQLite Database
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
    Budgets     Reports    Analytics
    Forecasts   Cashflow   Investment Tracking
```

## Zones (Data Warehouse Pattern)

```
┌─────────────────────────────────────────────┐
│  RAW ZONE                                  │
│  RawImportRecord (immutable)               │
│  - original payload exactly as received    │
│  - source type + mime type                 │
│  - metadata (filename, phone, etc.)        │
│  - never modified after import             │
├─────────────────────────────────────────────┤
│  CLEAN ZONE                                │
│  Canonical Transaction                     │
│  - normalized fields                       │
│  - category                                │
│  - source links (many-to-many)             │
│  - derived from ingestion pipeline         │
├─────────────────────────────────────────────┤
│  ANALYTICS ZONE                            │
│  Aggregates, trends, forecasts             │
│  Derived from clean zone only              │
└─────────────────────────────────────────────┘
```

**Key rule:** Never throw away raw data. If the parser improves, re-run ETL over raw imports.

## Source Types

```rust
enum Source {
    Sms,
    Notification,
    BankStatement,
    Manual,
    Api,
    Email,
    Import,
}
```

Every imported record knows its origin. The source link table tracks which raw imports contributed to each canonical transaction.

## Ingestion Pipeline

```
RawImportRecord
        │
        ▼
   ImportPipeline (SourceParser registry)
        │
        ▼
   ParsedCandidate (source-agnostic)
        │
        ▼
   Normalize        (amount, timestamp, merchant standardization)
        │
        ▼
   Deduplicate      (match on reference, amount+merchant+timestamp)
        │
        ▼
   Enrich           (category inference, merchant resolution)
        │
        ▼
   Validate         (required fields, sanity checks)
        │
        ▼
   Load             (write to SQLite, create source links)
```

## Duplicate Resolution

Duplicates are **merged**, not deleted. The source link table provides traceability:

```
Transaction (id: abc)
├── SourceLink → RawImport (SMS)
└── SourceLink → RawImport (Notification)
```

Resolution strategy:
1. **Exact match on UTR/Reference** → automatic merge
2. **Match on amount + merchant + date** → merge
3. **Match on amount + date only** → merge (lower confidence)
4. **Conflicting amounts** → keep both, flag for review
5. **Ambiguous** → keep both, flag for review

## Crate Structure

```
finance-core/
├── domain/                         Pure business types
│   ├── transaction.rs              CanonicalTransaction
│   ├── money.rs                    Money (Decimal wrapper)
│   ├── category.rs                 Category enum + Display/FromStr
│   ├── transaction_type.rs         Credit/Debit enum + Display/FromStr
│   ├── merchant.rs                 Merchant struct (name + upi_id)
│   ├── budget.rs                   Budget struct
│   ├── budget_period.rs            Daily/Weekly/Monthly/Yearly
│   ├── source.rs                   Source enum
│   ├── source_link.rs              SourceLink (many-to-many)
│   ├── raw_import.rs               RawImportRecord (immutable)
│   ├── parsed_candidate.rs         ParsedCandidate (parser output)
│   └── reminder.rs                 (placeholder)
│
├── application/                    Orchestration logic
│   └── ingestion/
│       └── mod.rs                  (stages extracted here as they grow)
│
├── infrastructure/                 I/O and external systems
│   ├── database/
│   │   ├── connection.rs           Database::open() with migrations
│   │   └── migrations.rs           Embedded SQL, version tracking
│   ├── repository/
│   │   └── transaction.rs          CRUD operations
│   └── import/
│       ├── parser.rs               SourceParser trait + ParseError
│       ├── pipeline.rs             ImportPipeline (parser registry)
│       ├── sms/                    (future: paytm.rs, gpay.rs, phonepe.rs)
│       ├── notifications/
│       ├── csv/
│       └── statements/
│
└── errors.rs                       FinanceError enum
```

## Golden Tests

```
tests/samples/
├── shared/                         Cross-provider test cases
│   ├── non_upi.txt
│   ├── bank_transfer.txt
│   ├── amount_variants.txt
│   └── date_variants.txt
└── sms/
    ├── paytm/
    │   └── valid.txt
    ├── gpay/
    │   └── valid.txt
    └── phonepe/
        └── valid.txt
```

Each provider folder can contain `valid/`, `invalid/`, `edge_cases/`. When a user reports a weird SMS, drop it into `edge_cases/` to permanently expand the regression suite.

## Design Decisions

### Domain is pure Rust
No serde, no database, no I/O. Domain types are plain structs and enums.
Serialization lives in the infrastructure layer.

### RawImportRecord is immutable
Never modified after import. If parsers improve, re-run ETL over raw imports.
This is the "Raw Zone" — the landing zone that never changes.

### SourceParser is source-agnostic
One trait for SMS, notifications, CSV, bank statements, anything.
Adding a new source means implementing one adapter.

### ParsedCandidate is source-agnostic
Parser extracts raw fields only. It doesn't know where the data came from.
The relationship to the original raw import is tracked via SourceLink.

### SourceLink provides lineage
Many-to-many between transactions and raw imports. One transaction can
originate from SMS + notification + CSV. Analytics reads clean zone only.

### Category inference in Enrich stage
Parsers don't infer categories. The Enrich stage applies business rules
after normalization and dedup.

### Money has no currency field
The app is INR-only for UPI. Adding Currency solves no current problem.

### Migrations tracked via schema_migrations table
Each migration is embedded at compile time and applied in order. The
`schema_migrations` table prevents re-application.
