pub mod models;
pub mod schema;
pub mod runs;
pub mod artifacts;
pub mod patterns;
pub mod ledger;
pub mod spine;
pub mod flaws;

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

/// Global database connection
static DB_CONNECTION: Mutex<Option<Connection>> = Mutex::new(None);

/// Gets the path to the database file using Tauri's app_data_dir
pub fn get_db_path(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .context("Failed to get app data directory")?;

    // Ensure the directory exists
    std::fs::create_dir_all(&app_data_dir)
        .context("Failed to create app data directory")?;

    Ok(app_data_dir.join("method-vi.db"))
}

/// Checks if the database file exists
pub fn database_exists(app_handle: &tauri::AppHandle) -> Result<bool> {
    let db_path = get_db_path(app_handle)?;
    Ok(db_path.exists())
}

/// Initializes the database, creating tables and indexes if needed
pub fn init_database(app_handle: &tauri::AppHandle) -> Result<()> {
    let db_path = get_db_path(app_handle)?;

    println!("Initializing database at: {:?}", db_path);

    let conn = Connection::open(&db_path)
        .context("Failed to open database connection")?;

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])
        .context("Failed to enable foreign keys")?;

    // Create schema (tables and indexes)
    schema::create_schema(&conn)
        .context("Failed to create database schema")?;

    println!("Database schema created successfully");

    // Store the connection globally
    let mut db_conn = DB_CONNECTION.lock().unwrap();
    *db_conn = Some(conn);

    Ok(())
}

/// Gets a reference to the database connection
///
/// Note: This returns a new connection each time. For production use,
/// consider using a connection pool (r2d2) for better concurrency.
pub fn get_connection(app_handle: &tauri::AppHandle) -> Result<Connection> {
    let db_path = get_db_path(app_handle)?;

    let conn = Connection::open(&db_path)
        .context("Failed to open database connection")?;

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])
        .context("Failed to enable foreign keys")?;

    Ok(conn)
}
