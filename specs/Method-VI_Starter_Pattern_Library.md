# Method-VI Starter Pattern Library

**Version:** 1.0.0  
**Status:** SPECIFICATION - Ready for Content Development  
**Parent Document:** module-plan-method-vi.md  
**Purpose:** Solve the "Cold Start" problem by shipping pre-installed patterns with the application

---

## Overview

### The Cold Start Problem

As identified in the cross-LLM architectural review:

> "As a standalone desktop app with local storage, a new user has **zero patterns**. This means Step 0 and Step 6.5 (Learning Harvest) will be functionally useless for the first 10-20 runs until the user builds their own library."

### Solution

Ship 8-10 pre-installed patterns (`is_starter=1`) covering common intent categories. These patterns:

- Demonstrate the Learning Plane's value immediately at Step 0
- Provide proven starting points for common work types
- Can be modified or rejected by users
- Will be supplemented by user-generated patterns over time

---

## Pattern Categories

Method-VI defines three Intent Categories (from Core v1.0.1):

| Category | Description | Use Cases |
|----------|-------------|-----------|
| **Exploratory** | Open-ended investigation, discovery, ideation | Research, brainstorming, concept development |
| **Analytical** | Structured examination, diagnosis, assessment | Audits, reviews, root cause analysis, comparisons |
| **Operational** | Process design, implementation planning, execution | Frameworks, workflows, project plans, procedures |

### Distribution

The starter library includes patterns across all three categories:

- **Exploratory:** 2 patterns
- **Analytical:** 4 patterns  
- **Operational:** 4 patterns

---

## Pattern Schema

Each starter pattern follows this structure, aligned with the `patterns` table in the Knowledge Repository:

```yaml
# Pattern Card - Starter Library Format
id: "starter-{category}-{number}"  # e.g., "starter-analytical-001"
intent_category: "Exploratory | Analytical | Operational"

# Success Metrics (simulated for starters, based on expected performance)
ci_achievement: 0.85  # Minimum threshold for pattern extraction
ev_stability: 5.0     # Expected EV range (±%)
ias_score: 0.90       # Expected Intent Alignment

# Pattern Components (JSON blobs in database)
architecture_pattern:
  charter_type: string
  flow_geometry: "Linear | Cyclic | Branching"
  reflection_cadence: string
  recommended_telemetry: "Lite | Standard | Full"

analysis_pattern:
  high_value_lenses: [array of lens names]
  lens_sequence: [ordered array]
  lens_efficacy_notes: string

synthesis_pattern:
  model_geometry: "Linear | Cyclic | Branching"
  thesis_approach: string
  typical_principle_count: integer

structure_pattern:
  framework_type: string
  typical_section_count: integer
  innovation_notes: string

validation_pattern:
  effective_tests: [array]
  recommended_adversarial: [array or null]

# Applicability Guidance
applicability:
  similar_contexts: [array of use cases]
  pitfalls: [array of when NOT to use]
  adaptations: [array of customization suggestions]

# Vitality Metadata (initialized for starters)
vitality_freshness: 1.0
vitality_relevance: 1.0
application_count: 0
success_count: 0
created_at: "2025-01-01T00:00:00Z"  # Release date
last_applied: null
source_run_id: null  # No source run for starters
is_starter: 1
```

---

## Starter Pattern Definitions

### Pattern 1: Strategic Planning

```yaml
id: "starter-operational-001"
intent_category: "Operational"
name: "Strategic Planning"
description: "Framework development for organizational strategy, roadmaps, and long-term planning"

ci_achievement: 0.87
ev_stability: 8.0
ias_score: 0.88

architecture_pattern:
  charter_type: "Vision-to-execution cascade with stakeholder alignment"
  flow_geometry: "Linear"
  reflection_cadence: "After each major section (vision, analysis, strategy, roadmap)"
  recommended_telemetry: "Standard"

analysis_pattern:
  high_value_lenses: ["Structural", "Thematic", "Intent"]
  lens_sequence: ["Intent", "Thematic", "Structural", "Logic", "Evidence", "Expression"]
  lens_efficacy_notes: "Intent lens critical for stakeholder alignment; Thematic reveals strategic pillars"

synthesis_pattern:
  model_geometry: "Linear"
  thesis_approach: "North-star vision statement derived from thematic convergence"
  typical_principle_count: 5

structure_pattern:
  framework_type: "Hierarchical cascade (Vision → Pillars → Objectives → Initiatives)"
  typical_section_count: 6
  innovation_notes: "Success metrics embedded at each level for traceability"

validation_pattern:
  effective_tests: ["Logic validation of strategy-to-action links", "Stakeholder alignment check"]
  recommended_adversarial: ["Assumption inversion on market conditions"]

applicability:
  similar_contexts:
    - "Annual strategic planning"
    - "Business unit strategy development"
    - "Product roadmap creation"
    - "Organizational transformation planning"
  pitfalls:
    - "Not suitable for tactical/operational quick fixes"
    - "Requires executive stakeholder input; don't use if unavailable"
    - "May over-engineer simple initiatives"
  adaptations:
    - "For startups: compress to 3-section lightweight version"
    - "For enterprises: expand validation with compliance checks"
    - "For product teams: replace Pillars with Product Areas"
```

