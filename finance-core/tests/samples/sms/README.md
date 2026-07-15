# SMS Sample Corpus

Real anonymized SMS messages used to build and test bank parsers.

## Structure

```
sms/
├── raw/              # Unprocessed messages from phone (never committed)
│   └── sbi/
│
├── sbi/              # Anonymized, classified regression corpus
│   ├── valid.txt     # Transaction messages (debit/credit)
│   ├── edge_cases.txt
│   └── invalid.txt   # Non-transactions (OTP, promo, balance alerts)
│
├── hdfc/             # pending
├── icici/            # pending
├── axis/             # pending
├── kotak/            # pending
├── generic/          # Generic UPI fallback — 20 samples
├── gpay/
├── paytm/
└── phonepe/
```

## Workflow

1. Paste raw SMS into `raw/<bank>/`
2. I anonymize and classify each message
3. Classified messages go into `<bank>/valid.txt`, `edge_cases.txt`, or `invalid.txt`
4. Raw originals stay in `raw/` as reference

## Classification

- **valid** — transaction message (debit/credit/refund)
- **edge_case** — unusual format, masked account, mixed language
- **invalid** — OTP, promo, balance alert, failed transaction, account notification

## Target per bank

- 10–15 debit messages
- 10–15 credit messages
- 5–10 edge cases
- 5 invalid/non-transaction messages

## Anonymization rules

1. Account numbers → `XX1234`
2. UPI refs → keep format, randomize numbers
3. Names → placeholders
4. Balances → optionally randomized (preserve format)
5. Keep bank names, amounts, timestamps accurate

Example:

```
SBI: INR 1,500.00 debited from A/c XX1234 on 15/01/26.
Ref: UPI/1234567890/ABC@YBL. Avl Bal: INR 25,000.00
```

## How parsers use these

Each parser runs against its bank's `valid.txt` in unit tests.
The golden test harness (`tests/harness.rs`) loads samples and asserts parsing success.
