# Financial App

A privacy-focused Android application that helps users understand and manage their finances by automatically tracking transactions from UPI SMS notifications and organizing them into budgets, reminders, and financial insights.

The application is designed to work **entirely offline**, ensuring that sensitive financial information never leaves the user's device.

## Features

### Transaction Tracking

* Automatically detect UPI transactions from SMS notifications.
* Categorize expenses and income.
* View transaction history.
* Manual transaction entry.

### Budget Planning

* Create monthly or custom budgets.
* Track spending against budget limits.
* Receive warnings when approaching budget limits.
* Spending analytics.

### Payment Management

* Payment reminders.
* Recurring payment schedules.
* Auto-payment shortcuts (where supported by Android and banking apps).

## Planned Features

* Spending trends and statistics.
* Category-based analytics.
* Goal-based savings tracker.
* CSV/Excel import and export.
* Local encrypted database.
* Search and filtering.
* Multiple account support.

## Privacy

* **Offline-first**.
* No cloud synchronization.
* No advertisements.
* User data remains on the device.
* Optional local encryption.

## Tech Stack

### Android

* Kotlin
* Jetpack Compose

### Core Logic

* Rust

  * Business logic
  * Budget calculations
  * Transaction processing
  * Database layer
  * Encryption
  * Import/Export

### Database

* SQLite (via rusqlite)

### Communication

* JNI (Kotlin ↔ Rust)

---

## Future Scope

* Desktop version using the same Rust core.
* iOS support.
* Optional encrypted backup.
* Financial reports.
* AI-powered spending insights (fully local).

