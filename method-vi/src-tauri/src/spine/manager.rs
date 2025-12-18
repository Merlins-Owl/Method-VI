use super::types::*;
use std::collections::{HashMap, HashSet, VecDeque};

/// Coherence Spine Manager
///
/// Manages the directed acyclic graph (DAG) of artifacts and their dependencies.
/// Enforces artifact dependency integrity and enables circuit breaker validation
/// for Phase 2 modes.
///
/// The spine maintains:
/// - Artifact nodes (deliverables/checkpoints)
/// - Dependency edges (relationships between artifacts)
/// - Critical Path tracking (Intent_Anchor → Charter → Baseline → Core_Thesis)
/// - Integrity validation (no breaks, orphans, or cycles)
pub struct SpineManager {
    /// Map of artifact ID to artifact
    artifacts: HashMap<String, Artifact>,

    /// List of all dependencies
    dependencies: Vec<Dependency>,
}

impl SpineManager {
    /// Creates a new empty SpineManager
    pub fn new() -> Self {
        SpineManager {
            artifacts: HashMap::new(),
            dependencies: Vec::new(),
        }
    }

    /// Adds an artifact to the spine
    ///
    /// # Arguments
    /// * `artifact` - The artifact to add
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(String)` if artifact ID already exists
    ///
    /// # Example
    /// ```
    /// use method_vi::spine::{SpineManager, Artifact, ArtifactType};
    /// use chrono::Utc;
    ///
    /// let mut manager = SpineManager::new();
    /// let artifact = Artifact {
    ///     id: "intent-001".to_string(),
    ///     artifact_type: ArtifactType::Intent_Anchor,
    ///     step_origin: 0,
    ///     hash: "abc123".to_string(),
    ///     is_immutable: true,
    ///     created_at: Utc::now(),
    ///     parent_hash: None,
    /// };
    /// manager.add_artifact(artifact);
    /// ```
    pub fn add_artifact(&mut self, artifact: Artifact) -> Result<(), String> {
        if self.artifacts.contains_key(&artifact.id) {
            return Err(format!("Artifact with id '{}' already exists", artifact.id));
        }
        self.artifacts.insert(artifact.id.clone(), artifact);
        Ok(())
    }

    /// Adds a dependency edge to the spine
    ///
    /// # Arguments
    /// * `dependency` - The dependency to add
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(String)` if source or target artifact doesn't exist
    ///
    /// # Example
    /// ```
    /// use method_vi::spine::{SpineManager, Dependency, DependencyType};
    /// use chrono::Utc;
    ///
    /// let mut manager = SpineManager::new();
    /// // ... add artifacts first ...
    /// let dependency = Dependency {
    ///     source_id: "charter-001".to_string(),
    ///     target_id: "intent-001".to_string(),
    ///     dependency_type: DependencyType::DerivedFrom,
    ///     created_at: Utc::now(),
    /// };
    /// // manager.add_dependency(dependency);
    /// ```
    pub fn add_dependency(&mut self, dependency: Dependency) -> Result<(), String> {
        // Verify both artifacts exist
        if !self.artifacts.contains_key(&dependency.source_id) {
            return Err(format!(
                "Source artifact '{}' does not exist",
                dependency.source_id
            ));
        }
        if !self.artifacts.contains_key(&dependency.target_id) {
            return Err(format!(
                "Target artifact '{}' does not exist",
                dependency.target_id
            ));
        }

        // Check if adding this dependency would create a cycle
        if self.would_create_cycle(&dependency.source_id, &dependency.target_id) {
            return Err(format!(
                "Adding dependency from '{}' to '{}' would create a cycle",
                dependency.source_id, dependency.target_id
            ));
        }

        self.dependencies.push(dependency);
        Ok(())
    }

    /// Gets all dependencies for an artifact (artifacts this one depends on)
    ///
    /// Returns a list of artifacts that the given artifact depends on,
    /// along with the type of dependency.
    ///
    /// # Arguments
    /// * `artifact_id` - ID of the artifact to query
    ///
    /// # Returns
    /// List of DependencyInfo (empty if no dependencies or artifact doesn't exist)
    ///
    /// # Example
    /// ```
    /// let deps = manager.get_dependencies("charter-001");
    /// // Returns: [{id: "intent-001", type: DerivedFrom}]
    /// ```
    pub fn get_dependencies(&self, artifact_id: &str) -> Vec<DependencyInfo> {
        self.dependencies
            .iter()
            .filter(|dep| dep.source_id == artifact_id)
            .map(|dep| DependencyInfo {
                id: dep.target_id.clone(),
                dependency_type: dep.dependency_type.clone(),
            })
            .collect()
    }

