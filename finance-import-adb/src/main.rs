use std::path::PathBuf;

use clap::{Parser, Subcommand};
use finance_core::domain::raw_import::RawImportRecord;
use finance_core::domain::source::ImportSource;
use finance_core::infrastructure::database::connection::Database;
use finance_core::infrastructure::repository::raw_import::RawImportRepository;
use finance_import_adb::adb::AdbSmsImporter;
use finance_import_adb::Importer;

#[derive(Parser)]
#[command(name = "finance-import-adb")]
#[command(about = "Import financial data from Android phones via ADB")]
struct Cli {
    /// Database file path
    #[arg(short, long, default_value = "finance.db")]
    db: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Import SMS messages from phone via ADB
    Sms {
        /// Only import SMS from addresses matching this pattern
        #[arg(short, long)]
        filter: Option<String>,

        /// Maximum number of messages to import
        #[arg(short, long)]
        limit: Option<usize>,

        /// Dry run — show what would be imported without saving
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Sms { filter, limit, dry_run } => {
            // Check ADB first
            AdbSmsImporter::check_adb()?;

            let mut importer = AdbSmsImporter::new();
            if let Some(f) = filter {
                importer = importer.with_address_filter(f);
            }
            if let Some(l) = limit {
                importer = importer.with_limit(l);
            }

            println!("Reading SMS from phone...");
            let records = importer.import()?;
            println!("Found {} SMS messages", records.len());

            if dry_run {
                println!("\nDry run — would import:");
                for (i, record) in records.iter().enumerate() {
                    println!("  {}. {:?}", i + 1, record.as_text());
                }
                return Ok(());
            }

            // Open database and insert records
            let db = Database::open(&cli.db)?;
            let repo = RawImportRepository::new(db.connection());

            let mut inserted = 0;
            let mut skipped = 0;

            for record in &records {
                let mut raw = RawImportRecord::from_text(
                    ImportSource::Sms,
                    record.as_text().unwrap_or(""),
                );

                // Copy metadata from ADB record
                for (k, v) in &record.metadata {
                    raw = raw.with_metadata(k, v);
                }

                match repo.insert_if_new(&mut raw)? {
                    true => inserted += 1,
                    false => skipped += 1,
                }
            }

            println!("\nImported: {} new, {} skipped (duplicates)", inserted, skipped);
        }
    }

    Ok(())
}
