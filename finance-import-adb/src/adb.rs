use std::process::Command;

use finance_core::domain::{raw_import::RawImportRecord, source::ImportSource};

use crate::{ImportError, Importer};

/// ADB SMS importer — reads SMS from an Android phone connected via USB.
///
/// Uses `adb shell content query` to read the SMS content provider.
/// No app installation required — works with USB debugging enabled.
pub struct AdbSmsImporter {
    /// Optional filter: only import SMS from addresses matching this pattern.
    /// If None, imports all SMS.
    pub address_filter: Option<String>,
    /// Maximum number of messages to import. None = no limit.
    pub limit: Option<usize>,
}

impl AdbSmsImporter {
    pub fn new() -> Self {
        Self {
            address_filter: None,
            limit: None,
        }
    }

    pub fn with_address_filter(mut self, filter: impl Into<String>) -> Self {
        self.address_filter = Some(filter.into());
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Check if ADB is available and a device is connected.
    pub fn check_adb() -> Result<(), ImportError> {
        // Check if adb exists
        let output = Command::new("adb").arg("version").output()?;
        if !output.status.success() {
            return Err(ImportError::AdbNotFound(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Check if device is connected
        let output = Command::new("adb")
            .args(["devices"])
            .output()?;
        if !output.status.success() {
            return Err(ImportError::NoDevice);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // First line is "List of devices attached", second line should have a device
        let lines: Vec<&str> = stdout.lines().collect();
        if lines.len() < 2 || lines[1].trim().is_empty() || lines[1].contains("offline") {
            return Err(ImportError::NoDevice);
        }

        Ok(())
    }

    /// Query SMS messages from the phone.
    fn query_sms(&self) -> Result<String, ImportError> {
        // Query SMS inbox: address (sender), body (message text), date (timestamp)
        let mut cmd = Command::new("adb");
        cmd.args([
            "shell",
            "content",
            "query",
            "--uri",
            "content://sms/inbox",
            "--projection",
            "address:body:date",
        ]);

        let output = cmd.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("permission") || stderr.contains("SecurityException") {
                return Err(ImportError::PermissionDenied(stderr.to_string()));
            }
            return Err(ImportError::QueryFailed(stderr.to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl Default for AdbSmsImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Importer for AdbSmsImporter {
    fn import(&mut self) -> Result<Vec<RawImportRecord>, ImportError> {
        Self::check_adb()?;

        let raw_output = self.query_sms()?;
        let records = parse_adb_sms_output(&raw_output);

        let records: Vec<RawImportRecord> = records
            .into_iter()
            .filter(|r| {
                // Apply address filter if set
                if let Some(ref filter) = self.address_filter {
                    if let Some(addr) = r.metadata.get("address") {
                        if !addr.contains(filter) {
                            return false;
                        }
                    }
                }
                true
            })
            .take(self.limit.unwrap_or(usize::MAX))
            .collect();

        Ok(records)
    }
}

/// Parse ADB content query output into RawImportRecords.
///
/// ADB output format:
/// ```text
/// Row: 0 _id=123, address=+919876543210, body=Your OTP is 123456, date=1234567890000
/// Row: 1 _id=124, address=BANK-SMS, body=INR 500 debited..., date=1234567891000
/// ```
fn parse_adb_sms_output(output: &str) -> Vec<RawImportRecord> {
    let mut records = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with("Row:") {
            continue;
        }

        // Extract fields from the row
        if let Some(body) = extract_field(line, "body") {
            if body.is_empty() {
                continue;
            }

            let mut record = RawImportRecord::from_text(ImportSource::Sms, &body);

            // Store metadata for downstream use
            if let Some(addr) = extract_field(line, "address") {
                record = record.with_metadata("address", addr);
            }
            if let Some(date) = extract_field(line, "date") {
                record = record.with_metadata("date", date);
            }

            records.push(record);
        }
    }

    records
}

/// Extract a field value from an ADB content query row.
///
/// Format: `field_name=value` where value ends at next `, field_name=` or end of line.
fn extract_field(row: &str, field_name: &str) -> Option<String> {
    let pattern = format!("{}=", field_name);
    let start = row.find(&pattern)? + pattern.len();

    // Find the next field: look for ", next_field="
    let rest = &row[start..];
    let end = rest.find(", ").unwrap_or(rest.len());

    let value = rest[..end].trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_field_basic() {
        let row = "Row: 0 _id=123, address=+919876543210, body=Hello, date=1234567890000";
        assert_eq!(extract_field(row, "address"), Some("+919876543210".to_string()));
        assert_eq!(extract_field(row, "body"), Some("Hello".to_string()));
        assert_eq!(extract_field(row, "date"), Some("1234567890000".to_string()));
    }

    #[test]
    fn extract_field_with_commas_in_body() {
        let row = "Row: 0 _id=123, address=BANK, body=INR 1,500.00 debited, date=123";
        // Body ends at next field separator
        assert_eq!(extract_field(row, "body"), Some("INR 1,500.00 debited".to_string()));
    }

    #[test]
    fn extract_field_missing() {
        let row = "Row: 0 _id=123, address=+919876543210";
        assert_eq!(extract_field(row, "body"), None);
    }

    #[test]
    fn parse_empty_output() {
        let records = parse_adb_sms_output("");
        assert!(records.is_empty());
    }

    #[test]
    fn parse_multiple_rows() {
        let output = "\
Row: 0 _id=1, address=BANK1, body=OTP is 123456, date=1000
Row: 1 _id=2, address=BANK2, body=INR 500 debited, date=2000";
        let records = parse_adb_sms_output(output);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].as_text(), Some("OTP is 123456"));
        assert_eq!(records[1].as_text(), Some("INR 500 debited"));
        assert_eq!(
            records[0].metadata.get("address"),
            Some(&"BANK1".to_string())
        );
        assert_eq!(
            records[1].metadata.get("address"),
            Some(&"BANK2".to_string())
        );
    }

    #[test]
    fn parse_skips_empty_and_non_row_lines() {
        let output = "\
Some header line
Row: 0 _id=1, address=BANK, body=Valid SMS, date=1000

Row: 1 _id=2, address=BANK, body=, date=2000
Another line";
        let records = parse_adb_sms_output(output);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].as_text(), Some("Valid SMS"));
    }
}
