import React, { useEffect, useState } from 'react';
import type { ModeInfo } from '../types/callouts';
import { MODE_COLORS } from '../types/callouts';
import { calloutApi } from '../utils/calloutApi';

interface ModeBadgeProps {
  className?: string;
  showDetails?: boolean;
}

export const ModeBadge: React.FC<ModeBadgeProps> = ({
  className = '',
  showDetails = false
}) => {
  const [modeInfo, setModeInfo] = useState<ModeInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [expanded, setExpanded] = useState(false);

  useEffect(() => {
    const fetchMode = async () => {
      try {
        const info = await calloutApi.getCurrentMode();
        setModeInfo(info);
      } catch (error) {
        console.error('Failed to fetch mode:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchMode();
    const interval = setInterval(fetchMode, 5000);
    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return (
      <div className={`px-3 py-1.5 rounded-full bg-gray-800 border border-gray-700 ${className}`}>
        <span className="text-sm text-gray-400">Loading...</span>
      </div>
    );
  }

  if (!modeInfo?.mode) {
    return (
      <div
        className={`px-3 py-1.5 rounded-full bg-gray-800 border border-gray-700 ${className}`}
        title="Mode is determined after Step 2 baseline analysis. Keep working!"
      >
        <span className="text-sm text-gray-400">Mode: Detecting...</span>
      </div>
    );
  }

  const colors = MODE_COLORS[modeInfo.mode];
  const confidencePercent = modeInfo.confidence !== null && modeInfo.confidence !== undefined
    ? Math.round(modeInfo.confidence * 100)
    : null;

  return (
    <div className={`relative ${className}`}>
      <div
        className={`flex items-center gap-2 px-3 py-1.5 rounded-full ${colors.bg} border border-gray-600 cursor-pointer hover:opacity-80 transition-opacity`}
        onClick={() => showDetails && setExpanded(!expanded)}
      >
        <div className={`w-2 h-2 rounded-full ${colors.text.replace('text-', 'bg-')}`} />
        <span className={`text-sm font-medium ${colors.text}`}>
          {modeInfo.mode}
        </span>
        {confidencePercent !== null && (
          <span className="text-xs text-gray-400">
            ({confidencePercent}%)
          </span>
        )}
        {modeInfo.is_locked && (
          <span
            className="text-xs text-gray-500 cursor-help"
            title="Mode is fixed for this run. You can continue editing."
          >
            🔒
          </span>
        )}
      </div>

      {showDetails && expanded && (
        <div className="absolute top-full left-0 mt-2 p-3 bg-gray-800 border border-gray-700 rounded-lg shadow-lg min-w-[250px] z-40">
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-400">Mode:</span>
              <span className={colors.text}>
                {modeInfo.display_name ?? modeInfo.mode}
              </span>
            </div>
            {modeInfo.ci_baseline !== null && modeInfo.ci_baseline !== undefined && (
              <div className="flex justify-between">
                <span className="text-gray-400">CI Baseline:</span>
                <span className="text-white">{modeInfo.ci_baseline.toFixed(2)}</span>
              </div>
            )}
            {confidencePercent !== null && (
              <div className="flex justify-between">
                <span className="text-gray-400">Confidence:</span>
                <span className="text-white">{confidencePercent}%</span>
              </div>
            )}
            <div className="flex justify-between">
              <span className="text-gray-400">Status:</span>
              <span className="text-white">
                {modeInfo.is_locked ? 'Locked for run' : 'Detecting...'}
              </span>
            </div>
            {modeInfo.user_message && (
              <div className="pt-2 border-t border-gray-700">
                <p className="text-gray-300 text-xs">{modeInfo.user_message}</p>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};

export default ModeBadge;
