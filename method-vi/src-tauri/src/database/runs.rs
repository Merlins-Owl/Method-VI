use super::models::Run;
use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};

/// Creates a new run in the database
pub fn create_run(conn: &Connection, run: &Run) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO runs (id, intent_anchor_hash, created_at, completed_at, final_ci, final_ev, status)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        rusqlite::params![
            run.id,
            run.intent_anchor_hash,
            run.created_at.to_rfc3339(),
            run.completed_at.as_ref().map(|dt| dt.to_rfc3339()),
            run.final_ci,
            run.final_ev,
            run.status,
        ],
    )
    .context("Failed to create run")?;

    Ok(())
}

/// Gets a run by ID
pub fn get_run(conn: &Connection, id: &str) -> Result<Option<Run>> {
    let mut stmt = conn
        .prepare("SELECT id, intent_anchor_hash, created_at, completed_at, final_ci, final_ev, status FROM runs WHERE id = ?1")
        .context("Failed to prepare query")?;

    let run = stmt
        .query_row([id], |row| {
            Ok(Run {
                id: row.get(0)?,
                intent_anchor_hash: row.get(1)?,
                created_at: row.get::<_, String>(2)?.parse().unwrap(),
                completed_at: row
                    .get::<_, Option<String>>(3)?
                    .and_then(|s| s.parse().ok()),
                final_ci: row.get(4)?,
                final_ev: row.get(5)?,
                status: row.get(6)?,
            })
        })
        .optional()
        .context("Failed to query run")?;

    Ok(run)
}

/// Updates an existing run
pub fn update_run(conn: &Connection, run: &Run) -> Result<()> {
    let rows_affected = conn
        .execute(
            r#"
            UPDATE runs
            SET intent_anchor_hash = ?2,
                created_at = ?3,
                completed_at = ?4,
                final_ci = ?5,
                final_ev = ?6,
                status = ?7
            WHERE id = ?1
            "#,
            rusqlite::params![
                run.id,
                run.intent_anchor_hash,
                run.created_at.to_rfc3339(),
                run.completed_at.as_ref().map(|dt| dt.to_rfc3339()),
                run.final_ci,
                run.final_ev,
                run.status,
            ],
        )
        .context("Failed to update run")?;

    if rows_affected == 0 {
        anyhow::bail!("Run with id '{}' not found", run.id);
    }

    Ok(())
}

/// Deletes a run by ID
pub fn delete_run(conn: &Connection, id: &str) -> Result<()> {
    let rows_affected = conn
        .execute("DELETE FROM runs WHERE id = ?1", [id])
        .context("Failed to delete run")?;

    if rows_affected == 0 {
        anyhow::bail!("Run with id '{}' not found", id);
    }

    Ok(())
}

/// Lists all runs
pub fn list_runs(conn: &Connection) -> Result<Vec<Run>> {
    let mut stmt = conn
        .prepare("SELECT id, intent_anchor_hash, created_at, completed_at, final_ci, final_ev, status FROM runs ORDER BY created_at DESC")
        .context("Failed to prepare query")?;

    let runs = stmt
        .query_map([], |row| {
            Ok(Run {
                id: row.get(0)?,
                intent_anchor_hash: row.get(1)?,
                created_at: row.get::<_, String>(2)?.parse().unwrap(),
                completed_at: row
                    .get::<_, Option<String>>(3)?
                    .and_then(|s| s.parse().ok()),
                final_ci: row.get(4)?,
                final_ev: row.get(5)?,
                status: row.get(6)?,
            })
        })
        .context("Failed to query runs")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect runs")?;

    Ok(runs)
}

/// Gets all active runs (status = 'active')
pub fn get_active_runs(conn: &Connection) -> Result<Vec<Run>> {
    let mut stmt = conn
        .prepare("SELECT id, intent_anchor_hash, created_at, completed_at, final_ci, final_ev, status FROM runs WHERE status = 'active' ORDER BY created_at DESC")
        .context("Failed to prepare query")?;

    let runs = stmt
        .query_map([], |row| {
            Ok(Run {
                id: row.get(0)?,
                intent_anchor_hash: row.get(1)?,
                created_at: row.get::<_, String>(2)?.parse().unwrap(),
                completed_at: row
                    .get::<_, Option<String>>(3)?
                    .and_then(|s| s.parse().ok()),
                final_ci: row.get(4)?,
                final_ev: row.get(5)?,
                status: row.get(6)?,
            })
        })
        .context("Failed to query active runs")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect runs")?;

    Ok(runs)
}

