/**
 * Callout System Types
 * Mirrors backend types from src-tauri/src/governance/callouts.rs
 *
 * FFI Contract (verified via Task 0 discovery):
 * - Enums serialize as PascalCase strings
 * - Struct fields serialize as snake_case
 */

export type CalloutTier = 'Info' | 'Attention' | 'Warning' | 'Critical';

export type StructureMode = 'Architecting' | 'Builder' | 'Refining';

/**
 * Callout struct - matches Rust Callout exactly
 * Field names verified from src-tauri/src/governance/callouts.rs
 */
export interface Callout {
  id: string;
  tier: CalloutTier;
  original_tier: CalloutTier | null;
  metric_name: string;
  current_value: number;
  previous_value: number | null;
  delta: number | null;
  threshold_context: string;
  explanation: string;
  recommendation: string;
  requires_acknowledgment: boolean;
  acknowledged: boolean;
}

export interface CalloutCountByTier {
  info: number;
  attention: number;
  warning: number;
  critical: number;
}

export interface CalloutSummary {
  total: number;
  by_tier: CalloutCountByTier;
  pending_acknowledgments: number;
  can_proceed: boolean;
}

export interface ModeInfo {
  mode: StructureMode | null;
  display_name: string | null;
  user_message: string | null;
  ci_baseline: number | null;
  confidence: number | null;
  is_locked: boolean;
}

/**
 * Tier color and label mapping for Tailwind
 *
 * UX Guardrail: Labels use human-friendly terms instead of tier names
 * to prevent "Attention" vs "Warning" confusion.
 */
export const TIER_COLORS: Record<CalloutTier, {
  bg: string;
  border: string;
  text: string;
  label: string;
}> = {
  Info: {
    bg: 'bg-blue-900/30',
    border: 'border-blue-500',
    text: 'text-blue-400',
    label: 'Info'
  },
  Attention: {
    bg: 'bg-yellow-900/30',
    border: 'border-yellow-500',
    text: 'text-yellow-400',
    label: 'Minor'
  },
  Warning: {
    bg: 'bg-orange-900/30',
    border: 'border-orange-500',
    text: 'text-orange-400',
    label: 'Important'
  },
  Critical: {
    bg: 'bg-red-900/30',
    border: 'border-red-500',
    text: 'text-red-400',
    label: 'Must Review'
  },
};

export const MODE_COLORS: Record<StructureMode, { bg: string; text: string }> = {
  Architecting: { bg: 'bg-purple-900/30', text: 'text-purple-400' },
  Builder: { bg: 'bg-blue-900/30', text: 'text-blue-400' },
  Refining: { bg: 'bg-green-900/30', text: 'text-green-400' },
};

/**
 * Artifact fidelity level
 */
export type ArtifactFidelity = 'Draft' | 'Placeholder' | 'Final';

/**
 * Summary of an artifact for gate preview
 */
export interface ArtifactSummary {
  artifact_key: string;
  display_name: string;
  fidelity: ArtifactFidelity;
  preview_snippet?: string;
}

/**
 * Gate preview data - shows what was created and what's missing
 */
export interface GatePreview {
  step: number;
  artifacts_created: ArtifactSummary[];
  missing_required: string[];
  has_hard_blocks: boolean;
}

/**
 * Gate decision types
 */
export type GateDecision = 'approve' | 'request_changes' | 'start_over';
