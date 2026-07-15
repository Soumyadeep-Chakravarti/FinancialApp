ALTER TABLE source_links ADD COLUMN parser_name TEXT NOT NULL DEFAULT '';
ALTER TABLE source_links ADD COLUMN parser_version INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_source_links_parser ON source_links(parser_name, parser_version);