---

### Pattern 2: Root Cause Analysis

```yaml
id: "starter-analytical-001"
intent_category: "Analytical"
name: "Root Cause Analysis"
description: "Systematic diagnosis of problems to identify underlying causes rather than symptoms"

ci_achievement: 0.89
ev_stability: 4.0
ias_score: 0.92

architecture_pattern:
  charter_type: "Problem-focused investigation with causal chain mapping"
  flow_geometry: "Branching"
  reflection_cadence: "After each causal hypothesis is tested"
  recommended_telemetry: "Full"

analysis_pattern:
  high_value_lenses: ["Logic", "Evidence", "Structural"]
  lens_sequence: ["Evidence", "Logic", "Structural", "Thematic", "Intent", "Expression"]
  lens_efficacy_notes: "Evidence lens surfaces data; Logic lens tests causal claims; Structural reveals system interactions"

synthesis_pattern:
  model_geometry: "Branching"
  thesis_approach: "Causal tree with weighted probability paths to root causes"
  typical_principle_count: 3

structure_pattern:
  framework_type: "Fishbone/Ishikawa with evidence annotations"
  typical_section_count: 5
  innovation_notes: "Each branch includes confidence score and evidence quality rating"

validation_pattern:
  effective_tests: ["Logic validation of causal chains", "Evidence audit for each claim"]
  recommended_adversarial: ["Assumption inversion", "Alternative cause injection"]

applicability:
  similar_contexts:
    - "Production incident post-mortems"
    - "Quality defect analysis"
    - "Process failure investigation"
    - "Customer complaint root cause"
  pitfalls:
    - "Not for problems with obvious solutions"
    - "Requires access to evidence/data; insufficient data = weak analysis"
    - "Can over-analyze simple issues"
  adaptations:
    - "For IT incidents: add timeline reconstruction step"
    - "For manufacturing: integrate 5-Why methodology"
    - "For service failures: add customer journey mapping"
```

---

### Pattern 3: Comparative Assessment

```yaml
id: "starter-analytical-002"
intent_category: "Analytical"
name: "Comparative Assessment"
description: "Structured comparison of options, vendors, approaches, or solutions against defined criteria"

ci_achievement: 0.86
ev_stability: 6.0
ias_score: 0.90

architecture_pattern:
  charter_type: "Criteria-based evaluation with weighted scoring"
  flow_geometry: "Linear"
  reflection_cadence: "After criteria definition, after each option analysis"
  recommended_telemetry: "Standard"

analysis_pattern:
  high_value_lenses: ["Evidence", "Structural", "Logic"]
  lens_sequence: ["Structural", "Evidence", "Logic", "Thematic", "Expression", "Intent"]
  lens_efficacy_notes: "Structural defines comparison framework; Evidence populates data; Logic validates scoring"

synthesis_pattern:
  model_geometry: "Linear"
  thesis_approach: "Recommendation with confidence level based on criteria coverage"
  typical_principle_count: 4

structure_pattern:
  framework_type: "Evaluation matrix with narrative justifications"
  typical_section_count: 5
  innovation_notes: "Sensitivity analysis section showing how results change with weight adjustments"

validation_pattern:
  effective_tests: ["Evidence audit for data accuracy", "Logic validation of scoring rationale"]
  recommended_adversarial: ["Weight sensitivity testing"]

applicability:
  similar_contexts:
    - "Vendor selection"
    - "Technology evaluation"
    - "Investment comparison"
    - "Policy option analysis"
  pitfalls:
    - "Not for decisions with single dominant factor"
    - "Requires comparable options; apples-to-oranges comparisons fail"
    - "Can create false precision with arbitrary weights"
  adaptations:
    - "For procurement: add TCO calculations"
    - "For technical decisions: add integration complexity scoring"
    - "For policy: add stakeholder impact matrix"
```

