import { invoke } from '@tauri-apps/api/core';
import type { Callout, CalloutSummary, ModeInfo } from '../types/callouts';

export const calloutApi = {
  getAllCallouts: () => invoke<Callout[]>('get_all_callouts'),

  getPendingCallouts: () => invoke<Callout[]>('get_pending_callouts'),

  getCalloutSummary: () => invoke<CalloutSummary>('get_callout_summary'),

  canProceed: () => invoke<boolean>('can_proceed'),

  acknowledgeCallout: (calloutId: string, userConfirmation: string) =>
    invoke('acknowledge_callout', { calloutId, userConfirmation }),

  acknowledgeAllCallouts: (userConfirmation: string) =>
    invoke('acknowledge_all_callouts', { userConfirmation }),

  getCurrentMode: () => invoke<ModeInfo>('get_current_mode'),
};

export default calloutApi;
