import React, { useState, useEffect } from 'react';
import type { Callout, CalloutSummary } from '../types/callouts';
import { calloutApi } from '../utils/calloutApi';

interface CalloutBadgeProps {
  summary: CalloutSummary | null;
  loading?: boolean;
}

export const CalloutBadge: React.FC<CalloutBadgeProps> = ({ summary, loading }) => {
  const [showPanel, setShowPanel] = useState(false);
  const [callouts, setCallouts] = useState<Callout[]>([]);
  const [panelLoading, setPanelLoading] = useState(false);
  const [acknowledging, setAcknowledging] = useState<string | null>(null);

  // Escape key closes panel
  useEffect(() => {
    if (!showPanel) return;

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setShowPanel(false);
      }
    };

    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [showPanel]);

  // Fetch callouts when panel opens
  useEffect(() => {
    if (showPanel) {
      fetchCallouts();
    }
  }, [showPanel]);

  const fetchCallouts = async () => {
    setPanelLoading(true);
    try {
      const calloutData = await calloutApi.getAllCallouts();
      setCallouts(calloutData);
    } catch (error) {
      console.error('Failed to fetch callouts:', error);
    } finally {
      setPanelLoading(false);
    }
  };

  const handleAcknowledge = async (calloutId: string) => {
    setAcknowledging(calloutId);
    try {
      await calloutApi.acknowledgeCallout(calloutId, 'User acknowledged');
      await fetchCallouts();
    } catch (error) {
      console.error('Failed to acknowledge callout:', error);
    } finally {
      setAcknowledging(null);
    }
  };

  const handleAcknowledgeAll = async () => {
    setAcknowledging('all');
    try {
      await calloutApi.acknowledgeAllCallouts('User acknowledged all');
      await fetchCallouts();
    } catch (error) {
      console.error('Failed to acknowledge all callouts:', error);
    } finally {
      setAcknowledging(null);
    }
  };

  // Loading state for badge
  if (loading) {
    return (
      <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-gray-800 border border-gray-700">
        <div className="w-2 h-2 rounded-full bg-gray-500 animate-pulse" />
        <span className="text-sm text-gray-400">Loading...</span>
      </div>
    );
  }

  // Determine badge styling based on severity
  const getBadgeContent = () => {
    // No summary or no callouts - grey state
    if (!summary || summary.total === 0) {
      return {
        borderClass: 'border-gray-700',
        dot: <div className="w-2 h-2 rounded-full bg-gray-500" />,
        text: <span className="text-sm text-gray-400">—</span>,
        pending: null
      };
    }

    const hasCritical = summary.by_tier.critical > 0;
    const hasWarning = summary.by_tier.warning > 0;
    const hasAttention = summary.by_tier.attention > 0;
    const hasPending = summary.pending_acknowledgments > 0;

    const countText = (
      <span className="text-sm text-gray-200">
        {summary.total} Callout{summary.total !== 1 ? 's' : ''}
      </span>
    );

    const pendingText = hasPending ? (
      <span className="text-sm text-gray-400">• {summary.pending_acknowledgments} pending</span>
    ) : null;

    if (hasCritical) {
      return {
        borderClass: 'border-red-600',
        dot: <span className="w-2 h-2 rounded-full bg-red-500" />,
        text: countText,
        pending: pendingText
      };
    }

    if (hasWarning) {
      return {
        borderClass: 'border-orange-600',
        dot: <span className="w-2 h-2 rounded-full bg-orange-500" />,
        text: countText,
        pending: pendingText
      };
    }

    if (hasAttention) {
      return {
        borderClass: 'border-yellow-600',
        dot: <span className="w-2 h-2 rounded-full bg-yellow-500" />,
        text: countText,
        pending: pendingText
      };
    }

    // Info only - grey (no severity dot needed)
    return {
      borderClass: 'border-gray-600',
      dot: null,
      text: countText,
      pending: pendingText
    };
  };

  // Get panel title based on state
  const getPanelTitle = () => {
    if (panelLoading) return 'Callouts';
    if (!summary || summary.total === 0) return 'No Callouts ✓';

    const pendingCount = summary.pending_acknowledgments;
    if (pendingCount > 0) {
      return (
        <div>
          <div className="text-lg font-semibold text-white">Callouts ({summary.total})</div>
          <div className="text-xs text-gray-400">{pendingCount} pending</div>
        </div>
      );
    }

    return `Callouts (${summary.total})`;
  };

  // Render panel content based on state
  const renderPanelContent = () => {
    if (panelLoading) {
      return <div className="text-gray-400 py-8 text-center">Loading...</div>;
    }

    // No active run or before Step 2
    if (!summary || (summary.total === 0 && summary.can_proceed)) {
      return (
        <div className="text-gray-300 space-y-3">
          <p className="text-sm">
            Callouts track metric health and alert you to potential issues.
          </p>
          <p className="text-sm">
            They appear starting at Step 2 when governance metrics are calculated.
          </p>
          <div className="text-sm space-y-2">
            <p className="font-medium text-gray-200">Severity Tiers:</p>
            <ul className="space-y-1 text-xs">
              <li className="flex items-start gap-2">
                <span className="text-red-400">●</span>
                <span><span className="font-medium text-red-400">Critical</span> - Must acknowledge before proceeding</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-orange-400">●</span>
                <span><span className="font-medium text-orange-400">Warning</span> - Review recommended</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-yellow-400">●</span>
                <span><span className="font-medium text-yellow-400">Attention</span> - Minor concern</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-gray-400">●</span>
                <span><span className="font-medium text-gray-400">Info</span> - Informational only</span>
              </li>
            </ul>
          </div>
        </div>
      );
    }

    // No callouts at Steps 2+
    if (summary.total === 0) {
      return (
        <div className="text-center py-8">
          <p className="text-gray-300">All metrics are within acceptable ranges.</p>
        </div>
      );
    }

    // Has callouts - group by tier
    const criticalCallouts = callouts.filter(c => c.tier === 'Critical');
    const warningCallouts = callouts.filter(c => c.tier === 'Warning');
    const attentionCallouts = callouts.filter(c => c.tier === 'Attention');
    const infoCallouts = callouts.filter(c => c.tier === 'Info');

    return (
      <div className="space-y-4">
        {criticalCallouts.length > 0 && (
          <CalloutSection
            title="Must Review"
            callouts={criticalCallouts}
            color="red"
            onAcknowledge={handleAcknowledge}
            acknowledging={acknowledging}
          />
        )}

        {warningCallouts.length > 0 && (
          <CalloutSection
            title="Important"
            callouts={warningCallouts}
            color="orange"
            onAcknowledge={handleAcknowledge}
            acknowledging={acknowledging}
          />
        )}

        {attentionCallouts.length > 0 && (
          <CalloutSection
            title="Attention"
            callouts={attentionCallouts}
            color="yellow"
            onAcknowledge={handleAcknowledge}
            acknowledging={acknowledging}
          />
        )}

        {infoCallouts.length > 0 && (
          <CalloutSection
            title="Informational"
            callouts={infoCallouts}
            color="gray"
            onAcknowledge={handleAcknowledge}
            acknowledging={acknowledging}
          />
        )}

        {/* Acknowledge All button if multiple pending */}
        {summary && summary.pending_acknowledgments > 1 && (
          <AcknowledgeAllButton
            onAcknowledgeAll={handleAcknowledgeAll}
            acknowledging={acknowledging === 'all'}
          />
        )}
      </div>
    );
  };

  const badge = getBadgeContent();

  return (
    <div className="relative">
      {/* Badge */}
      <div
        className={`flex items-center gap-2 px-3 py-1.5 rounded-full bg-gray-800 ${badge.borderClass} cursor-pointer hover:bg-gray-700 transition-colors`}
        onClick={() => setShowPanel(!showPanel)}
      >
        {badge.dot}
        {badge.text}
        {badge.pending}
      </div>

      {/* Panel (when open) */}
      {showPanel && (
        <>
          {/* Backdrop - fixed to cover whole screen */}
          <div
            className="fixed inset-0 z-40 bg-black/50"
            onClick={() => setShowPanel(false)}
          />

          {/* Panel - positioned relative to badge */}
          <div
            className="absolute top-full right-0 mt-2 z-50 bg-gray-900 border border-gray-700 rounded-lg shadow-2xl p-4 min-w-80 max-w-md max-h-[80vh] overflow-y-auto"
            onClick={(e) => e.stopPropagation()}
          >
            {/* Header */}
            <div className="flex justify-between items-start mb-4">
              <div className="flex-1">{getPanelTitle()}</div>
              <button
                onClick={() => setShowPanel(false)}
                className="text-gray-400 hover:text-white text-xl leading-none ml-4"
              >
                ✕
              </button>
            </div>

            {/* Content */}
            {renderPanelContent()}
          </div>
        </>
      )}
    </div>
  );
};

