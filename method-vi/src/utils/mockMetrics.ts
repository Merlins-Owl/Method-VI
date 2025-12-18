/**
 * Mock metric data for testing the Metric Explainability Contract
 * These will be replaced with real calculations later
 */

import {
  MetricResult,
  MetricsState,
  DEFAULT_THRESHOLDS,
  calculateMetricStatus,
  METRIC_METADATA,
} from '../types/metrics';

/**
 * Generate a mock CI (Confidence Index) metric
 */
export function mockCI(value: number = 0.78): MetricResult {
  const threshold = DEFAULT_THRESHOLDS.critical_6.CI;
  const metadata = METRIC_METADATA.CI;
  const status = calculateMetricStatus(value, threshold, metadata.inverseScale);

  return {
    metric_name: 'CI',
    value,
    threshold,
    status,
    inputs_used: [
      {
        name: 'structural_coherence',
        value: 0.82,
        source: 'Step 3 Structural Lens',
      },
      {
        name: 'term_consistency',
        value: 0.74,
        source: 'Step 5 Header Report',
      },
    ],
    calculation_method: 'Weighted average of coherence dimensions',
    interpretation:
      'Content clarity is below target, primarily due to inconsistent terminology.',
    recommendation:
      value < threshold.pass
        ? 'Review Header Report and normalize terms before proceeding.'
        : null,
  };
}

/**
 * Generate a mock EV (Expected Value) metric
 */
export function mockEV(value: number = 12): MetricResult {
  const threshold = DEFAULT_THRESHOLDS.critical_6.EV;
  const metadata = METRIC_METADATA.EV;
  const status = calculateMetricStatus(value, threshold, metadata.inverseScale);

  return {
    metric_name: 'EV',
    value,
    threshold,
    status,
    inputs_used: [
      {
        name: 'baseline_complexity',
        value: 245,
        source: 'Step 1 Baseline Snapshot',
      },
      {
        name: 'charter_scope',
        value: 'medium',
        source: 'Step 1 Charter',
      },
      {
        name: 'structural_delta',
        value: 18,
        source: 'Step 3 Analysis',
      },
    ],
    calculation_method:
      'Predicted edit distance based on complexity and scope analysis',
    interpretation:
      'Expected Value is slightly above optimal range, indicating moderate complexity.',
    recommendation:
      value > threshold.pass
        ? 'Consider breaking down into smaller components or simplifying scope.'
        : null,
  };
}

/**
 * Generate a mock IAS (Intent Alignment Score) metric
 */
export function mockIAS(value: number = 0.85): MetricResult {
  const threshold = DEFAULT_THRESHOLDS.critical_6.IAS;
  const metadata = METRIC_METADATA.IAS;
  const status = calculateMetricStatus(value, threshold, metadata.inverseScale);

  return {
    metric_name: 'IAS',
    value,
    threshold,
    status,
    inputs_used: [
      {
        name: 'intent_anchor_match',
        value: 0.88,
        source: 'Step 0 Intent Summary',
      },
      {
        name: 'charter_alignment',
        value: 0.82,
        source: 'Step 1 Charter',
      },
    ],
    calculation_method:
      'Semantic similarity between intent anchor and current state',
    interpretation:
      'Strong alignment with original intent. Charter remains true to user goals.',
    recommendation: null,
  };
}

/**
 * Generate a mock EFI (Execution Fidelity Index) metric
 */
export function mockEFI(value: number = 96): MetricResult {
  const threshold = DEFAULT_THRESHOLDS.critical_6.EFI;
  const metadata = METRIC_METADATA.EFI;
  const status = calculateMetricStatus(value, threshold, metadata.inverseScale);

  return {
    metric_name: 'EFI',
    value,
    threshold,
    status,
    inputs_used: [
      {
        name: 'gates_passed',
        value: 3,
        source: 'Orchestrator Ledger',
      },
      {
        name: 'gates_total',
        value: 3,
        source: 'Step Tracker',
      },
      {
        name: 'interventions',
        value: 0,
        source: 'Governance Agent',
      },
    ],
    calculation_method:
      '(gates_passed / gates_total) * 100 - (interventions * 2)',
    interpretation:
      'Excellent adherence to Method-VI process with no interventions required.',
    recommendation: null,
  };
}

