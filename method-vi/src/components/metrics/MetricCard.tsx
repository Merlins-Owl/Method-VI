import { useState } from 'react';
import {
  MetricResult,
  METRIC_METADATA,
  formatMetricValue,
} from '../../types/metrics';

interface MetricCardProps {
  metric: MetricResult;
  compact?: boolean;
  onExpand?: () => void;
}

// Status color configuration with explicit values
const STATUS_STYLES = {
  pass: {
    bg: 'rgba(34, 197, 94, 0.2)',      // green-500 at 20% opacity
    border: '#22c55e',                  // green-500
    text: '#4ade80',                    // green-400
    badgeBg: '#22c55e',                 // green-500
    badgeText: '#ffffff',
  },
  warning: {
    bg: 'rgba(234, 179, 8, 0.2)',       // yellow-500 at 20% opacity
    border: '#eab308',                  // yellow-500
    text: '#facc15',                    // yellow-400
    badgeBg: '#eab308',                 // yellow-500
    badgeText: '#000000',
  },
  fail: {
    bg: 'rgba(239, 68, 68, 0.2)',       // red-500 at 20% opacity
    border: '#ef4444',                  // red-500
    text: '#f87171',                    // red-400
    badgeBg: '#ef4444',                 // red-500
    badgeText: '#ffffff',
  },
  halt: {
    bg: 'rgba(239, 68, 68, 0.2)',       // red-500 at 20% opacity
    border: '#ef4444',                  // red-500
    text: '#f87171',                    // red-400
    badgeBg: '#ef4444',                 // red-500
    badgeText: '#ffffff',
  },
};

