// TODO: Implement CRUD operations for artifacts table
// Reference runs.rs for implementation pattern

use super::models::Artifact;
use anyhow::Result;
use rusqlite::Connection;

pub fn create_artifact(_conn: &Connection, _artifact: &Artifact) -> Result<()> {
    todo!("Implement create_artifact")
}

pub fn get_artifact(_conn: &Connection, _id: &str) -> Result<Option<Artifact>> {
    todo!("Implement get_artifact")
}

pub fn list_artifacts_by_run(_conn: &Connection, _run_id: &str) -> Result<Vec<Artifact>> {
    todo!("Implement list_artifacts_by_run")
}
