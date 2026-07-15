/// Identifies which parser produced a parsed candidate.
///
/// Used for lineage tracking: when a parser is upgraded or a bug is found,
/// you can answer "which records were parsed by SbiParser v1?" and reprocess them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserInfo {
    pub name: &'static str,
    pub version: u16,
}

impl ParserInfo {
    pub const fn new(name: &'static str, version: u16) -> Self {
        Self { name, version }
    }

    /// Construct from an owned String (e.g., from database).
    /// Leaks the string for 'static lifetime — acceptable for parser names
    /// which are small and few.
    pub fn new_static(name: &str, version: u16) -> Self {
        let owned: String = name.to_string();
        Self {
            name: Box::leak(owned.into_boxed_str()),
            version,
        }
    }
}

impl std::fmt::Display for ParserInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} v{}", self.name, self.version)
    }
}
