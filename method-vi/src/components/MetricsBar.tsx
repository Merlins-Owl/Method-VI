import { useState } from 'react';
import { MetricsState, MetricResult } from '../types/metrics';
import MetricCard from './metrics/MetricCard';

interface MetricsBarProps {
  metrics?: MetricsState;
}

export default function MetricsBar({ metrics }: MetricsBarProps) {
  const [selectedMetric, setSelectedMetric] = useState<MetricResult | null>(null);

  // Get array of all metrics
  const metricsList: (MetricResult | null)[] = [
    metrics?.ci ?? null,
    metrics?.ev ?? null,
    metrics?.ias ?? null,
    metrics?.efi ?? null,
    metrics?.sec ?? null,
    metrics?.pci ?? null,
  ];

  const handleMetricClick = (metric: MetricResult) => {
    setSelectedMetric(metric);
  };

  const handleCloseModal = () => {
    setSelectedMetric(null);
  };

  return (
    <>
      <div className="bg-gray-900 border-t border-gray-700 px-6 py-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            {metricsList.map((metric, idx) => {
              if (!metric) {
                // Show placeholder for null metrics
                const metricNames = ['CI', 'EV', 'IAS', 'EFI', 'SEC', 'PCI'];
                return (
                  <div
                    key={idx}
                    className="px-3 py-2 rounded border-2 border-gray-700 bg-gray-800/30"
                  >
                    <div className="flex items-center gap-2">
                      <div className="text-sm font-medium text-gray-600">
                        {metricNames[idx]}
                      </div>
                      <div className="text-lg font-bold text-gray-600">-</div>
                    </div>
                  </div>
                );
              }

              return (
                <MetricCard
                  key={metric.metric_name}
                  metric={metric}
                  compact={true}
                  onExpand={() => handleMetricClick(metric)}
                />
              );
            })}
          </div>

          <div className="flex items-center gap-3">
            <button
              onClick={() => {
                /* TODO: Open full dashboard */
              }}
              className="text-xs text-gray-400 hover:text-gray-300 transition-colors flex items-center gap-1"
            >
              <svg
                className="w-4 h-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
                />
              </svg>
              Dashboard
            </button>
            <div className="text-xs text-gray-500">Critical 6 Metrics</div>
          </div>
        </div>
      </div>

      {/* Modal for expanded metric view */}
      {selectedMetric && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="bg-gray-800 rounded-lg max-w-2xl w-full max-h-[80vh] overflow-y-auto">
            <div className="sticky top-0 bg-gray-800 border-b border-gray-700 p-4 flex items-center justify-between">
              <h2 className="text-xl font-bold text-white">Metric Details</h2>
              <button
                onClick={handleCloseModal}
                className="text-gray-400 hover:text-white transition-colors"
              >
                <svg
                  className="w-6 h-6"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            </div>
            <div className="p-4">
              <MetricCard metric={selectedMetric} compact={false} />
            </div>
          </div>
        </div>
      )}
    </>
  );
}