---

### Pattern 4: Process Design

```yaml
id: "starter-operational-002"
intent_category: "Operational"
name: "Process Design"
description: "Creation of new workflows, procedures, or operational processes"

ci_achievement: 0.88
ev_stability: 7.0
ias_score: 0.89

architecture_pattern:
  charter_type: "End-to-end process with roles, triggers, and outputs"
  flow_geometry: "Cyclic"
  reflection_cadence: "After each process phase design"
  recommended_telemetry: "Standard"

analysis_pattern:
  high_value_lenses: ["Structural", "Logic", "Intent"]
  lens_sequence: ["Intent", "Structural", "Logic", "Evidence", "Thematic", "Expression"]
  lens_efficacy_notes: "Intent clarifies process purpose; Structural maps flow; Logic validates handoffs"

synthesis_pattern:
  model_geometry: "Cyclic"
  thesis_approach: "Process purpose statement with success criteria"
  typical_principle_count: 4

structure_pattern:
  framework_type: "SIPOC-extended (Suppliers, Inputs, Process, Outputs, Customers + Metrics)"
  typical_section_count: 7
  innovation_notes: "Exception handling paths documented alongside happy path"

validation_pattern:
  effective_tests: ["Logic validation of process flow", "Completeness check for edge cases"]
  recommended_adversarial: ["Edge case injection", "Scale stress testing"]

applicability:
  similar_contexts:
    - "New workflow creation"
    - "Process standardization"
    - "Automation preparation"
    - "Compliance procedure design"
  pitfalls:
    - "Not for process optimization (use Process Improvement pattern)"
    - "Requires clear process boundaries; fuzzy scope = fuzzy process"
    - "Can over-engineer simple procedures"
  adaptations:
    - "For automation: add decision logic specifications"
    - "For compliance: add control point annotations"
    - "For customer-facing: add experience checkpoints"
```

---

### Pattern 5: Research Synthesis

```yaml
id: "starter-exploratory-001"
intent_category: "Exploratory"
name: "Research Synthesis"
description: "Integration of multiple sources, studies, or perspectives into coherent findings"

ci_achievement: 0.85
ev_stability: 10.0
ias_score: 0.86

architecture_pattern:
  charter_type: "Multi-source integration with theme extraction"
  flow_geometry: "Branching"
  reflection_cadence: "After each source cluster analysis"
  recommended_telemetry: "Full"

analysis_pattern:
  high_value_lenses: ["Thematic", "Evidence", "Logic"]
  lens_sequence: ["Evidence", "Thematic", "Logic", "Structural", "Intent", "Expression"]
  lens_efficacy_notes: "Thematic lens critical for pattern recognition across sources"

synthesis_pattern:
  model_geometry: "Branching"
  thesis_approach: "Emergent themes with convergence/divergence mapping"
  typical_principle_count: 3

structure_pattern:
  framework_type: "Thematic clusters with source attribution"
  typical_section_count: 5
  innovation_notes: "Confidence levels per theme based on source agreement"

validation_pattern:
  effective_tests: ["Evidence audit for source quality", "Logic validation of theme derivation"]
  recommended_adversarial: ["Alternative interpretation generation"]

applicability:
  similar_contexts:
    - "Literature review"
    - "Market research synthesis"
    - "Competitive intelligence compilation"
    - "Expert interview integration"
  pitfalls:
    - "Not for single-source analysis"
    - "Requires diverse sources; homogeneous sources = confirmation bias"
    - "Can lose nuance in over-synthesis"
  adaptations:
    - "For academic: add methodology quality scoring"
    - "For market research: add recency weighting"
    - "For competitive intel: add source reliability ratings"
```

---

### Pattern 6: Framework Development

