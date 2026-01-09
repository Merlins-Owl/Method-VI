import React, { useEffect, useState } from 'react';
import type { ModeInfo } from '../types/callouts';
import { calloutApi } from '../utils/calloutApi';

interface ModeBadgeProps {
  showDetails?: boolean;
  className?: string;
}

const getModeColor = (mode: string | null) => {
  switch (mode) {
    case 'Architecting': return 'bg-purple-600';
    case 'Builder': return 'bg-blue-600';
    case 'Refining': return 'bg-green-600';
    default: return 'bg-gray-600';
  }
};

export const ModeBadge: React.FC<ModeBadgeProps> = ({ showDetails = false, className = '' }) => {
  const [modeInfo, setModeInfo] = useState<ModeInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [showPopover, setShowPopover] = useState(false);

  // Fetch mode info and poll every 5 seconds
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

  // Escape key closes popover
  useEffect(() => {
    if (!showPopover) return;

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setShowPopover(false);
      }
    };

    window.addEventListener('keydown', handleEscape);
    return () => window.removeEventListener('keydown', handleEscape);
  }, [showPopover]);

  if (loading) {
    return (
      <div className={`px-3 py-1.5 rounded-full bg-gray-600 text-white text-sm ${className}`}>
        Loading...
      </div>
    );
  }

  const mode = modeInfo?.mode ?? null;
  const colorClass = getModeColor(mode);
  const confidencePercent = modeInfo?.confidence !== null && modeInfo?.confidence !== undefined
    ? Math.round(modeInfo.confidence * 100)
    : null;

  return (
    <div className="relative">
      <div
        className={`px-3 py-1.5 rounded-full ${colorClass} text-white text-sm cursor-pointer hover:opacity-90 transition-opacity ${className}`}
        onClick={() => showDetails && setShowPopover(!showPopover)}
      >
        {mode ? (
          <span>
            {mode} {confidencePercent !== null && `(${confidencePercent}%)`}
          </span>
        ) : (
          <span>Detecting...</span>
        )}
      </div>

      {/* Popover */}
      {showDetails && showPopover && (
        <>
          {/* Backdrop - fixed to cover whole screen */}
          <div
            className="fixed inset-0 z-40 bg-black/50"
            onClick={() => setShowPopover(false)}
          />

          {/* Popover - positioned relative to badge */}
          <div
            className="absolute top-full left-0 mt-2 z-50 bg-gray-900 border border-gray-700 rounded-lg shadow-2xl p-4 w-auto min-w-64 max-w-sm"
            onClick={(e) => e.stopPropagation()}
          >
            {mode ? (
              // Mode detected - show details
              <div className="space-y-3 text-sm">
                <div className="text-lg font-semibold text-white border-b border-gray-700 pb-2">
                  {mode} Mode
                </div>

                {confidencePercent !== null && (
                  <div className="flex justify-between">
                    <span className="text-gray-400">Confidence:</span>
                    <span className="text-white">{confidencePercent}%</span>
                  </div>
                )}

                {modeInfo?.ci_baseline !== null && modeInfo?.ci_baseline !== undefined && (
                  <div className="flex justify-between">
                    <span className="text-gray-400">CI Baseline:</span>
                    <span className="text-white font-mono">{modeInfo.ci_baseline.toFixed(2)}</span>
                  </div>
                )}

                {modeInfo?.user_message && (
                  <p className="text-gray-300 text-xs leading-relaxed pt-2 border-t border-gray-700">
                    {modeInfo.user_message}
                  </p>
                )}

                <div className="pt-2 border-t border-gray-700">
                  <p className="font-medium text-gray-200 mb-2">Current Thresholds:</p>
                  <div className="space-y-1 text-xs text-gray-400">
                    {mode === 'Architecting' && (
                      <>
                        <div>CI Pass: 0.50 | Warn: 0.35</div>
                        <div>IAS Pass: 0.50 | Warn: 0.35</div>
                      </>
                    )}
                    {mode === 'Builder' && (
                      <>
                        <div>CI Pass: 0.65 | Warn: 0.50</div>
                        <div>IAS Pass: 0.65 | Warn: 0.50</div>
                      </>
                    )}
                    {mode === 'Refining' && (
                      <>
                        <div>CI Pass: 0.80 | Warn: 0.70</div>
                        <div>IAS Pass: 0.80 | Warn: 0.70</div>
                      </>
                    )}
                  </div>
                </div>
              </div>
            ) : (
              // No mode detected - show explanation
              <div className="space-y-3 text-sm">
                <div className="text-lg font-semibold text-white border-b border-gray-700 pb-2">
                  Mode Detection
                </div>

                <p className="text-gray-300 text-xs leading-relaxed">
                  Your structure mode will be detected at Step 2 based on Coherence Index.
                </p>

                <div>
                  <p className="font-medium text-gray-200 mb-2">Modes:</p>
                  <div className="space-y-2 text-xs">
                    <div>
                      <span className="font-medium text-purple-400">Architecting</span>
                      <span className="text-gray-400"> (CI &lt; 0.35)</span>
                      <p className="text-gray-500 mt-0.5">Early exploration, relaxed thresholds</p>
                    </div>
                    <div>
                      <span className="font-medium text-blue-400">Builder</span>
                      <span className="text-gray-400"> (CI 0.35 - 0.70)</span>
                      <p className="text-gray-500 mt-0.5">Active development, standard thresholds</p>
                    </div>
                    <div>
                      <span className="font-medium text-green-400">Refining</span>
                      <span className="text-gray-400"> (CI ≥ 0.70)</span>
                      <p className="text-gray-500 mt-0.5">Polish phase, elevated thresholds</p>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </>
      )}
    </div>
  );
};

export default ModeBadge;
