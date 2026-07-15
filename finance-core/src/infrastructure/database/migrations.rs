use rusqlite::Connection;

use crate::errors::FinanceError;

/// Embedded migration files, applied in order.
const MIGRATIONS: &[(&str, &str)] = &[
    (
        "0001_initial",
        include_str!("../../../migrations/0001_initial.sql"),
    ),
    (
        "0002_raw_imports_source_links",
        include_str!("../../../migrations/0002_raw_imports_source_links.sql"),
    ),
    (
        "0003_parser_lineage",
        include_str!("../../../migrations/0003_parser_lineage.sql"),
    ),
    (
        "0004_relationship_type",
        include_str!("../../../migrations/0004_relationship_type.sql"),
    ),
];

pub fn run_migrations(conn: &Connection) -> Result<(), FinanceError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    let applied: Vec<String> = {
        let mut stmt = conn.prepare("SELECT version FROM schema_migrations")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    for (version, sql) in MIGRATIONS {
        if applied.contains(&version.to_string()) {
            continue;
        }

        conn.execute_batch(sql)?;
        conn.execute(
            "INSERT INTO schema_migrations (version) VALUES (?1)",
            [version],
        )?;
    }

    Ok(())
}