```yaml
id: "starter-operational-003"
intent_category: "Operational"
name: "Framework Development"
description: "Creation of conceptual frameworks, models, or methodologies for repeated use"

ci_achievement: 0.90
ev_stability: 5.0
ias_score: 0.91

architecture_pattern:
  charter_type: "Reusable model with clear boundaries and application guidance"
  flow_geometry: "Cyclic"
  reflection_cadence: "After core model, after each extension"
  recommended_telemetry: "Full"

analysis_pattern:
  high_value_lenses: ["Structural", "Thematic", "Logic"]
  lens_sequence: ["Thematic", "Structural", "Logic", "Intent", "Evidence", "Expression"]
  lens_efficacy_notes: "Thematic identifies core concepts; Structural creates architecture; Logic validates relationships"

synthesis_pattern:
  model_geometry: "Cyclic"
  thesis_approach: "Core principle statement with component relationship map"
  typical_principle_count: 5

structure_pattern:
  framework_type: "Layered model (Core → Components → Applications → Extensions)"
  typical_section_count: 8
  innovation_notes: "Anti-patterns section documenting misuse cases"

validation_pattern:
  effective_tests: ["Logic validation of component relationships", "Completeness check for use cases"]
  recommended_adversarial: ["Edge case injection", "Context shift testing"]

applicability:
  similar_contexts:
    - "Methodology creation"
    - "Mental model development"
    - "Assessment framework design"
    - "Decision framework creation"
  pitfalls:
    - "Not for one-time analyses"
    - "Requires abstraction capability; too concrete = not reusable"
    - "Can over-complicate simple concepts"
  adaptations:
    - "For teaching: add progressive complexity levels"
    - "For enterprise: add governance and ownership sections"
    - "For tools: add implementation specifications"
```

---

### Pattern 7: Audit & Compliance Review

```yaml
id: "starter-analytical-003"
intent_category: "Analytical"
name: "Audit & Compliance Review"
description: "Systematic assessment against standards, requirements, or best practices"

ci_achievement: 0.91
ev_stability: 3.0
ias_score: 0.93

architecture_pattern:
  charter_type: "Criteria-based assessment with gap identification"
  flow_geometry: "Linear"
  reflection_cadence: "After each assessment domain"
  recommended_telemetry: "Full"

analysis_pattern:
  high_value_lenses: ["Evidence", "Logic", "Structural"]
  lens_sequence: ["Structural", "Evidence", "Logic", "Intent", "Thematic", "Expression"]
  lens_efficacy_notes: "Evidence lens critical for substantiation; Structure ensures coverage"

synthesis_pattern:
  model_geometry: "Linear"
  thesis_approach: "Compliance status with prioritized remediation roadmap"
  typical_principle_count: 3

structure_pattern:
  framework_type: "Control matrix with finding severity and remediation"
  typical_section_count: 6
  innovation_notes: "Risk-based prioritization of findings"

validation_pattern:
  effective_tests: ["Evidence audit for each finding", "Logic validation of severity ratings"]
  recommended_adversarial: null

applicability:
  similar_contexts:
    - "Regulatory compliance assessment"
    - "Security audit"
    - "Process maturity assessment"
    - "Standards conformance review"
  pitfalls:
    - "Not for exploratory analysis"
    - "Requires clear criteria; ambiguous standards = subjective findings"
    - "Can miss systemic issues by focusing on checklist items"
  adaptations:
    - "For security: add threat modeling integration"
    - "For regulatory: add citation requirements"
    - "For maturity: add capability level definitions"
```

---

### Pattern 8: Concept Development

```yaml
id: "starter-exploratory-002"
intent_category: "Exploratory"
name: "Concept Development"
description: "Ideation and refinement of new concepts, products, or initiatives"

ci_achievement: 0.85
ev_stability: 12.0
ias_score: 0.84

architecture_pattern:
  charter_type: "Open exploration with progressive convergence"
  flow_geometry: "Branching"
  reflection_cadence: "After divergent phase, after each convergence"
  recommended_telemetry: "Standard"

analysis_pattern:
  high_value_lenses: ["Thematic", "Intent", "Expression"]
  lens_sequence: ["Intent", "Thematic", "Expression", "Structural", "Logic", "Evidence"]
  lens_efficacy_notes: "Intent grounds exploration; Thematic reveals patterns; Expression tests communication"

synthesis_pattern:
  model_geometry: "Branching"
  thesis_approach: "Concept statement with differentiation and value proposition"
  typical_principle_count: 3

structure_pattern:
  framework_type: "Concept canvas (Problem, Solution, Value, Differentiation)"
  typical_section_count: 5
  innovation_notes: "Includes 'killed ideas' section for learning"

validation_pattern:
  effective_tests: ["Logic validation of value claims", "Expression clarity testing"]
  recommended_adversarial: ["Assumption inversion", "Context shift"]

applicability:
  similar_contexts:
    - "New product ideation"
    - "Service design"
    - "Initiative conceptualization"
    - "Innovation workshops"
  pitfalls:
    - "Not for execution planning (use different pattern after concept is validated)"
    - "Requires tolerance for ambiguity; premature convergence kills innovation"
    - "Can generate ideas without feasibility grounding"
  adaptations:
    - "For products: add feasibility/viability gates"
    - "For services: add customer journey sketching"
    - "For initiatives: add stakeholder impact mapping"
```

