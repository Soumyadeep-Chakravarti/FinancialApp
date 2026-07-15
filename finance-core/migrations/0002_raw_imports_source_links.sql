CREATE TABLE IF NOT EXISTS raw_imports (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    source      TEXT NOT NULL,
    mime_type   TEXT,
    payload     BLOB NOT NULL,
    checksum    BLOB NOT NULL UNIQUE,
    imported_at TEXT NOT NULL DEFAULT (datetime('now')),
    metadata    TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_raw_imports_checksum ON raw_imports(checksum);
CREATE INDEX IF NOT EXISTS idx_raw_imports_source ON raw_imports(source);

CREATE TABLE IF NOT EXISTS source_links (
    transaction_id TEXT NOT NULL,
    raw_import_id  INTEGER NOT NULL,
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (transaction_id, raw_import_id),
    FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE CASCADE,
    FOREIGN KEY (raw_import_id) REFERENCES raw_imports(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_source_links_transaction ON source_links(transaction_id);
CREATE INDEX IF NOT EXISTS idx_source_links_raw_import ON source_links(raw_import_id);