    /// Gets all dependents of an artifact (artifacts that depend on this one)
    ///
    /// Returns a list of artifacts that depend on the given artifact.
    ///
    /// # Arguments
    /// * `artifact_id` - ID of the artifact to query
    ///
    /// # Returns
    /// List of artifact IDs that depend on this artifact
    ///
    /// # Example
    /// ```
    /// let dependents = manager.get_dependents("intent-001");
    /// // Returns: ["charter-001", "baseline-001"]
    /// ```
    pub fn get_dependents(&self, artifact_id: &str) -> Vec<String> {
        self.dependencies
            .iter()
            .filter(|dep| dep.target_id == artifact_id)
            .map(|dep| dep.source_id.clone())
            .collect()
    }

    /// Checks if an artifact is on the Critical Path
    ///
    /// Critical Path: Intent_Anchor → Charter → Baseline → Core_Thesis
    /// Artifacts on the Critical Path are immutable and cannot be targeted
    /// by Surgical Mode in Phase 2.
    ///
    /// # Arguments
    /// * `artifact_id` - ID of the artifact to check
    ///
    /// # Returns
    /// * `true` if artifact is on Critical Path
    /// * `false` if not on Critical Path or artifact doesn't exist
    ///
    /// # Example
    /// ```
    /// assert!(manager.is_on_critical_path("intent-001"));  // true
    /// assert!(manager.is_on_critical_path("charter-001")); // true
    /// assert!(!manager.is_on_critical_path("section-001")); // false
    /// ```
    pub fn is_on_critical_path(&self, artifact_id: &str) -> bool {
        self.artifacts
            .get(artifact_id)
            .map(|artifact| artifact.artifact_type.is_on_critical_path())
            .unwrap_or(false)
    }

    /// Validates the integrity of the entire spine
    ///
    /// Checks for:
    /// - Broken edges (dependencies pointing to non-existent artifacts)
    /// - Orphaned artifacts (no path to Intent_Anchor)
    /// - Cycles (should never exist in a valid DAG)
    ///
    /// # Returns
    /// SpineIntegrityReport with validation results
    ///
    /// # Example
    /// ```
    /// let report = manager.validate_spine_integrity();
    /// if !report.valid {
    ///     println!("Spine has issues:");
    ///     println!("  Breaks: {:?}", report.breaks);
    ///     println!("  Orphans: {:?}", report.orphans);
    ///     println!("  Cycles: {:?}", report.cycles);
    /// }
    /// ```
    pub fn validate_spine_integrity(&self) -> SpineIntegrityReport {
        let mut breaks = Vec::new();
        let mut orphans = Vec::new();
        let cycles = self.detect_cycles();

        // Check for broken edges
        for dep in &self.dependencies {
            if !self.artifacts.contains_key(&dep.source_id) {
                breaks.push(BrokenEdge {
                    from: dep.source_id.clone(),
                    to: dep.target_id.clone(),
                    dependency_type: dep.dependency_type.clone(),
                });
            }
            if !self.artifacts.contains_key(&dep.target_id) {
                breaks.push(BrokenEdge {
                    from: dep.source_id.clone(),
                    to: dep.target_id.clone(),
                    dependency_type: dep.dependency_type.clone(),
                });
            }
        }

        // Check for orphaned artifacts (no path to Intent_Anchor)
        // Find all Intent_Anchors (roots)
        let roots: Vec<String> = self
            .artifacts
            .values()
            .filter(|a| matches!(a.artifact_type, ArtifactType::Intent_Anchor))
            .map(|a| a.id.clone())
            .collect();

        // If no roots, all artifacts are orphans (except empty spine)
        if roots.is_empty() {
            if !self.artifacts.is_empty() {
                orphans = self.artifacts.keys().cloned().collect();
            }
        } else {
            // Find all reachable artifacts from roots
            let mut reachable = HashSet::new();
            for root in &roots {
                self.mark_reachable(root, &mut reachable);
            }

            // Any artifact not reachable is an orphan
            for artifact_id in self.artifacts.keys() {
                if !reachable.contains(artifact_id) {
                    orphans.push(artifact_id.clone());
                }
            }
        }

        let valid = breaks.is_empty() && orphans.is_empty() && cycles.is_empty();

        SpineIntegrityReport {
            valid,
            breaks,
            orphans,
            cycles,
        }
    }