---

### Pattern 9: Stakeholder Analysis

```yaml
id: "starter-analytical-004"
intent_category: "Analytical"
name: "Stakeholder Analysis"
description: "Systematic mapping of stakeholder interests, influence, and engagement strategies"

ci_achievement: 0.87
ev_stability: 5.0
ias_score: 0.89

architecture_pattern:
  charter_type: "Multi-dimensional stakeholder mapping with engagement planning"
  flow_geometry: "Linear"
  reflection_cadence: "After identification, after each stakeholder cluster"
  recommended_telemetry: "Standard"

analysis_pattern:
  high_value_lenses: ["Intent", "Thematic", "Structural"]
  lens_sequence: ["Intent", "Thematic", "Structural", "Logic", "Evidence", "Expression"]
  lens_efficacy_notes: "Intent reveals stakeholder motivations; Thematic groups by interest"

synthesis_pattern:
  model_geometry: "Linear"
  thesis_approach: "Engagement strategy prioritized by influence and interest"
  typical_principle_count: 4

structure_pattern:
  framework_type: "Power/Interest matrix with engagement playbooks"
  typical_section_count: 5
  innovation_notes: "Dynamic updating triggers for stakeholder position changes"

validation_pattern:
  effective_tests: ["Logic validation of influence assessments", "Completeness check for stakeholder coverage"]
  recommended_adversarial: ["Position shift scenarios"]

applicability:
  similar_contexts:
    - "Change management planning"
    - "Project stakeholder mapping"
    - "Political landscape analysis"
    - "Partnership strategy"
  pitfalls:
    - "Not for transactional relationships"
    - "Requires honest assessment; political sensitivity can distort"
    - "Can become stale quickly; needs regular updates"
  adaptations:
    - "For change management: add resistance/support forecasting"
    - "For projects: add RACI integration"
    - "For partnerships: add value exchange mapping"
```

---

### Pattern 10: Project Planning

```yaml
id: "starter-operational-004"
intent_category: "Operational"
name: "Project Planning"
description: "Comprehensive project plan development including scope, timeline, resources, and risks"

ci_achievement: 0.88
ev_stability: 6.0
ias_score: 0.90

architecture_pattern:
  charter_type: "Structured project definition with execution roadmap"
  flow_geometry: "Linear"
  reflection_cadence: "After scope, after schedule, after risk assessment"
  recommended_telemetry: "Standard"

analysis_pattern:
  high_value_lenses: ["Structural", "Logic", "Evidence"]
  lens_sequence: ["Intent", "Structural", "Logic", "Evidence", "Thematic", "Expression"]
  lens_efficacy_notes: "Structural defines WBS; Logic validates dependencies; Evidence grounds estimates"

synthesis_pattern:
  model_geometry: "Linear"
  thesis_approach: "Project success criteria with critical path identification"
  typical_principle_count: 4

structure_pattern:
  framework_type: "Integrated project plan (Scope, Schedule, Resources, Risks, Governance)"
  typical_section_count: 7
  innovation_notes: "Decision log template integrated for change management"

validation_pattern:
  effective_tests: ["Logic validation of dependencies", "Completeness check for deliverables"]
  recommended_adversarial: ["Schedule compression stress test", "Resource constraint scenarios"]

applicability:
  similar_contexts:
    - "Initiative planning"
    - "Product launch planning"
    - "Implementation planning"
    - "Program component planning"
  pitfalls:
    - "Not for ongoing operations (use Process Design)"
    - "Requires defined end state; open-ended work doesn't fit"
    - "Can create false precision in uncertain environments"
  adaptations:
    - "For agile: replace detailed schedule with sprint structure"
    - "For enterprise: add governance and approval gates"
    - "For technical: add architecture decision records"
```

