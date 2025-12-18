/**
 * Metrics types following the Metric Explainability Contract
 * From specs/module-plan-method-vi.md (line 2971)
 */

/**
 * Metric input - a value that contributed to the metric calculation
 */
export interface MetricInput {
  name: string;
  value: number | string | boolean;
  source: string; // Which artifact/step provided this
}

/**
 * Threshold values for a metric
 */
export interface MetricThreshold {
  pass: number;
  warning: number | null;
  halt: number | null;
}

/**
 * Metric status based on threshold comparison
 */
export type MetricStatus = 'pass' | 'warning' | 'fail';

/**
 * Complete metric result with explainability data
 */
export interface MetricResult {
  metric_name: 'CI' | 'EV' | 'IAS' | 'EFI' | 'SEC' | 'PCI';
  value: number;
  threshold: MetricThreshold;
  status: MetricStatus;
  inputs_used: MetricInput[];
  calculation_method: string;
  interpretation: string;
  recommendation: string | null; // What to do if out of band (null if passing)
}

/**
 * Metric metadata for display
 */
export interface MetricMetadata {
  name: 'CI' | 'EV' | 'IAS' | 'EFI' | 'SEC' | 'PCI';
  fullName: string;
  description: string;
  unit: string; // e.g., "%", "points", "score"
  inverseScale: boolean; // true if lower is better (like EV)
}

/**
 * Complete metrics state
 */
export interface MetricsState {
  ci: MetricResult | null;
  ev: MetricResult | null;
  ias: MetricResult | null;
  efi: MetricResult | null;
  sec: MetricResult | null;
  pci: MetricResult | null;
}

/**
 * Metric history entry for charting
 */
export interface MetricHistoryEntry {
  step: number;
  timestamp: string;
  metrics: MetricsState;
}

/**
 * Threshold configuration (from thresholds.json)
 */
export interface ThresholdConfig {
  version: string;
  source: string;
  critical_6: {
    CI: MetricThreshold;
    EV: MetricThreshold;
    IAS: MetricThreshold;
    EFI: MetricThreshold;
    SEC: MetricThreshold;
    PCI: MetricThreshold;
  };
  advisory_5?: {
    GLR: { warning: number };
    RCC: { warning: number };
    CAI: { warning: number };
    RUV: { warning: number };
    LLE: { warning: number };
  };
}

/**
 * Metadata for all 6 critical metrics
 */
export const METRIC_METADATA: Record<string, MetricMetadata> = {
  CI: {
    name: 'CI',
    fullName: 'Coherence Index',
    description: 'Overall clarity and consistency of content',
    unit: 'score',
    inverseScale: false,
  },
  EV: {
    name: 'EV',
    fullName: 'Expansion Variance',
    description: 'Deviation from baseline scope',
    unit: 'points',
    inverseScale: true, // Lower is better
  },
  IAS: {
    name: 'IAS',
    fullName: 'Intent Alignment Score',
    description: 'Alignment with original intent',
    unit: 'score',
    inverseScale: false,
  },
  EFI: {
    name: 'EFI',
    fullName: 'Execution Fidelity Index',
    description: 'Adherence to Method-VI process',
    unit: '%',
    inverseScale: false,
  },
  SEC: {
    name: 'SEC',
    fullName: 'Scope Expansion Count',
    description: 'Number of approved scope expansions',
    unit: 'count',
    inverseScale: false,
  },
  PCI: {
    name: 'PCI',
    fullName: 'Process Compliance Index',
    description: 'Conformance to governance rules',
    unit: 'score',
    inverseScale: false,
  },
};

/**
 * Default thresholds from Method-VI Core
 * From specs/module-plan-method-vi.md (line 3119)
 */
export const DEFAULT_THRESHOLDS: ThresholdConfig = {
  version: '1.0.0',
  source: 'Method-VI Core v1.0.1',
  critical_6: {
    CI: { pass: 0.80, warning: 0.70, halt: 0.50 },
    EV: { pass: 10, warning: 20, halt: 30 },
    IAS: { pass: 0.80, warning: 0.70, halt: 0.50 },
    EFI: { pass: 95, warning: 90, halt: 80 },
    SEC: { pass: 100, warning: null, halt: null },
    PCI: { pass: 0.90, warning: 0.85, halt: 0.70 },
  },
};

/**
 * Calculate metric status based on value and thresholds
 */
export function calculateMetricStatus(
  value: number,
  threshold: MetricThreshold,
  inverseScale: boolean = false
): MetricStatus {
  const { pass, warning, halt } = threshold;

  if (inverseScale) {
    // For metrics like EV where lower is better
    if (value <= pass) return 'pass';
    if (warning !== null && value <= warning) return 'warning';
    return 'fail';
  } else {
    // For metrics where higher is better
    if (value >= pass) return 'pass';
    if (warning !== null && value >= warning) return 'warning';
    return 'fail';
  }
}

/**
 * Get color for metric status
 */
export function getStatusColor(status: MetricStatus): string {
  switch (status) {
    case 'pass':
      return 'text-green-500';
    case 'warning':
      return 'text-yellow-500';
    case 'fail':
      return 'text-red-500';
  }
}

/**
 * Get background color for metric status
 */
export function getStatusBgColor(status: MetricStatus): string {
  switch (status) {
    case 'pass':
      return 'bg-green-500/20 border-green-500';
    case 'warning':
      return 'bg-yellow-500/20 border-yellow-500';
    case 'fail':
      return 'bg-red-500/20 border-red-500';
  }
}

/**
 * Format metric value for display
 */
export function formatMetricValue(value: number, unit: string): string {
  switch (unit) {
    case '%':
      return `${Math.round(value)}%`;
    case 'score':
      return value.toFixed(2);
    case 'points':
    case 'count':
      return Math.round(value).toString();
    default:
      return value.toString();
  }
}