    /// Gets the lineage of an artifact (path to Intent_Anchor)
    ///
    /// Traces the dependency chain from the given artifact back to the
    /// Intent_Anchor (root). Returns the path in order from the artifact
    /// to the root.
    ///
    /// # Arguments
    /// * `artifact_id` - ID of the artifact to trace
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - Path from artifact to Intent_Anchor
    /// * `Err(String)` - If artifact doesn't exist or has broken lineage
    ///
    /// # Example
    /// ```
    /// let lineage = manager.get_lineage("charter-001");
    /// // Returns: ["charter-001", "intent-001"]
    /// ```
    pub fn get_lineage(&self, artifact_id: &str) -> Result<Vec<String>, String> {
        if !self.artifacts.contains_key(artifact_id) {
            return Err(format!("Artifact '{}' does not exist", artifact_id));
        }

        let mut lineage = Vec::new();
        let mut current_id = artifact_id.to_string();
        let mut visited = HashSet::new();

        loop {
            // Check for cycles in lineage
            if visited.contains(&current_id) {
                return Err(format!("Cycle detected in lineage at '{}'", current_id));
            }
            visited.insert(current_id.clone());

            lineage.push(current_id.clone());

            // Check if we've reached an Intent_Anchor
            if let Some(artifact) = self.artifacts.get(&current_id) {
                if matches!(artifact.artifact_type, ArtifactType::Intent_Anchor) {
                    return Ok(lineage);
                }
            }

            // Find parent via DerivedFrom dependency
            let parent_deps: Vec<_> = self
                .dependencies
                .iter()
                .filter(|dep| {
                    dep.source_id == current_id
                        && matches!(dep.dependency_type, DependencyType::DerivedFrom)
                })
                .collect();

            if parent_deps.is_empty() {
                // No parent found and not at Intent_Anchor - broken lineage
                return Err(format!(
                    "Broken lineage: '{}' has no DerivedFrom parent and is not Intent_Anchor",
                    current_id
                ));
            }

            // Follow the first DerivedFrom dependency
            current_id = parent_deps[0].target_id.clone();
        }
    }

    // === Helper Methods ===

    /// Checks if adding a dependency would create a cycle
    fn would_create_cycle(&self, from: &str, to: &str) -> bool {
        // If we add edge from->to, we create a cycle if there's already a path from to->from
        self.has_path(to, from)
    }

    /// Checks if there's a path from source to target
    fn has_path(&self, from: &str, to: &str) -> bool {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(from.to_string());

        while let Some(current) = queue.pop_front() {
            if current == to {
                return true;
            }

            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            // Add all targets of dependencies from current
            for dep in &self.dependencies {
                if dep.source_id == current {
                    queue.push_back(dep.target_id.clone());
                }
            }
        }

        false
    }

    /// Marks all artifacts reachable from the given root
    fn mark_reachable(&self, root: &str, reachable: &mut HashSet<String>) {
        if reachable.contains(root) {
            return;
        }
        reachable.insert(root.to_string());

        // Get all dependents (artifacts that depend on this one)
        for dep in &self.dependencies {
            if dep.target_id == root {
                self.mark_reachable(&dep.source_id, reachable);
            }
        }
    }