/**
 * Generate a mock SEC (Stakeholder Engagement Coefficient) metric
 */
export function mockSEC(value: number = 100): MetricResult {
  const threshold = DEFAULT_THRESHOLDS.critical_6.SEC;
  const metadata = METRIC_METADATA.SEC;
  const status = calculateMetricStatus(value, threshold, metadata.inverseScale);

  return {
    metric_name: 'SEC',
    value,
    threshold,
    status,
    inputs_used: [
      {
        name: 'gate_approvals',
        value: 3,
        source: 'Gate Protocol Log',
      },
      {
        name: 'gate_rejections',
        value: 0,
        source: 'Gate Protocol Log',
      },
      {
        name: 'clarifications_provided',
        value: 2,
        source: 'Step 0 Clarification Log',
      },
    ],
    calculation_method:
      'Ratio of engaged interactions to total decision points * 100',
    interpretation:
      'Perfect stakeholder engagement. All gates approved with thoughtful input.',
    recommendation: null,
  };
}

/**
 * Generate a mock PCI (Process Compliance Index) metric
 */
export function mockPCI(value: number = 0.92): MetricResult {
  const threshold = DEFAULT_THRESHOLDS.critical_6.PCI;
  const metadata = METRIC_METADATA.PCI;
  const status = calculateMetricStatus(value, threshold, metadata.inverseScale);

  return {
    metric_name: 'PCI',
    value,
    threshold,
    status,
    inputs_used: [
      {
        name: 'role_transitions',
        value: 3,
        source: 'Orchestrator State Machine',
      },
      {
        name: 'forbidden_actions',
        value: 0,
        source: 'Context Manager Violations',
      },
      {
        name: 'immutability_violations',
        value: 0,
        source: 'Ledger Validator',
      },
    ],
    calculation_method:
      '1.0 - (violations / total_checkpoints) with role transition bonuses',
    interpretation:
      'High compliance with governance rules. No violations detected.',
    recommendation: null,
  };
}

/**
 * Generate complete mock metrics state
 */
export function generateMockMetrics(overrides?: Partial<{
  ci: number;
  ev: number;
  ias: number;
  efi: number;
  sec: number;
  pci: number;
}>): MetricsState {
  return {
    ci: mockCI(overrides?.ci),
    ev: mockEV(overrides?.ev),
    ias: mockIAS(overrides?.ias),
    efi: mockEFI(overrides?.efi),
    sec: mockSEC(overrides?.sec),
    pci: mockPCI(overrides?.pci),
  };
}

/**
 * Generate mock metrics for different scenarios
 */
export const MOCK_SCENARIOS = {
  // All metrics passing
  allPass: generateMockMetrics({
    ci: 0.85,
    ev: 8,
    ias: 0.88,
    efi: 98,
    sec: 100,
    pci: 0.95,
  }),

  // Some warnings
  someWarnings: generateMockMetrics({
    ci: 0.75, // Warning
    ev: 18, // Warning
    ias: 0.85,
    efi: 92, // Warning
    sec: 100,
    pci: 0.92,
  }),

  // Some failures
  someFailures: generateMockMetrics({
    ci: 0.45, // Fail
    ev: 35, // Fail
    ias: 0.82,
    efi: 96,
    sec: 100,
    pci: 0.68, // Fail
  }),

  // Step 0 just started
  step0Start: {
    ci: null,
    ev: null,
    ias: mockIAS(0.88), // Only IAS available from intent analysis
    efi: null,
    sec: mockSEC(100), // User engaged with intent capture
    pci: null,
  },

  // Step 1 in progress
  step1Progress: {
    ci: null,
    ev: mockEV(12),
    ias: mockIAS(0.85),
    efi: mockEFI(95),
    sec: mockSEC(100),
    pci: mockPCI(0.90),
  },
};
