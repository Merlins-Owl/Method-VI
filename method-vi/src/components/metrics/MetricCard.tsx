import { useState } from 'react';
import {
  MetricResult,
  METRIC_METADATA,
  getStatusColor,
  getStatusBgColor,
  formatMetricValue,
} from '../../types/metrics';

interface MetricCardProps {
  metric: MetricResult;
  compact?: boolean;
  onExpand?: () => void;
}

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

  const getStatusClasses = (status: string) => {
    if (status === 'pass') {
      return {
        bg: 'bg-green-100',
        border: 'border-green-500',
        text: 'text-green-700',
        badge: 'bg-green-500 text-white'
      };
    }
    if (status === 'warning') {
      return {
        bg: 'bg-yellow-100',
        border: 'border-yellow-500',
        text: 'text-yellow-700',
        badge: 'bg-yellow-500 text-white'
      };
    }
    // halt/fail
    return {
      bg: 'bg-red-100',
      border: 'border-red-500',
      text: 'text-red-700',
      badge: 'bg-red-500 text-white'
    };
  };

  const colors = getStatusClasses(metric.status);

  const renderCompactView = () => (
    <button
      onClick={handleToggle}
      className={`px-3 py-2 rounded-lg border-2 transition-all hover:scale-105 cursor-pointer ${colors.bg} ${colors.border}`}
    >
      <div className="flex items-center gap-2">
        <div className="text-sm font-medium text-gray-800">{metric.metric_name}</div>
        <div className={`text-lg font-bold ${colors.text}`}>
          {formatMetricValue(metric.value, metadata.unit)}
        </div>
      </div>
    </button>
  );

  const renderFullView = () => (
    <div
      className={`rounded-lg border-2 p-4 transition-all ${colors.bg} ${colors.border}`}
    >
      {/* Header */}
      <div className="flex items-start justify-between mb-3">
        <div>
          <h3 className="text-xl font-bold text-gray-900">
            {metric.metric_name} - {metadata.fullName}
          </h3>
          <p className="text-sm text-gray-600 mt-1">{metadata.description}</p>
        </div>
        <div className="text-right">
          <div className={`text-3xl font-bold ${colors.text}`}>
            {formatMetricValue(metric.value, metadata.unit)}
          </div>
          <div className="text-xs text-gray-600 mt-1">
            Pass: {formatMetricValue(metric.threshold.pass, metadata.unit)}
          </div>
        </div>
      </div>

      {/* Status Indicator */}
      <div className="flex items-center gap-2 mb-3">
        <span className={`px-2 py-1 rounded text-xs font-medium uppercase ${colors.badge}`}>
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
            className="absolute top-0 left-0 h-full bg-red-500/30"
            style={{
              width: metadata.inverseScale
                ? '100%'
                : `${((metric.threshold.warning ?? metric.threshold.halt ?? 0) / (metadata.inverseScale ? metric.threshold.pass : 100)) * 100}%`,
            }}
          />
          {/* Yellow zone (warning) */}
          {metric.threshold.warning !== null && (
            <div
              className="absolute top-0 h-full bg-yellow-500/30"
              style={{
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
            className="absolute top-0 h-full bg-green-500/30"
            style={{
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
            className={`absolute top-0 w-1 h-full ${getTextColor().replace('text-', 'bg-')}`}
            style={{
              left: `${(metric.value / (metadata.inverseScale ? (metric.threshold.halt ?? 100) : 100)) * 100}%`,
            }}
          />
        </div>
      </div>

      {/* Expandable "Why this score?" Section */}
      <div className="border-t border-gray-600 pt-3">
        <button
          onClick={handleToggle}
          className="flex items-center justify-between w-full text-left hover:bg-gray-700/30 p-2 rounded transition-colors"
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
                    className="flex justify-between items-start bg-gray-700/30 p-2 rounded"
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
              <p className="text-gray-400 bg-gray-700/30 p-2 rounded font-mono text-xs">
                {metric.calculation_method}
              </p>
            </div>

            {/* Interpretation */}
            <div>
              <h4 className="font-medium text-gray-300 mb-2">Interpretation:</h4>
              <p className="text-gray-400 bg-gray-700/30 p-2 rounded">
                {metric.interpretation}
              </p>
            </div>

            {/* Recommendation (if out of band) */}
            {metric.recommendation && (
              <div className="bg-yellow-500/10 border border-yellow-500/30 rounded p-3">
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
