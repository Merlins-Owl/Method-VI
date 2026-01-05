# Method-VI

AI-assisted framework development with adaptive governance and progressive quality metrics.

---

## Overview

Method-VI is a desktop application built with Tauri that guides users through a 7-step framework development process. It uses AI agents to analyze code quality, provide adaptive feedback, and maintain an immutable audit trail of framework evolution.

**Key Capabilities:**
- **7-Step Guided Development** - Structured progression from charter definition to validation
- **Adaptive Mode Detection** - Automatically classifies projects as Architecting, Builder, or Refining based on maturity
- **Callout-Based Governance** - Tiered feedback system (Info → Attention → Warning → Critical) instead of binary HALT conditions
- **Real-Time Quality Metrics** - Track Conceptual Integrity (CI), Implementation Alignment Score (IAS), Evidence-to-Framework Index (EFI), and Progression Consistency Index (PCI)
- **Immutable Audit Trail** - Steno-Ledger records every step, metric, and decision for complete transparency

---

## Prerequisites

**Required Software:**
- **Node.js**: v18.0.0 or higher (tested with v24.12.0)
- **npm**: 8.0.0 or higher (tested with 11.7.0)
- **Rust**: 1.70.0 or higher (tested with 1.92.0)
- **Cargo**: 1.70.0 or higher (tested with 1.92.0)

**Verify your versions:**
```bash
node --version    # Should show v18.0.0+
npm --version     # Should show 8.0.0+
rustc --version   # Should show 1.70.0+
cargo --version   # Should show 1.70.0+
```

---

## Installation

```bash
# Clone the repository
git clone https://github.com/Merlins-Owl/Method-VI.git

# Navigate to the application directory
cd Method-VI/method-vi-app/method-vi

# Install dependencies
npm install
```

---

## Configuration

**API Key Setup:**

Method-VI uses the Anthropic Claude API for AI-powered analysis. You'll need to configure your API key:

1. **Environment Variable** (Recommended for development):
   ```bash
   # Windows (PowerShell)
   $env:ANTHROPIC_API_KEY="your-api-key-here"

   # Windows (Command Prompt)
   set ANTHROPIC_API_KEY=your-api-key-here

   # macOS/Linux
   export ANTHROPIC_API_KEY="your-api-key-here"
   ```

2. **Local Configuration File**:
   - API keys in `.claude/settings.local.json` and test scripts are gitignored for security
   - Never commit API keys to version control
   - Create your own local configuration files as needed

**Get an API Key:**
- Sign up at https://console.anthropic.com/
- Generate an API key from your account dashboard

---

## Running the App

### Development Mode

```bash
# From method-vi-app/method-vi/
npm run tauri dev
```

This launches the app with hot-reload enabled. The frontend runs on `http://localhost:1420` and automatically reloads when you edit React components. Rust changes require a restart.

### Production Build

```bash
# Build the application for distribution
npm run tauri build
```

The compiled app will be in `src-tauri/target/release/`.

### Frontend Only (for UI development)

```bash
# Run Vite dev server without Tauri
npm run dev

# Build frontend assets
npm run build

# Preview production frontend build
npm run preview
```

---

## Running Tests

### Backend Tests (Rust)

```bash
# Navigate to Rust backend directory
cd method-vi-app/method-vi/src-tauri

# Run all tests (187 total tests)
cargo test

# Run specific test suites
cargo test governance        # 68 governance tests (callout + mode detection)
cargo test --test test_metrics
cargo test --test test_validation_agent
cargo test --test test_step_1

# Run with output logs
cargo test -- --nocapture
```

**Expected Results:**
- 187 total tests passing
- 68 governance tests (callout system + mode detection)
- All metrics tests passing (CI, IAS, EFI, PCI)

### Frontend Tests

```bash
# From method-vi-app/method-vi/
npm test
```

**Manual Testing:**

See `PHASE-5-TESTING-CHECKLIST.md` in the project root for comprehensive manual test cases covering:
- Mode detection UX (7 tests)
- Callout panel interactions (10 tests)
- Gate blocking behavior (6 tests)
- Auto-refresh (2 tests)
- Edge cases (4 tests)

---

## Project Structure

```
method-vi-app/
├── method-vi/                      # Main application directory
│   ├── src/                        # React frontend (TypeScript)
│   │   ├── components/             # UI components
│   │   │   ├── CalloutBadge.tsx    # Callout summary badge
│   │   │   ├── CalloutPanel.tsx    # Callout review modal
│   │   │   ├── ModeBadge.tsx       # Mode detection display
│   │   │   └── StatusBar.tsx       # Header status bar
│   │   ├── pages/                  # Route pages
│   │   ├── types/                  # TypeScript types
│   │   │   └── callouts.ts         # FFI contract types
│   │   └── utils/                  # Utilities
│   │       └── calloutApi.ts       # Tauri command wrappers
│   ├── src-tauri/                  # Rust backend
│   │   ├── src/
│   │   │   ├── agents/             # AI analysis agents
│   │   │   │   ├── analysis_synthesis.rs
│   │   │   │   ├── charter.rs
│   │   │   │   ├── elaboration.rs
│   │   │   │   ├── flame_test.rs
│   │   │   │   ├── governance_telemetry.rs
│   │   │   │   ├── orchestrator.rs
│   │   │   │   └── validation_learning.rs
│   │   │   ├── commands/           # Tauri commands (FFI)
│   │   │   │   ├── callout_commands.rs
│   │   │   │   ├── mode_commands.rs
│   │   │   │   └── mod.rs
│   │   │   ├── governance/         # Callout & mode system
│   │   │   │   ├── callout_manager.rs
│   │   │   │   ├── mode_detector.rs
│   │   │   │   └── types.rs
│   │   │   ├── database.rs         # SQLite interface
│   │   │   └── lib.rs              # Library entry point
│   │   ├── tests/                  # Integration tests
│   │   └── Cargo.toml              # Rust dependencies
│   ├── package.json                # Node dependencies
│   └── README.md                   # Tauri template README
├── ARCHITECTURE-2026-01-05-Phase5-Progression.md  # Full architecture docs
├── PHASE-5-TESTING-CHECKLIST.md    # Manual test cases
└── README.md                       # This file
```