---

## Installation Specification

### Database Insertion

Starter patterns are inserted during first-run initialization (`initialize-method-vi` workflow):

```sql
-- Example insertion for Strategic Planning pattern
INSERT INTO patterns (
  id, intent_category, ci_achievement, ev_stability,
  architecture_pattern, analysis_pattern, synthesis_pattern,
  structure_pattern, validation_pattern, applicability,
  vitality_freshness, vitality_relevance, application_count,
  success_count, created_at, last_applied, source_run_id, is_starter
) VALUES (
  'starter-operational-001',
  'Operational',
  0.87,
  8.0,
  '{"charter_type": "Vision-to-execution cascade...", ...}',
  '{"high_value_lenses": ["Structural", "Thematic", "Intent"], ...}',
  '{"model_geometry": "Linear", ...}',
  '{"framework_type": "Hierarchical cascade...", ...}',
  '{"effective_tests": ["Logic validation...", ...], ...}',
  '{"similar_contexts": [...], "pitfalls": [...], "adaptations": [...]}',
  1.0, 1.0, 0, 0,
  '2025-01-01T00:00:00Z',
  NULL, NULL, 1
);
```

### File Distribution

Patterns can be distributed as:

1. **Embedded SQL** - Initialization script with INSERT statements
2. **JSON seed file** - `data/starter-patterns.json` loaded at first run
3. **SQLite seed database** - Pre-populated `patterns.db` shipped with installer

**Recommended:** JSON seed file for easier updates and human readability.

---

## Pattern Recommendation Logic

At Step 0, the Scope & Pattern Agent queries starter patterns:

```sql
SELECT id, intent_category, applicability, ci_achievement
FROM patterns
WHERE intent_category = :user_intent_category
  AND (vitality_freshness * 0.4 + vitality_relevance * 0.6) > 0.3
ORDER BY ci_achievement DESC, application_count DESC
LIMIT 5;
```

### Fit Score Calculation

For each pattern, calculate fit score based on:

- **Category match:** 40% weight (exact match = 1.0, adjacent = 0.5, mismatch = 0)
- **Context similarity:** 40% weight (keyword matching against `similar_contexts`)
- **Success history:** 20% weight (ci_achievement normalized)

Present top 3-5 patterns with:
- Pattern name and description
- Fit score (0-100)
- "When it works well" (from `similar_contexts`)
- "Pitfalls to avoid" (from `pitfalls`)

---

## Maintenance & Evolution

### User Modifications

Users can:
- **Apply** a starter pattern (increments `application_count`)
- **Succeed** with a pattern (increments `success_count` if CI ≥ 0.85)
- **Reject** a pattern (no count change, but logged for learning)

Starter patterns cannot be deleted but can be "hidden" via user preference.

### Version Updates

When shipping new application versions:
- New starter patterns can be added (`is_starter=1`)
- Existing starter patterns are not modified (user success/application counts preserved)
- Deprecated patterns marked in release notes but not removed

### User-Generated Patterns

Patterns extracted at Step 6.5 have `is_starter=0` and `source_run_id` populated. These supplement but don't replace starter patterns.

---

## Success Criteria

The starter pattern library succeeds when:

1. **Step 0 demonstrates value** - New users see relevant recommendations immediately
2. **Patterns are applied** - ≥50% of users apply at least one starter pattern in first 5 runs
3. **Patterns lead to success** - Applied starter patterns achieve CI ≥ 0.85 at ≥60% rate
4. **Users build on patterns** - ≥30% of users extract their own patterns within 20 runs

---

## External Documentation Status

| Item | Status | Notes |
|------|--------|-------|
| Pattern schema | ✅ Complete | Aligned with module plan |
| 10 pattern definitions | ✅ Complete | Ready for review |
| Installation specification | ✅ Complete | JSON seed recommended |
| Recommendation logic | ✅ Complete | SQL + fit score algorithm |

**Next Steps:**
1. Review pattern definitions with domain experts
2. Create `starter-patterns.json` seed file
3. Implement pattern recommendation in Scope & Pattern Agent
4. Test fit score algorithm with sample intents

---

**Document Created:** 2025-12-17  
**Aligned With:** module-plan-method-vi.md (Architecture Hardened)  
**Status:** Ready for Content Review
