import React from 'react';
import type { CalloutSummary, CalloutTier } from '../types/callouts';
import { TIER_COLORS } from '../types/callouts';

interface CalloutBadgeProps {
  summary: CalloutSummary | null;
  onClick?: () => void;
  loading?: boolean;
}

export const CalloutBadge: React.FC<CalloutBadgeProps> = ({
  summary,
  onClick,
  loading = false
}) => {
  if (loading) {
    return (
      <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-gray-800 border border-gray-700">
        <div className="w-2 h-2 rounded-full bg-gray-500 animate-pulse" />
        <span className="text-sm text-gray-400">Loading...</span>
      </div>
    );
  }

  if (!summary || summary.total === 0) {
    return (
      <div
        className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-gray-800 border border-gray-700 cursor-pointer hover:bg-gray-700 transition-colors"
        onClick={onClick}
      >
        <div className="w-2 h-2 rounded-full bg-green-500" />
        <span className="text-sm text-gray-300">No callouts</span>
      </div>
    );
  }

  const highestTier: CalloutTier = summary.by_tier.critical > 0 ? 'Critical'
    : summary.by_tier.warning > 0 ? 'Warning'
    : summary.by_tier.attention > 0 ? 'Attention'
    : 'Info';

  const colors = TIER_COLORS[highestTier];

  return (
    <div
      className={`flex items-center gap-2 px-3 py-1.5 rounded-full ${colors.bg} border ${colors.border} cursor-pointer hover:opacity-80 transition-opacity`}
      onClick={onClick}
    >
      <div className={`w-2 h-2 rounded-full ${colors.text.replace('text-', 'bg-')}`} />
      <span className={`text-sm font-medium ${colors.text}`}>
        {summary.total} callout{summary.total !== 1 ? 's' : ''}
      </span>
      {summary.pending_acknowledgments > 0 && (
        <span className="text-xs bg-red-600 text-white px-1.5 py-0.5 rounded-full">
          {summary.pending_acknowledgments} pending
        </span>
      )}
    </div>
  );
};

export default CalloutBadge;
