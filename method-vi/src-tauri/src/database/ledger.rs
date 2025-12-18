// TODO: Implement CRUD operations for ledger_entries table
// Reference runs.rs for implementation pattern

use super::models::LedgerEntry;
use anyhow::Result;
use rusqlite::Connection;

pub fn create_ledger_entry(_conn: &Connection, _entry: &LedgerEntry) -> Result<()> {
    todo!("Implement create_ledger_entry")
}

pub fn get_ledger_entry(_conn: &Connection, _id: i64) -> Result<Option<LedgerEntry>> {
    todo!("Implement get_ledger_entry")
}

pub fn list_ledger_entries_by_run(_conn: &Connection, _run_id: &str) -> Result<Vec<LedgerEntry>> {
    todo!("Implement list_ledger_entries_by_run")
}