    /// Detects cycles in the spine using DFS
    fn detect_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for artifact_id in self.artifacts.keys() {
            if !visited.contains(artifact_id) {
                self.dfs_cycle_detection(
                    artifact_id,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    /// DFS helper for cycle detection
    fn dfs_cycle_detection(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        // Visit all neighbors (targets of dependencies from this node)
        for dep in &self.dependencies {
            if dep.source_id == node {
                let neighbor = &dep.target_id;

                if rec_stack.contains(neighbor) {
                    // Found a cycle - extract it from path
                    if let Some(cycle_start) = path.iter().position(|id| id == neighbor) {
                        let cycle = path[cycle_start..].to_vec();
                        cycles.push(cycle);
                    }
                } else if !visited.contains(neighbor) {
                    self.dfs_cycle_detection(neighbor, visited, rec_stack, path, cycles);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
    }

    /// Gets an artifact by ID (for testing/debugging)
    pub fn get_artifact(&self, id: &str) -> Option<&Artifact> {
        self.artifacts.get(id)
    }
}

impl Default for SpineManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Helper to create a test artifact
    fn create_artifact(
        id: &str,
        artifact_type: ArtifactType,
        step_origin: i32,
        parent_hash: Option<String>,
    ) -> Artifact {
        let is_immutable = matches!(
            artifact_type,
            ArtifactType::Intent_Anchor
                | ArtifactType::Charter
                | ArtifactType::Baseline
                | ArtifactType::Core_Thesis
        );

        Artifact {
            id: id.to_string(),
            artifact_type,
            step_origin,
            hash: format!("hash-{}", id),
            is_immutable,
            created_at: Utc::now(),
            parent_hash,
        }
    }

    /// Helper to create a test dependency
    fn create_dependency(
        source_id: &str,
        target_id: &str,
        dependency_type: DependencyType,
    ) -> Dependency {
        Dependency {
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            dependency_type,
            created_at: Utc::now(),
        }
    }

    // ===== TC-CS-001: Get Dependencies Tests =====

    #[test]
    fn tc_cs_001_a_root_has_no_dependencies() {
        println!("\n=== TC-CS-001-A: Root has no dependencies ===");
        let mut manager = SpineManager::new();

        let intent = create_artifact("art-001", ArtifactType::Intent_Anchor, 0, None);
        manager.add_artifact(intent).unwrap();

        let deps = manager.get_dependencies("art-001");
        println!("Dependencies for art-001: {:?}", deps);
        assert_eq!(deps.len(), 0, "Root should have no dependencies");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_cs_001_b_single_dependency() {
        println!("\n=== TC-CS-001-B: Single dependency ===");
        let mut manager = SpineManager::new();

        let intent = create_artifact("art-001", ArtifactType::Intent_Anchor, 0, None);
        let charter = create_artifact("art-002", ArtifactType::Charter, 1, Some("hash-art-001".to_string()));

        manager.add_artifact(intent).unwrap();
        manager.add_artifact(charter).unwrap();

        let dep = create_dependency("art-002", "art-001", DependencyType::DerivedFrom);
        manager.add_dependency(dep).unwrap();

        let deps = manager.get_dependencies("art-002");
        println!("Dependencies for art-002: {:?}", deps);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].id, "art-001");
        assert!(matches!(deps[0].dependency_type, DependencyType::DerivedFrom));
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_cs_001_c_multiple_dependencies() {
        println!("\n=== TC-CS-001-C: Multiple dependencies ===");
        let mut manager = SpineManager::new();

        let intent = create_artifact("art-001", ArtifactType::Intent_Anchor, 0, None);
        let charter = create_artifact("art-002", ArtifactType::Charter, 1, Some("hash-art-001".to_string()));
        let section = create_artifact("art-003", ArtifactType::Section, 2, Some("hash-art-002".to_string()));

        manager.add_artifact(intent).unwrap();
        manager.add_artifact(charter).unwrap();
        manager.add_artifact(section).unwrap();

        manager
            .add_dependency(create_dependency("art-003", "art-002", DependencyType::DerivedFrom))
            .unwrap();
        manager
            .add_dependency(create_dependency("art-003", "art-001", DependencyType::ConstrainedBy))
            .unwrap();

        let deps = manager.get_dependencies("art-003");
        println!("Dependencies for art-003: {:?}", deps);
        assert_eq!(deps.len(), 2);

        let dep_ids: Vec<String> = deps.iter().map(|d| d.id.clone()).collect();
        assert!(dep_ids.contains(&"art-001".to_string()));
        assert!(dep_ids.contains(&"art-002".to_string()));
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_cs_001_d_nonexistent_artifact() {
        println!("\n=== TC-CS-001-D: Handle missing artifact ===");
        let manager = SpineManager::new();

        let deps = manager.get_dependencies("nonexistent");
        println!("Dependencies for nonexistent: {:?}", deps);
        assert_eq!(deps.len(), 0, "Nonexistent artifact should return empty list");
        println!("✓ Test passed\n");
    }

    // ===== TC-CS-002: Get Dependents Tests =====

    #[test]
    fn tc_cs_002_a_root_has_multiple_dependents() {
        println!("\n=== TC-CS-002-A: Root has multiple dependents ===");
        let mut manager = SpineManager::new();

        let intent = create_artifact("art-001", ArtifactType::Intent_Anchor, 0, None);
        let charter = create_artifact("art-002", ArtifactType::Charter, 1, Some("hash-art-001".to_string()));
        let section = create_artifact("art-003", ArtifactType::Section, 2, Some("hash-art-001".to_string()));

        manager.add_artifact(intent).unwrap();
        manager.add_artifact(charter).unwrap();
        manager.add_artifact(section).unwrap();

        manager
            .add_dependency(create_dependency("art-002", "art-001", DependencyType::DerivedFrom))
            .unwrap();
        manager
            .add_dependency(create_dependency("art-003", "art-001", DependencyType::DerivedFrom))
            .unwrap();

        let dependents = manager.get_dependents("art-001");
        println!("Dependents of art-001: {:?}", dependents);
        assert_eq!(dependents.len(), 2);
        assert!(dependents.contains(&"art-002".to_string()));
        assert!(dependents.contains(&"art-003".to_string()));
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_cs_002_b_single_dependent() {
        println!("\n=== TC-CS-002-B: Single dependent ===");
        let mut manager = SpineManager::new();

        let intent = create_artifact("art-001", ArtifactType::Intent_Anchor, 0, None);
        let charter = create_artifact("art-002", ArtifactType::Charter, 1, Some("hash-art-001".to_string()));
        let section = create_artifact("art-003", ArtifactType::Section, 2, Some("hash-art-002".to_string()));

        manager.add_artifact(intent).unwrap();
        manager.add_artifact(charter).unwrap();
        manager.add_artifact(section).unwrap();

        manager
            .add_dependency(create_dependency("art-002", "art-001", DependencyType::DerivedFrom))
            .unwrap();
        manager
            .add_dependency(create_dependency("art-003", "art-002", DependencyType::DerivedFrom))
            .unwrap();

        let dependents = manager.get_dependents("art-002");
        println!("Dependents of art-002: {:?}", dependents);
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0], "art-003");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_cs_002_c_leaf_has_no_dependents() {
        println!("\n=== TC-CS-002-C: Leaf has no dependents ===");
        let mut manager = SpineManager::new();

        let intent = create_artifact("art-001", ArtifactType::Intent_Anchor, 0, None);
        let charter = create_artifact("art-002", ArtifactType::Charter, 1, Some("hash-art-001".to_string()));
        let section = create_artifact("art-003", ArtifactType::Section, 2, Some("hash-art-002".to_string()));

        manager.add_artifact(intent).unwrap();
        manager.add_artifact(charter).unwrap();
        manager.add_artifact(section).unwrap();

        manager
            .add_dependency(create_dependency("art-002", "art-001", DependencyType::DerivedFrom))
            .unwrap();
        manager
            .add_dependency(create_dependency("art-003", "art-002", DependencyType::DerivedFrom))
            .unwrap();

        let dependents = manager.get_dependents("art-003");
        println!("Dependents of art-003 (leaf): {:?}", dependents);
        assert_eq!(dependents.len(), 0, "Leaf should have no dependents");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_cs_002_d_nonexistent_artifact() {
        println!("\n=== TC-CS-002-D: Handle missing artifact ===");
        let manager = SpineManager::new();

        let dependents = manager.get_dependents("nonexistent");
        println!("Dependents of nonexistent: {:?}", dependents);
        assert_eq!(dependents.len(), 0, "Nonexistent artifact should return empty list");
        println!("✓ Test passed\n");
    }

    // ===== TC-CS-003: Is On Critical Path Tests =====

    #[test]
    fn tc_cs_003_critical_path_artifacts() {
        println!("\n=== TC-CS-003: Critical Path Artifacts ===");
        let mut manager = SpineManager::new();

        // Add Critical Path artifacts
        let intent = create_artifact("intent-001", ArtifactType::Intent_Anchor, 0, None);
        let charter = create_artifact("charter-001", ArtifactType::Charter, 1, Some("hash-intent-001".to_string()));
        let baseline = create_artifact("baseline-001", ArtifactType::Baseline, 2, Some("hash-charter-001".to_string()));
        let thesis = create_artifact("thesis-001", ArtifactType::Core_Thesis, 3, Some("hash-baseline-001".to_string()));

        // Add non-Critical Path artifacts
        let governance = create_artifact("gov-001", ArtifactType::Governance_Summary, 4, Some("hash-thesis-001".to_string()));
        let lens = create_artifact("lens-001", ArtifactType::Lens_Efficacy_Report, 4, Some("hash-thesis-001".to_string()));
        let innovation = create_artifact("innov-001", ArtifactType::Innovation_Notes, 4, Some("hash-thesis-001".to_string()));

        manager.add_artifact(intent).unwrap();
        manager.add_artifact(charter).unwrap();
        manager.add_artifact(baseline).unwrap();
        manager.add_artifact(thesis).unwrap();
        manager.add_artifact(governance).unwrap();
        manager.add_artifact(lens).unwrap();
        manager.add_artifact(innovation).unwrap();

        // TC-CS-003-A: Intent_Anchor
        println!("TC-CS-003-A: Intent_Anchor");
        assert!(manager.is_on_critical_path("intent-001"), "Intent_Anchor should be on Critical Path");

        // TC-CS-003-B: Charter
        println!("TC-CS-003-B: Charter");
        assert!(manager.is_on_critical_path("charter-001"), "Charter should be on Critical Path");

        // TC-CS-003-C: Baseline
        println!("TC-CS-003-C: Baseline");
        assert!(manager.is_on_critical_path("baseline-001"), "Baseline should be on Critical Path");

        // TC-CS-003-D: Core_Thesis
        println!("TC-CS-003-D: Core_Thesis");
        assert!(manager.is_on_critical_path("thesis-001"), "Core_Thesis should be on Critical Path");

        // TC-CS-003-E: Governance_Summary
        println!("TC-CS-003-E: Governance_Summary");
        assert!(!manager.is_on_critical_path("gov-001"), "Governance_Summary should NOT be on Critical Path");

        // TC-CS-003-F: Lens_Efficacy_Report
        println!("TC-CS-003-F: Lens_Efficacy_Report");
        assert!(!manager.is_on_critical_path("lens-001"), "Lens_Efficacy_Report should NOT be on Critical Path");

        // TC-CS-003-G: Innovation_Notes
        println!("TC-CS-003-G: Innovation_Notes");
        assert!(!manager.is_on_critical_path("innov-001"), "Innovation_Notes should NOT be on Critical Path");

        println!("✓ All Critical Path tests passed\n");
    }

    // ===== TC-CS-004: Validate Spine Integrity Tests =====

    #[test]
    fn tc_cs_004_a_valid_spine() {
        println!("\n=== TC-CS-004-A: Valid spine ===");
        let mut manager = SpineManager::new();

        let intent = create_artifact("art-001", ArtifactType::Intent_Anchor, 0, None);
        let charter = create_artifact("art-002", ArtifactType::Charter, 1, Some("hash-art-001".to_string()));

        manager.add_artifact(intent).unwrap();
        manager.add_artifact(charter).unwrap();
        manager
            .add_dependency(create_dependency("art-002", "art-001", DependencyType::DerivedFrom))
            .unwrap();

        let report = manager.validate_spine_integrity();
        println!("Integrity report: {:?}", report);
        assert!(report.valid, "Valid spine should pass validation");
        assert_eq!(report.breaks.len(), 0);
        assert_eq!(report.orphans.len(), 0);
        assert_eq!(report.cycles.len(), 0);
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_cs_004_b_broken_edge() {
        println!("\n=== TC-CS-004-B: Missing edge target ===");
        let mut manager = SpineManager::new();

        let charter = create_artifact("charter-001", ArtifactType::Charter, 1, Some("hash-intent-001".to_string()));
        manager.add_artifact(charter).unwrap();

        // Add dependency to non-existent artifact
        let broken_dep = create_dependency("charter-001", "intent-001", DependencyType::DerivedFrom);
        manager.dependencies.push(broken_dep);

        let report = manager.validate_spine_integrity();
        println!("Integrity report: {:?}", report);
        assert!(!report.valid, "Broken edge should fail validation");
        assert!(report.breaks.len() > 0, "Should detect broken edge");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_cs_004_c_orphan_artifact() {
        println!("\n=== TC-CS-004-C: Orphan artifact ===");
        let mut manager = SpineManager::new();

        let intent = create_artifact("intent-001", ArtifactType::Intent_Anchor, 0, None);
        let orphan = create_artifact("orphan-001", ArtifactType::Section, 2, Some("hash-unknown".to_string()));

        manager.add_artifact(intent).unwrap();
        manager.add_artifact(orphan).unwrap();
        // No dependency connecting orphan to intent

        let report = manager.validate_spine_integrity();
        println!("Integrity report: {:?}", report);
        assert!(!report.valid, "Orphan should fail validation");
        assert!(report.orphans.contains(&"orphan-001".to_string()), "Should detect orphan");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_cs_004_e_empty_spine() {
        println!("\n=== TC-CS-004-E: Empty spine ===");
        let manager = SpineManager::new();

        let report = manager.validate_spine_integrity();
        println!("Integrity report for empty spine: {:?}", report);
        assert!(report.valid, "Empty spine should be valid");
        assert_eq!(report.breaks.len(), 0);
        assert_eq!(report.orphans.len(), 0);
        assert_eq!(report.cycles.len(), 0);
        println!("✓ Test passed\n");
    }

    // ===== TC-CS-005: Get Lineage Tests =====

    #[test]
    fn tc_cs_005_a_intent_anchor_lineage() {
        println!("\n=== TC-CS-005-A: Intent_Anchor returns itself ===");
        let mut manager = SpineManager::new();

        let intent = create_artifact("intent-001", ArtifactType::Intent_Anchor, 0, None);
        manager.add_artifact(intent).unwrap();

        let lineage = manager.get_lineage("intent-001").unwrap();
        println!("Lineage for Intent_Anchor: {:?}", lineage);
        assert_eq!(lineage.len(), 1);
        assert_eq!(lineage[0], "intent-001");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_cs_005_b_charter_lineage() {
        println!("\n=== TC-CS-005-B: Charter lineage ===");
        let mut manager = SpineManager::new();

        let intent = create_artifact("intent-001", ArtifactType::Intent_Anchor, 0, None);
        let charter = create_artifact("charter-001", ArtifactType::Charter, 1, Some("hash-intent-001".to_string()));

        manager.add_artifact(intent).unwrap();
        manager.add_artifact(charter).unwrap();
        manager
            .add_dependency(create_dependency("charter-001", "intent-001", DependencyType::DerivedFrom))
            .unwrap();

        let lineage = manager.get_lineage("charter-001").unwrap();
        println!("Lineage for Charter: {:?}", lineage);
        assert_eq!(lineage.len(), 2);
        assert_eq!(lineage[0], "charter-001");
        assert_eq!(lineage[1], "intent-001");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_cs_005_c_full_lineage() {
        println!("\n=== TC-CS-005-C: Full lineage trace ===");
        let mut manager = SpineManager::new();

        // Build a full lineage chain
        let intent = create_artifact("intent-001", ArtifactType::Intent_Anchor, 0, None);
        let charter = create_artifact("charter-001", ArtifactType::Charter, 1, Some("hash-intent-001".to_string()));
        let baseline = create_artifact("baseline-001", ArtifactType::Baseline, 2, Some("hash-charter-001".to_string()));
        let thesis = create_artifact("thesis-001", ArtifactType::Core_Thesis, 3, Some("hash-baseline-001".to_string()));
        let diagnostic = create_artifact("diag-001", ArtifactType::Diagnostic_Summary, 4, Some("hash-thesis-001".to_string()));
        let framework = create_artifact("framework-001", ArtifactType::Framework_Draft, 5, Some("hash-diag-001".to_string()));

        manager.add_artifact(intent).unwrap();
        manager.add_artifact(charter).unwrap();
        manager.add_artifact(baseline).unwrap();
        manager.add_artifact(thesis).unwrap();
        manager.add_artifact(diagnostic).unwrap();
        manager.add_artifact(framework).unwrap();

        manager.add_dependency(create_dependency("charter-001", "intent-001", DependencyType::DerivedFrom)).unwrap();
        manager.add_dependency(create_dependency("baseline-001", "charter-001", DependencyType::DerivedFrom)).unwrap();
        manager.add_dependency(create_dependency("thesis-001", "baseline-001", DependencyType::DerivedFrom)).unwrap();
        manager.add_dependency(create_dependency("diag-001", "thesis-001", DependencyType::DerivedFrom)).unwrap();
        manager.add_dependency(create_dependency("framework-001", "diag-001", DependencyType::DerivedFrom)).unwrap();

        let lineage = manager.get_lineage("framework-001").unwrap();
        println!("Full lineage for Framework: {:?}", lineage);
        assert_eq!(lineage.len(), 6);
        assert_eq!(lineage[0], "framework-001");
        assert_eq!(lineage[5], "intent-001");
        println!("✓ Test passed\n");
    }

    #[test]
    fn tc_cs_005_d_orphan_lineage_error() {
        println!("\n=== TC-CS-005-D: Orphan artifact lineage error ===");
        let mut manager = SpineManager::new();

        let orphan = create_artifact("orphan-001", ArtifactType::Section, 2, Some("hash-unknown".to_string()));
        manager.add_artifact(orphan).unwrap();

        let result = manager.get_lineage("orphan-001");
        println!("Lineage result for orphan: {:?}", result);
        assert!(result.is_err(), "Orphan should return error");
        println!("✓ Test passed\n");
    }

    // ===== Additional Tests =====

    #[test]
    fn test_cycle_prevention() {
        println!("\n=== Test: Cycle Prevention ===");
        let mut manager = SpineManager::new();

        let art1 = create_artifact("art-001", ArtifactType::Section, 1, None);
        let art2 = create_artifact("art-002", ArtifactType::Section, 2, None);
        let art3 = create_artifact("art-003", ArtifactType::Section, 3, None);

        manager.add_artifact(art1).unwrap();
        manager.add_artifact(art2).unwrap();
        manager.add_artifact(art3).unwrap();

        // Create a path: art1 -> art2 -> art3
        manager.add_dependency(create_dependency("art-001", "art-002", DependencyType::DerivedFrom)).unwrap();
        manager.add_dependency(create_dependency("art-002", "art-003", DependencyType::DerivedFrom)).unwrap();

        // Try to create a cycle: art3 -> art1 (should fail)
        let result = manager.add_dependency(create_dependency("art-003", "art-001", DependencyType::DerivedFrom));
        println!("Cycle creation result: {:?}", result);
        assert!(result.is_err(), "Should prevent cycle creation");
        println!("✓ Test passed\n");
    }

    #[test]
    fn test_comprehensive_spine() {
        println!("\n=== Test: Comprehensive Spine Workflow ===");
        let mut manager = SpineManager::new();

        // Build a realistic Method-VI spine
        println!("Building Critical Path...");
        let intent = create_artifact("intent-001", ArtifactType::Intent_Anchor, 0, None);
        let charter = create_artifact("charter-001", ArtifactType::Charter, 1, Some("hash-intent-001".to_string()));
        let baseline = create_artifact("baseline-001", ArtifactType::Baseline, 2, Some("hash-charter-001".to_string()));
        let thesis = create_artifact("thesis-001", ArtifactType::Core_Thesis, 3, Some("hash-baseline-001".to_string()));

        manager.add_artifact(intent).unwrap();
        manager.add_artifact(charter).unwrap();
        manager.add_artifact(baseline).unwrap();
        manager.add_artifact(thesis).unwrap();

        manager.add_dependency(create_dependency("charter-001", "intent-001", DependencyType::DerivedFrom)).unwrap();
        manager.add_dependency(create_dependency("baseline-001", "charter-001", DependencyType::DerivedFrom)).unwrap();
        manager.add_dependency(create_dependency("thesis-001", "baseline-001", DependencyType::DerivedFrom)).unwrap();

        println!("Adding supporting artifacts...");
        let section1 = create_artifact("section-001", ArtifactType::Section, 4, Some("hash-thesis-001".to_string()));
        let section2 = create_artifact("section-002", ArtifactType::Section, 4, Some("hash-thesis-001".to_string()));

        manager.add_artifact(section1).unwrap();
        manager.add_artifact(section2).unwrap();

        manager.add_dependency(create_dependency("section-001", "thesis-001", DependencyType::DerivedFrom)).unwrap();
        manager.add_dependency(create_dependency("section-002", "thesis-001", DependencyType::DerivedFrom)).unwrap();
        manager.add_dependency(create_dependency("section-002", "charter-001", DependencyType::ConstrainedBy)).unwrap();

        // Validate entire spine
        println!("Validating spine integrity...");
        let report = manager.validate_spine_integrity();
        assert!(report.valid, "Comprehensive spine should be valid");
        println!("✓ Spine is valid");

        // Test queries
        println!("Testing queries...");
        assert!(manager.is_on_critical_path("intent-001"));
        assert!(manager.is_on_critical_path("charter-001"));
        assert!(manager.is_on_critical_path("baseline-001"));
        assert!(manager.is_on_critical_path("thesis-001"));
        assert!(!manager.is_on_critical_path("section-001"));

        let thesis_dependents = manager.get_dependents("thesis-001");
        assert_eq!(thesis_dependents.len(), 2);

        let section2_deps = manager.get_dependencies("section-002");
        assert_eq!(section2_deps.len(), 2);

        let section1_lineage = manager.get_lineage("section-001").unwrap();
        assert_eq!(section1_lineage.len(), 5); // section -> thesis -> baseline -> charter -> intent

        println!("✓ All queries successful");
        println!("✓ Comprehensive spine test passed\n");
    }
}
