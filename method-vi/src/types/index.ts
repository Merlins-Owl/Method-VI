// Method-VI Type Definitions

export interface RunContext {
  run_id: string;
  step: number;
  role: Role;
  ci: number | null;
  ev: number | null;
  mode: Mode;
  signal: Signal;
}

export type Role =
  | 'Observer'
  | 'Conductor'
  | 'Auditor'
  | 'Patcher'
  | 'Fabricator'
  | 'Examiner'
  | 'Curator'
  | 'Archivist';

export type Mode =
  | 'Standard'
  | 'Component'
  | 'Surgical';

export type Signal =
  | 'Initializing'
  | 'Active'
  | 'Awaiting_Gate'
  | 'Completed'
  | 'Halted'
  | 'Paused';

export type RunState =
  | { type: 'Step0Active' }
  | { type: 'Step0GatePending' }
  | { type: 'Step1Active' }
  | { type: 'Step1GatePending' }
  | { type: 'FutureStep'; step: number }
  | { type: 'Completed' }
  | { type: 'Halted'; reason: string };

/**
 * @deprecated Use MetricsState from './metrics' for new code
 * Legacy simple metrics interface
 */
export interface Metrics {
  ci: number | null;  // Coherence Index
  ev: number | null;  // Expected Value
  ias: number | null; // Intent Alignment Score
  efi: number | null; // Efficacy Index
  sec: number | null; // Scope Elasticity Compliance
  pci: number | null; // Pattern Confidence Index
}

// Export new metrics types
export * from './metrics';

export interface Step {
  number: number;
  name: string;
  description: string;
  role: Role;
  isGateStep: boolean;
}

export const STEPS: Step[] = [
  { number: 0, name: 'Intent Capture', description: 'Capture user intent and query patterns', role: 'Observer', isGateStep: true },
  { number: 1, name: 'Charter & Baseline', description: 'Create charter and freeze baseline', role: 'Conductor', isGateStep: true },
  { number: 2, name: 'Governance Setup', description: 'Establish metrics and compliance', role: 'Auditor', isGateStep: false },
  { number: 3, name: 'Analysis', description: 'Multi-lens diagnostic analysis', role: 'Examiner', isGateStep: false },
  { number: 4, name: 'Synthesis', description: 'Generate actionable recommendations', role: 'Curator', isGateStep: false },
  { number: 5, name: 'Implementation', description: 'Execute changes with precision', role: 'Patcher', isGateStep: false },
  { number: 6, name: 'Validation', description: 'Verify outcomes and capture learnings', role: 'Examiner', isGateStep: false },
  { number: 6.5, name: 'Closure', description: 'Archive and extract patterns', role: 'Archivist', isGateStep: false },
];
