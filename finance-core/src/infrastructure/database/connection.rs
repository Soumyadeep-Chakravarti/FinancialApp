use std::path::Path;

use rusqlite::Connection;

use crate::errors::FinanceError;

use super::migrations::run_migrations;

pub struct Database {
    connection: Connection,
}

impl Database {
    /// Opens (or creates) the SQLite database and runs pending migrations.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, FinanceError> {
        let connection = Connection::open(path)?;

        // Enable WAL mode for better concurrent read performance.
        connection.execute_batch("PRAGMA journal_mode=WAL;")?;
        // Enable foreign keys.
        connection.execute_batch("PRAGMA foreign_keys=ON;")?;

        run_migrations(&connection)?;

        Ok(Self { connection })
    }

    /// Returns the underlying SQLite connection.
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// Returns a mutable SQLite connection.
    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.connection
    }
}