export default function MetricCard({ metric, compact = false, onExpand }: MetricCardProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const metadata = METRIC_METADATA[metric.metric_name];

  const handleToggle = () => {
    if (compact && onExpand) {
      onExpand();
    } else {
      setIsExpanded(!isExpanded);
    }
  };

  // Get styles based on status, defaulting to 'fail' for unknown statuses
  const statusKey = metric.status as keyof typeof STATUS_STYLES;
  const styles = STATUS_STYLES[statusKey] || STATUS_STYLES.fail;

  const renderCompactView = () => (
    <button
      onClick={handleToggle}
      className="px-3 py-2 rounded-lg border-2 transition-all hover:scale-105 cursor-pointer"
      style={{
        backgroundColor: styles.bg,
        borderColor: styles.border,
      }}
    >
      <div className="flex items-center gap-2">
        <div className="text-sm font-medium text-gray-200">{metric.metric_name}</div>
        <div className="text-lg font-bold" style={{ color: styles.text }}>
          {formatMetricValue(metric.value, metadata.unit)}
        </div>
      </div>
    </button>
  );

  const renderFullView = () => (
    <div
      className="rounded-lg border-2 p-4 transition-all"
      style={{
        backgroundColor: styles.bg,
        borderColor: styles.border,
      }}
    >
      {/* Header */}
      <div className="flex items-start justify-between mb-3">
        <div>
          <h3 className="text-xl font-bold text-white">
            {metric.metric_name} - {metadata.fullName}
          </h3>
          <p className="text-sm text-gray-300 mt-1">{metadata.description}</p>
        </div>
        <div className="text-right">
          <div className="text-3xl font-bold" style={{ color: styles.text }}>
            {formatMetricValue(metric.value, metadata.unit)}
          </div>
          <div className="text-xs text-gray-400 mt-1">
            Pass: {formatMetricValue(metric.threshold.pass, metadata.unit)}
          </div>
        </div>
      </div>

      {/* Status Indicator */}
      <div className="flex items-center gap-2 mb-3">
        <span
          className="px-2 py-1 rounded text-xs font-medium uppercase"
          style={{
            backgroundColor: styles.badgeBg,
            color: styles.badgeText,
          }}
        >
          {metric.status}
        </span>
      </div>

      {/* Threshold Indicator */}
      <div className="mb-4">
        <div className="flex items-center justify-between text-xs text-gray-400 mb-1">
          <span>
            {metadata.inverseScale ? 'Higher = Worse' : 'Lower = Worse'}
          </span>
          <span>
            {metadata.inverseScale ? 'Lower = Better' : 'Higher = Better'}
          </span>
        </div>
        <div className="relative h-2 bg-gray-700 rounded-full overflow-hidden">
          {/* Red zone (fail) */}
          <div
            className="absolute top-0 left-0 h-full"
            style={{
              backgroundColor: 'rgba(239, 68, 68, 0.3)',
              width: metadata.inverseScale
                ? '100%'
                : `${((metric.threshold.warning ?? metric.threshold.halt ?? 0) / (metadata.inverseScale ? metric.threshold.pass : 100)) * 100}%`,
            }}
          />
          {/* Yellow zone (warning) */}
          {metric.threshold.warning !== null && (
            <div
              className="absolute top-0 h-full"
              style={{
                backgroundColor: 'rgba(234, 179, 8, 0.3)',
                left: metadata.inverseScale
                  ? `${(metric.threshold.warning / (metric.threshold.halt ?? 100)) * 100}%`
                  : `${((metric.threshold.warning ?? 0) / 100) * 100}%`,
                width: metadata.inverseScale
                  ? `${((metric.threshold.pass - metric.threshold.warning) / (metric.threshold.halt ?? 100)) * 100}%`
                  : `${((metric.threshold.pass - (metric.threshold.warning ?? 0)) / 100) * 100}%`,
              }}
            />
          )}
          {/* Green zone (pass) */}
          <div
            className="absolute top-0 h-full"
            style={{
              backgroundColor: 'rgba(34, 197, 94, 0.3)',
              left: metadata.inverseScale
                ? '0%'
                : `${(metric.threshold.pass / 100) * 100}%`,
              width: metadata.inverseScale
                ? `${(metric.threshold.pass / (metric.threshold.halt ?? 100)) * 100}%`
                : `${((100 - metric.threshold.pass) / 100) * 100}%`,
            }}
          />
          {/* Current value marker */}
          <div
            className="absolute top-0 w-1 h-full"
            style={{
              backgroundColor: styles.text,
              left: `${(metric.value / (metadata.inverseScale ? (metric.threshold.halt ?? 100) : 100)) * 100}%`,
            }}
          />
        </div>
      </div>

      {/* Expandable "Why this score?" Section */}
      <div className="border-t border-gray-600 pt-3">
        <button
          onClick={handleToggle}
          className="flex items-center justify-between w-full text-left hover:bg-gray-700 hover:bg-opacity-30 p-2 rounded transition-colors"
        >
          <span className="text-sm font-medium text-gray-300">
            Why this score?
          </span>
          <svg
            className={`w-5 h-5 text-gray-400 transition-transform ${
              isExpanded ? 'rotate-180' : ''
            }`}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M19 9l-7 7-7-7"
            />
          </svg>
        </button>

        {isExpanded && (
          <div className="mt-3 space-y-3 text-sm">
            {/* Inputs Used */}
            <div>
              <h4 className="font-medium text-gray-300 mb-2">Inputs Used:</h4>
              <div className="space-y-1">
                {metric.inputs_used.map((input, idx) => (
                  <div
                    key={idx}
                    className="flex justify-between items-start bg-gray-700 bg-opacity-30 p-2 rounded"
                  >
                    <div>
                      <div className="text-gray-300">{input.name}</div>
                      <div className="text-xs text-gray-500">{input.source}</div>
                    </div>
                    <div className="text-gray-200 font-mono">
                      {typeof input.value === 'number'
                        ? input.value.toFixed(2)
                        : String(input.value)}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Calculation Method */}
            <div>
              <h4 className="font-medium text-gray-300 mb-2">
                Calculation Method:
              </h4>
              <p className="text-gray-400 bg-gray-700 bg-opacity-30 p-2 rounded font-mono text-xs">
                {metric.calculation_method}
              </p>
            </div>

            {/* Interpretation */}
            <div>
              <h4 className="font-medium text-gray-300 mb-2">Interpretation:</h4>
              <p className="text-gray-400 bg-gray-700 bg-opacity-30 p-2 rounded">
                {metric.interpretation}
              </p>
            </div>

            {/* Recommendation (if out of band) */}
            {metric.recommendation && (
              <div
                className="rounded p-3"
                style={{
                  backgroundColor: 'rgba(234, 179, 8, 0.1)',
                  border: '1px solid rgba(234, 179, 8, 0.3)',
                }}
              >
                <h4 className="font-medium text-yellow-500 mb-2 flex items-center gap-2">
                  <svg
                    className="w-5 h-5"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                    />
                  </svg>
                  Recommendation:
                </h4>
                <p className="text-gray-300">{metric.recommendation}</p>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );

  return compact ? renderCompactView() : renderFullView();
}
