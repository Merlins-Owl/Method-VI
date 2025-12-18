use anyhow::{Context, Result};
use rusqlite::Connection;

/// SQL statements for creating all tables
pub const SQL_CREATE_TABLES: &[&str] = &[
    // runs table
    r#"
    CREATE TABLE IF NOT EXISTS runs (
        id TEXT PRIMARY KEY,
        intent_anchor_hash TEXT NOT NULL,
        created_at DATETIME NOT NULL,
        completed_at DATETIME,
        final_ci REAL,
        final_ev REAL,
        status TEXT
    )
    "#,
    // artifacts table
    r#"
    CREATE TABLE IF NOT EXISTS artifacts (
        id TEXT PRIMARY KEY,
        run_id TEXT NOT NULL,
        type TEXT NOT NULL,
        step_origin INTEGER NOT NULL,
        hash TEXT NOT NULL,
        is_immutable INTEGER DEFAULT 0,
        content_path TEXT,
        created_at DATETIME NOT NULL,
        parent_hash TEXT,
        FOREIGN KEY (run_id) REFERENCES runs(id)
    )
    "#,
    // spine_edges table
    r#"
    CREATE TABLE IF NOT EXISTS spine_edges (
        source_id TEXT NOT NULL,
        target_id TEXT NOT NULL,
        edge_type TEXT NOT NULL,
        created_at DATETIME NOT NULL,
        PRIMARY KEY (source_id, target_id),
        FOREIGN KEY (source_id) REFERENCES artifacts(id),
        FOREIGN KEY (target_id) REFERENCES artifacts(id)
    )
    "#,
    // patterns table
    r#"
    CREATE TABLE IF NOT EXISTS patterns (
        id TEXT PRIMARY KEY,
        intent_category TEXT NOT NULL,
        ci_achievement REAL,
        ev_stability REAL,
        architecture_pattern TEXT,
        analysis_pattern TEXT,
        synthesis_pattern TEXT,
        structure_pattern TEXT,
        validation_pattern TEXT,
        applicability TEXT,
        vitality_freshness REAL DEFAULT 1.0,
        vitality_relevance REAL DEFAULT 1.0,
        application_count INTEGER DEFAULT 0,
        success_count INTEGER DEFAULT 0,
        created_at DATETIME NOT NULL,
        last_applied DATETIME,
        source_run_id TEXT,
        is_starter INTEGER DEFAULT 0,
        FOREIGN KEY (source_run_id) REFERENCES runs(id)
    )
    "#,
    // ledger_entries table
    r#"
    CREATE TABLE IF NOT EXISTS ledger_entries (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        run_id TEXT NOT NULL,
        entry_type TEXT NOT NULL,
        step INTEGER,
        role TEXT,
        payload TEXT,
        prior_hash TEXT,
        hash TEXT NOT NULL,
        created_at DATETIME NOT NULL,
        FOREIGN KEY (run_id) REFERENCES runs(id)
    )
    "#,
    // persistent_flaws table
    r#"
    CREATE TABLE IF NOT EXISTS persistent_flaws (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        flaw_description TEXT NOT NULL,
        occurrence_count INTEGER DEFAULT 1,
        first_seen DATETIME NOT NULL,
        last_seen DATETIME NOT NULL,
        affected_runs TEXT,
        resolution_status TEXT,
        policy_ticket TEXT
    )
    "#,
];

/// SQL statements for creating all indexes
pub const SQL_CREATE_INDEXES: &[&str] = &[
    "CREATE INDEX IF NOT EXISTS idx_patterns_category ON patterns(intent_category)",
    "CREATE INDEX IF NOT EXISTS idx_patterns_vitality ON patterns(vitality_freshness, vitality_relevance)",
    "CREATE INDEX IF NOT EXISTS idx_artifacts_run ON artifacts(run_id)",
    "CREATE INDEX IF NOT EXISTS idx_ledger_run ON ledger_entries(run_id)",
];

/// Creates all tables and indexes in the database
pub fn create_schema(conn: &Connection) -> Result<()> {
    // Create all tables
    for sql in SQL_CREATE_TABLES {
        conn.execute(sql, [])
            .context("Failed to create table")?;
    }

    // Create all indexes
    for sql in SQL_CREATE_INDEXES {
        conn.execute(sql, [])
            .context("Failed to create index")?;
    }

    // Create schema version table for future migrations
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at DATETIME NOT NULL
        )
        "#,
        [],
    )
    .context("Failed to create schema_version table")?;

    // Record initial schema version
    conn.execute(
        "INSERT OR IGNORE INTO schema_version (version, applied_at) VALUES (1, datetime('now'))",
        [],
    )
    .context("Failed to insert schema version")?;

    Ok(())
}

/// Gets the current schema version
pub fn get_schema_version(conn: &Connection) -> Result<i32> {
    let version: i32 = conn
        .query_row(
            "SELECT MAX(version) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(version)
}