// Callout section component
interface CalloutSectionProps {
  title: string;
  callouts: Callout[];
  color: 'red' | 'orange' | 'yellow' | 'gray';
  onAcknowledge: (id: string) => void;
  acknowledging: string | null;
}

const CalloutSection: React.FC<CalloutSectionProps> = ({
  title,
  callouts,
  color,
  onAcknowledge,
  acknowledging,
}) => {
  const headerClass =
    color === 'red' ? 'text-red-400' :
    color === 'orange' ? 'text-orange-400' :
    color === 'yellow' ? 'text-yellow-400' :
    'text-gray-400';

  return (
    <div>
      <h4 className={`text-sm font-medium ${headerClass} mb-2`}>
        {title} ({callouts.length})
      </h4>
      <div className="space-y-2">
        {callouts.map(callout => (
          <CalloutCard
            key={callout.id}
            callout={callout}
            color={color}
            onAcknowledge={() => onAcknowledge(callout.id)}
            acknowledging={acknowledging === callout.id}
          />
        ))}
      </div>
    </div>
  );
};

// Individual callout card
interface CalloutCardProps {
  callout: Callout;
  color: 'red' | 'orange' | 'yellow' | 'gray';
  onAcknowledge: () => void;
  acknowledging: boolean;
}

const CalloutCard: React.FC<CalloutCardProps> = ({
  callout,
  color,
  onAcknowledge,
  acknowledging,
}) => {
  const [confirmed, setConfirmed] = useState(false);
  const needsAcknowledge = callout.requires_acknowledgment && !callout.acknowledged;

  const bgClass =
    color === 'red' ? 'bg-red-900/30' :
    color === 'orange' ? 'bg-orange-900/30' :
    color === 'yellow' ? 'bg-yellow-900/30' :
    'bg-gray-800';

  const borderClass =
    color === 'red' ? 'border-red-700' :
    color === 'orange' ? 'border-orange-700' :
    color === 'yellow' ? 'border-yellow-700' :
    'border-gray-600';

  return (
    <div className={`${bgClass} border ${borderClass} rounded p-3 space-y-2`}>
      <div className="font-medium text-white text-sm">
        {callout.metric_name}
      </div>

      <div className="text-xs text-gray-400">
        Value: {callout.current_value.toFixed(2)}
        {callout.delta !== null && callout.delta !== undefined && (
          <span className="ml-2">
            Change: {callout.delta >= 0 ? '+' : ''}{callout.delta.toFixed(2)}
          </span>
        )}
      </div>

      <p className="text-sm text-gray-300">{callout.explanation}</p>

      {callout.recommendation && (
        <p className="text-xs text-gray-400">💡 {callout.recommendation}</p>
      )}

      {needsAcknowledge && (
        <div className="pt-2 space-y-2">
          <label className="flex items-center gap-2 text-xs text-gray-400 cursor-pointer">
            <input
              type="checkbox"
              checked={confirmed}
              onChange={(e) => setConfirmed(e.target.checked)}
              className="w-3 h-3 rounded"
            />
            I understand the risk
          </label>
          <button
            onClick={onAcknowledge}
            disabled={!confirmed || acknowledging}
            className="w-full px-3 py-1.5 bg-red-600 hover:bg-red-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white text-sm rounded transition-colors"
          >
            {acknowledging ? 'Acknowledging...' : 'Acknowledge'}
          </button>
        </div>
      )}

      {callout.acknowledged && (
        <div className="text-green-400 text-sm">✓ Acknowledged</div>
      )}
    </div>
  );
};

// Acknowledge All button
interface AcknowledgeAllButtonProps {
  onAcknowledgeAll: () => void;
  acknowledging: boolean;
}

const AcknowledgeAllButton: React.FC<AcknowledgeAllButtonProps> = ({
  onAcknowledgeAll,
  acknowledging,
}) => {
  const [confirmed, setConfirmed] = useState(false);

  return (
    <div className="pt-4 border-t border-gray-700 space-y-3">
      <label className="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
        <input
          type="checkbox"
          checked={confirmed}
          onChange={(e) => setConfirmed(e.target.checked)}
          className="w-4 h-4 rounded"
        />
        I have reviewed all concerns and understand the risks
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

export default CalloutBadge;
