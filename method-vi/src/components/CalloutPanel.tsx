import React, { useEffect, useState } from 'react';
import type { Callout, CalloutSummary, CalloutTier } from '../types/callouts';
import { TIER_COLORS } from '../types/callouts';
import { calloutApi } from '../utils/calloutApi';

interface CalloutPanelProps {
  isOpen: boolean;
  onClose: () => void;
  onAcknowledge?: () => void;
}

const TIER_ORDER: CalloutTier[] = ['Critical', 'Warning', 'Attention', 'Info'];

export const CalloutPanel: React.FC<CalloutPanelProps> = ({
  isOpen,
  onClose,
  onAcknowledge
}) => {
  const [callouts, setCallouts] = useState<Callout[]>([]);
  const [summary, setSummary] = useState<CalloutSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [acknowledging, setAcknowledging] = useState<string | null>(null);

  useEffect(() => {
    if (isOpen) {
      fetchCallouts();
    }
  }, [isOpen]);

  useEffect(() => {
    const handleEsc = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        onClose();
      }
    };
    window.addEventListener('keydown', handleEsc);
    return () => window.removeEventListener('keydown', handleEsc);
  }, [isOpen, onClose]);

  const fetchCallouts = async () => {
    setLoading(true);
    try {
      const [calloutData, summaryData] = await Promise.all([
        calloutApi.getAllCallouts(),
        calloutApi.getCalloutSummary(),
      ]);
      setCallouts(calloutData);
      setSummary(summaryData);
    } catch (error) {
      console.error('Failed to fetch callouts:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleAcknowledge = async (calloutId: string) => {
    setAcknowledging(calloutId);
    try {
      await calloutApi.acknowledgeCallout(calloutId, 'User acknowledged after confirmation');
      await fetchCallouts();
      onAcknowledge?.();
    } catch (error) {
      console.error('Failed to acknowledge callout:', error);
    } finally {
      setAcknowledging(null);
    }
  };

  const handleAcknowledgeAll = async () => {
    setAcknowledging('all');
    try {
      await calloutApi.acknowledgeAllCallouts('User acknowledged all after confirmation');
      await fetchCallouts();
      onAcknowledge?.();
    } catch (error) {
      console.error('Failed to acknowledge all callouts:', error);
    } finally {
      setAcknowledging(null);
    }
  };

  if (!isOpen) return null;

  const groupedCallouts = TIER_ORDER.reduce((acc, tier) => {
    acc[tier] = callouts.filter(c => c.tier === tier);
    return acc;
  }, {} as Record<CalloutTier, Callout[]>);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onClose}
      />

      <div className="relative bg-gray-900 border border-gray-700 rounded-lg shadow-xl w-full max-w-2xl max-h-[80vh] overflow-hidden">
        <div className="flex items-center justify-between p-4 border-b border-gray-700">
          <h2 className="text-lg font-semibold text-white">
            Callouts {summary && `(${summary.total})`}
          </h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white transition-colors"
          >
            ✕
          </button>
        </div>

        <div className="p-4 overflow-y-auto max-h-[60vh]">
          {loading ? (
            <div className="text-center text-gray-400 py-8">Loading callouts...</div>
          ) : callouts.length === 0 ? (
            <div className="text-center text-gray-400 py-8">No callouts to display</div>
          ) : (
            <div className="space-y-6">
              {TIER_ORDER.map(tier => {
                const tierCallouts = groupedCallouts[tier];
                if (tierCallouts.length === 0) return null;

                const colors = TIER_COLORS[tier];
                return (
                  <div key={tier}>
                    <h3 className={`text-sm font-medium ${colors.text} mb-2`}>
                      {colors.label} ({tierCallouts.length})
                    </h3>
                    <div className="space-y-2">
                      {tierCallouts.map(callout => (
                        <CalloutItem
                          key={callout.id}
                          callout={callout}
                          onAcknowledge={() => handleAcknowledge(callout.id)}
                          acknowledging={acknowledging === callout.id}
                        />
                      ))}
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {summary && summary.pending_acknowledgments > 0 && (
          <AcknowledgeAllFooter
            pendingCount={summary.pending_acknowledgments}
            onAcknowledgeAll={handleAcknowledgeAll}
            acknowledging={acknowledging === 'all'}
          />
        )}
      </div>
    </div>
  );
};

interface AcknowledgeAllFooterProps {
  pendingCount: number;
  onAcknowledgeAll: () => void;
  acknowledging: boolean;
}

const AcknowledgeAllFooter: React.FC<AcknowledgeAllFooterProps> = ({
  pendingCount,
  onAcknowledgeAll,
  acknowledging,
}) => {
  const [confirmed, setConfirmed] = useState(false);

  useEffect(() => {
    setConfirmed(false);
  }, [pendingCount]);

  return (
    <div className="p-4 border-t border-gray-700 bg-gray-800/50 space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-sm text-gray-400">
          {pendingCount} callout{pendingCount !== 1 ? 's' : ''} pending acknowledgment
        </span>
      </div>

      <label className="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
        <input
          type="checkbox"
          checked={confirmed}
          onChange={(e) => setConfirmed(e.target.checked)}
          className="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-500 focus:ring-blue-500 focus:ring-offset-gray-800"
        />
        I have reviewed these concerns and understand the risks
      </label>

      <button
        onClick={onAcknowledgeAll}
        disabled={!confirmed || acknowledging}
        className="w-full px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white rounded transition-colors"
      >
        {acknowledging ? 'Acknowledging...' : 'Acknowledge All'}
      </button>
    </div>
  );
};

interface CalloutItemProps {
  callout: Callout;
  onAcknowledge: () => void;
  acknowledging: boolean;
}

const CalloutItem: React.FC<CalloutItemProps> = ({ callout, onAcknowledge, acknowledging }) => {
  const [confirmed, setConfirmed] = useState(false);
  const colors = TIER_COLORS[callout.tier];
  const needsAcknowledge = callout.requires_acknowledgment && !callout.acknowledged;

  return (
    <div className={`${colors.bg} border ${colors.border} rounded p-3`}>
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-1">
            <span className={`font-medium ${colors.text}`}>{callout.metric_name}</span>
          </div>
          {/* Use 'explanation' field (actual Rust field name) */}
          <p className="text-gray-300 text-sm">{callout.explanation}</p>
          {callout.recommendation && (
            <p className="text-gray-400 text-xs mt-1">💡 {callout.recommendation}</p>
          )}
          <div className="flex items-center gap-4 mt-2 text-xs text-gray-500">
            <span>Value: {callout.current_value.toFixed(2)}</span>
            {/* UX Guardrail: "Change:" instead of "Delta:" */}
            {callout.delta !== null && (
              <span>Change: {callout.delta >= 0 ? '+' : ''}{callout.delta.toFixed(2)}</span>
            )}
          </div>
        </div>

        {needsAcknowledge && (
          <div className="flex flex-col items-end gap-2">
            <label className="flex items-center gap-1.5 text-xs text-gray-400 cursor-pointer">
              <input
                type="checkbox"
                checked={confirmed}
                onChange={(e) => setConfirmed(e.target.checked)}
                className="w-3 h-3 rounded border-gray-600 bg-gray-700 text-red-500 focus:ring-red-500"
              />
              I understand
            </label>
            <button
              onClick={onAcknowledge}
              disabled={!confirmed || acknowledging}
              className="px-3 py-1 bg-red-600 hover:bg-red-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white text-sm rounded transition-colors whitespace-nowrap"
            >
              {acknowledging ? '...' : 'Acknowledge'}
            </button>
          </div>
        )}

        {callout.acknowledged && (
          <span className="text-green-400 text-sm">✓ Acknowledged</span>
        )}
      </div>
    </div>
  );
};

export default CalloutPanel;
