CREATE TABLE IF NOT EXISTS transactions (
    id          TEXT PRIMARY KEY,
    amount      TEXT NOT NULL,
    merchant    TEXT NOT NULL,
    category    TEXT NOT NULL,
    kind        TEXT NOT NULL,
    timestamp   TEXT NOT NULL,
    reference   TEXT UNIQUE,
    notes       TEXT
);