/// Gets all completed runs (status = 'completed')
pub fn get_completed_runs(conn: &Connection) -> Result<Vec<Run>> {
    let mut stmt = conn
        .prepare("SELECT id, intent_anchor_hash, created_at, completed_at, final_ci, final_ev, status FROM runs WHERE status = 'completed' ORDER BY completed_at DESC")
        .context("Failed to prepare query")?;

    let runs = stmt
        .query_map([], |row| {
            Ok(Run {
                id: row.get(0)?,
                intent_anchor_hash: row.get(1)?,
                created_at: row.get::<_, String>(2)?.parse().unwrap(),
                completed_at: row
                    .get::<_, Option<String>>(3)?
                    .and_then(|s| s.parse().ok()),
                final_ci: row.get(4)?,
                final_ev: row.get(5)?,
                status: row.get(6)?,
            })
        })
        .context("Failed to query completed runs")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect runs")?;

    Ok(runs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::schema;
    use chrono::Utc;
    use rusqlite::Connection;

    /// Helper function to create an in-memory test database
    fn setup_test_db() -> Result<Connection> {
        let conn = Connection::open_in_memory()
            .context("Failed to create in-memory database")?;

        conn.execute("PRAGMA foreign_keys = ON", [])
            .context("Failed to enable foreign keys")?;

        schema::create_schema(&conn)
            .context("Failed to create schema")?;

        Ok(conn)
    }

    #[test]
    fn test_database_initialization_and_crud() {
        println!("\n=== Starting Database Test ===\n");

        // Step 1: Initialize the database
        println!("Step 1: Initializing in-memory test database...");
        let conn = setup_test_db().expect("Failed to initialize test database");
        println!("✓ Database initialized successfully\n");

        // Step 2: Create a test run
        println!("Step 2: Creating test run...");
        let test_run = Run {
            id: "2025-12-17-Test-Run".to_string(),
            intent_anchor_hash: "abc123def456".to_string(),
            created_at: Utc::now(),
            completed_at: None,
            final_ci: None,
            final_ev: None,
            status: Some("active".to_string()),
        };

        create_run(&conn, &test_run).expect("Failed to create run");
        println!("✓ Test run created: {}", test_run.id);
        println!("  - Intent Hash: {}", test_run.intent_anchor_hash);
        println!("  - Status: {}\n", test_run.status.as_ref().unwrap());

        // Step 3: Read it back
        println!("Step 3: Reading test run back from database...");
        let retrieved_run = get_run(&conn, &test_run.id)
            .expect("Failed to get run")
            .expect("Run not found");

        println!("✓ Test run retrieved successfully");
        println!("  - ID: {}", retrieved_run.id);
        println!("  - Intent Hash: {}", retrieved_run.intent_anchor_hash);
        println!("  - Status: {}\n", retrieved_run.status.as_ref().unwrap());

        // Verify the data matches
        assert_eq!(retrieved_run.id, test_run.id);
        assert_eq!(retrieved_run.intent_anchor_hash, test_run.intent_anchor_hash);
        assert_eq!(retrieved_run.status, test_run.status);

        // Step 4: Test additional operations
        println!("Step 4: Testing additional CRUD operations...");

        // Test list_runs
        let all_runs = list_runs(&conn).expect("Failed to list runs");
        println!("✓ Listed all runs: {} found", all_runs.len());
        assert_eq!(all_runs.len(), 1);

        // Test get_active_runs
        let active_runs = get_active_runs(&conn).expect("Failed to get active runs");
        println!("✓ Listed active runs: {} found", active_runs.len());
        assert_eq!(active_runs.len(), 1);

        // Test update_run
        let mut updated_run = retrieved_run.clone();
        updated_run.status = Some("completed".to_string());
        updated_run.completed_at = Some(Utc::now());
        updated_run.final_ci = Some(0.95);
        updated_run.final_ev = Some(0.12);

        update_run(&conn, &updated_run).expect("Failed to update run");
        println!("✓ Updated run status to 'completed' with CI=0.95, EV=0.12");

        // Verify update
        let completed_run = get_run(&conn, &test_run.id)
            .expect("Failed to get run")
            .expect("Run not found");
        assert_eq!(completed_run.status, Some("completed".to_string()));
        assert_eq!(completed_run.final_ci, Some(0.95));
        println!("✓ Update verified\n");

        // Test get_completed_runs
        let completed_runs = get_completed_runs(&conn).expect("Failed to get completed runs");
        println!("✓ Listed completed runs: {} found", completed_runs.len());
        assert_eq!(completed_runs.len(), 1);

        // Test delete_run
        delete_run(&conn, &test_run.id).expect("Failed to delete run");
        println!("✓ Deleted test run");

        // Verify deletion
        let deleted_run = get_run(&conn, &test_run.id).expect("Failed to query run");
        assert!(deleted_run.is_none());
        println!("✓ Deletion verified\n");

        // Step 5: Success
        println!("=== 🎉 ALL TESTS PASSED SUCCESSFULLY! 🎉 ===\n");
        println!("Database Operations Verified:");
        println!("  ✓ Schema creation");
        println!("  ✓ CREATE run");
        println!("  ✓ READ run (get by ID)");
        println!("  ✓ UPDATE run");
        println!("  ✓ DELETE run");
        println!("  ✓ LIST runs (all, active, completed)");
        println!("\n=== Test Complete ===\n");
    }

    #[test]
    fn test_multiple_runs() {
        println!("\n=== Testing Multiple Runs ===\n");

        let conn = setup_test_db().expect("Failed to initialize test database");

        // Create multiple test runs
        for i in 1..=3 {
            let run = Run {
                id: format!("2025-12-17-Test-Run-{}", i),
                intent_anchor_hash: format!("hash{}", i),
                created_at: Utc::now(),
                completed_at: None,
                final_ci: None,
                final_ev: None,
                status: Some("active".to_string()),
            };
            create_run(&conn, &run).expect("Failed to create run");
            println!("✓ Created run: {}", run.id);
        }

        let all_runs = list_runs(&conn).expect("Failed to list runs");
        println!("\n✓ Total runs in database: {}", all_runs.len());
        assert_eq!(all_runs.len(), 3);

        println!("\n=== Multiple Runs Test Passed ===\n");
    }
}