---

## Documentation

### Architecture & Design

- **[ARCHITECTURE-2026-01-05-Phase5-Progression.md](ARCHITECTURE-2026-01-05-Phase5-Progression.md)** - Complete system architecture documentation including:
  - Progression architecture philosophy
  - 4 non-negotiable constraints
  - Callout system design
  - Mode detection algorithm
  - Frontend integration patterns
  - Complete file reference (64 files)

### Testing

- **[PHASE-5-TESTING-CHECKLIST.md](PHASE-5-TESTING-CHECKLIST.md)** - 66 manual test cases for frontend integration
- **[TEST-RESULTS-E2E-2025-12-31.md](TEST-RESULTS-E2E-2025-12-31.md)** - End-to-end test results for metrics redesign

### Implementation Summaries

- **FIX-021 through FIX-027** - Detailed implementation summaries for metrics redesign fixes
- **[INTEGRATION-TEST-REPORT-2025-12-31.md](INTEGRATION-TEST-REPORT-2025-12-31.md)** - Integration test report

---

## How Method-VI Works

### The 7-Step Process

1. **Step 0: Charter Definition** - Define the framework's purpose and scope
2. **Step 1: Elaboration** - Expand framework details and structure
3. **Step 2: Baseline CI** - Establish baseline Conceptual Integrity score
4. **Step 3: Flame Test (CI Delta)** - Measure CI improvement from iterative refinement
5. **Step 4: Implementation** - Develop code implementation of the framework
6. **Step 5: Implementation Alignment (IAS)** - Assess code-to-framework fidelity
7. **Step 6: Validation** - Final review and evidence gathering

### Mode Detection

After Step 2, Method-VI automatically classifies your project:

- **Architecting** (CI < 0.50) - Early-stage, exploratory work → Lower thresholds, noise filtering
- **Builder** (0.50 ≤ CI < 0.80) - Active development → Standard thresholds
- **Refining** (CI ≥ 0.80) - Mature, polishing phase → Elevated thresholds

Mode affects threshold scaling and callout generation, adapting governance to project maturity.

### Callout System

Instead of binary HALT conditions, Method-VI uses tiered callouts:

- **Info** (Green) - Informational, no action required
- **Attention** (Yellow) - Worth noting, monitor
- **Warning** (Orange) - Should address, but can proceed
- **Critical** (Red) - Must acknowledge before gate approval

Only Critical callouts block progression through gates, creating friction-based governance rather than hard stops.

---

## Current Status

**Version**: 0.1.0 (Phase 5 Complete)

**Phase 5 (Progression Architecture) - Completed January 2026:**
- ✅ Frontend callout system integration
- ✅ Mode detection UI components
- ✅ Gate blocking with Critical acknowledgments
- ✅ Auto-refresh status bar (5s polling)
- ✅ 187 tests passing (68 governance, 119 other)
- ✅ Production-ready desktop app

**Recent Milestones:**
- Metrics redesign (FIX-021 through FIX-027)
- Mode-aware threshold scaling
- Deterministic PCI checklist
- Immutable steno-ledger audit trail

---

## Troubleshooting

### API Key Not Set

**Error**: `ANTHROPIC_API_KEY environment variable not set`

**Solution**:
```bash
# Set the environment variable before running the app
export ANTHROPIC_API_KEY="your-api-key-here"  # macOS/Linux
$env:ANTHROPIC_API_KEY="your-api-key-here"    # Windows PowerShell
```

### Build Fails

**Error**: `error: failed to compile method-vi`

**Solution**:
1. Verify Rust version: `rustc --version` (need 1.70.0+)
2. Clean build cache: `cd src-tauri && cargo clean`
3. Rebuild: `cargo build`

### Tests Fail

**Error**: Tests fail with database or API errors

**Solution**:
1. Ensure clean build: `cargo clean && cargo build`
2. Check API key is set for integration tests
3. Run with logs: `cargo test -- --nocapture`
4. Check individual test files in `src-tauri/tests/`

### Frontend Won't Load

**Error**: `Failed to fetch` or blank screen

**Solution**:
1. Check Node version: `node --version` (need v18.0.0+)
2. Reinstall dependencies: `rm -rf node_modules && npm install`
3. Clear Vite cache: `rm -rf .vite`
4. Restart dev server: `npm run tauri dev`

### Pre-existing TypeScript Errors

**Note**: There are 3 pre-existing TypeScript errors in `MetricsDashboard.tsx` unrelated to Phase 5 work. These don't prevent the app from running in development mode (`npm run tauri dev` works fine).

---

## Contributing

Method-VI is currently in active development. If you encounter issues:

1. Check the troubleshooting section above
2. Review architecture documentation for system understanding
3. Run the test suite to verify your environment
4. Open an issue on GitHub with:
   - Your environment (OS, Node, Rust versions)
   - Steps to reproduce
   - Expected vs. actual behavior
   - Relevant logs or screenshots

---

## License

Proprietary - All rights reserved

---

## Links

- **GitHub Repository**: https://github.com/Merlins-Owl/Method-VI
- **Anthropic Claude API**: https://console.anthropic.com/
- **Tauri Documentation**: https://v2.tauri.app/

---

**Built with Tauri v2, React 19, Rust 1.92, and Claude Sonnet 4.5**
