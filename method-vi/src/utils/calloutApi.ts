import { invoke } from '@tauri-apps/api/core';
import type { Callout, CalloutSummary, ModeInfo, GatePreview, GateDecision } from '../types/callouts';

export const calloutApi = {
  getAllCallouts: () => invoke<Callout[]>('get_all_callouts'),

  getPendingCallouts: () => invoke<Callout[]>('get_pending_callouts'),

  getCalloutSummary: () => invoke<CalloutSummary>('get_callout_summary'),

  canProceed: () => invoke<boolean>('can_proceed'),

  acknowledgeCallout: (calloutId: string, userConfirmation: string) =>
    invoke('acknowledge_callout', { calloutId, confirmation: userConfirmation }),

  acknowledgeAllCallouts: (userConfirmation: string) =>
    invoke('acknowledge_all_callouts', { confirmation: userConfirmation }),

  getCurrentMode: () => invoke<ModeInfo>('get_current_mode'),

  // Gate preview and decision functions
  getGatePreview: (step: number) => invoke<GatePreview>('get_gate_preview', { step }),

  getHardBlocks: () => invoke<Callout[]>('get_hard_blocks'),

  submitGateDecision: (decision: GateDecision, feedback?: string) =>
    invoke('submit_gate_decision', { decision, feedback }),
};

export default calloutApi;
