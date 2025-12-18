import { useState } from 'react';
import MainLayout from '../components/layout/MainLayout';
import MetricCard from '../components/metrics/MetricCard';
import { MetricsState } from '../types/metrics';
import { generateMockMetrics, MOCK_SCENARIOS } from '../utils/mockMetrics';

type TestScenario = 'allPass' | 'someWarnings' | 'someFailures' | 'custom';

export default function MetricsTestPage() {
  const [scenario, setScenario] = useState<TestScenario>('allPass');
  const [customCI, setCustomCI] = useState(0.85);

  // Get metrics based on selected scenario
  const getMetrics = (): MetricsState => {
    if (scenario === 'custom') {
      return generateMockMetrics({ ci: customCI });
    }
    return MOCK_SCENARIOS[scenario];
  };

  const metrics = getMetrics();

  // Helper to determine color for custom CI input
  const getCIColor = (value: number) => {
    if (value >= 0.80) return 'text-green-500';
    if (value >= 0.70) return 'text-yellow-500';
    return 'text-red-500';
  };

  return (
    <MainLayout metrics={metrics} showSidebar={false}>
      <div className="p-8 max-w-7xl mx-auto">
        <h1 className="text-4xl font-bold text-white mb-8">
          Metrics Verification Test Page
        </h1>

        {/* Test Scenario Selector */}
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6 mb-8">
          <h2 className="text-2xl font-bold text-white mb-4">
            Test Scenarios
          </h2>

          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <button
              onClick={() => setScenario('allPass')}
              className={`
                p-4 rounded border-2 transition-all
                ${
                  scenario === 'allPass'
                    ? 'bg-green-500/20 border-green-500 text-green-500'
                    : 'bg-gray-700 border-gray-600 text-gray-300 hover:border-gray-500'
                }
              `}
            >
              <div className="font-bold mb-1">All Pass</div>
              <div className="text-sm opacity-75">All metrics green</div>
            </button>

            <button
              onClick={() => setScenario('someWarnings')}
              className={`
                p-4 rounded border-2 transition-all
                ${
                  scenario === 'someWarnings'
                    ? 'bg-yellow-500/20 border-yellow-500 text-yellow-500'
                    : 'bg-gray-700 border-gray-600 text-gray-300 hover:border-gray-500'
                }
              `}
            >
              <div className="font-bold mb-1">Some Warnings</div>
              <div className="text-sm opacity-75">Mix of pass/warning</div>
            </button>

            <button
              onClick={() => setScenario('someFailures')}
              className={`
                p-4 rounded border-2 transition-all
                ${
                  scenario === 'someFailures'
                    ? 'bg-red-500/20 border-red-500 text-red-500'
                    : 'bg-gray-700 border-gray-600 text-gray-300 hover:border-gray-500'
                }
              `}
            >
              <div className="font-bold mb-1">Some Failures</div>
              <div className="text-sm opacity-75">Mix with failures</div>
            </button>

            <button
              onClick={() => setScenario('custom')}
              className={`
                p-4 rounded border-2 transition-all
                ${
                  scenario === 'custom'
                    ? 'bg-blue-500/20 border-blue-500 text-blue-500'
                    : 'bg-gray-700 border-gray-600 text-gray-300 hover:border-gray-500'
                }
              `}
            >
              <div className="font-bold mb-1">Custom CI</div>
              <div className="text-sm opacity-75">Adjust CI value</div>
            </button>
          </div>

          {/* Custom CI Slider */}
          {scenario === 'custom' && (
            <div className="mt-6 p-4 bg-gray-700 rounded">
              <label className="block text-sm font-medium text-gray-300 mb-2">
                CI Value:{' '}
                <span className={`font-bold ${getCIColor(customCI)}`}>
                  {customCI.toFixed(2)}
                </span>
              </label>
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={customCI}
                onChange={(e) => setCustomCI(parseFloat(e.target.value))}
                className="w-full h-2 bg-gray-600 rounded-lg appearance-none cursor-pointer"
              />
              <div className="flex justify-between text-xs text-gray-400 mt-1">
                <span>0.00 (Fail)</span>
                <span>0.70 (Warning)</span>
                <span>0.80 (Pass)</span>
                <span>1.00</span>
              </div>
            </div>
          )}
        </div>

        {/* Verification Checklist */}
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6 mb-8">
          <h2 className="text-2xl font-bold text-white mb-4">
            Verification Checklist
          </h2>

          <div className="space-y-2 text-gray-300">
            <div className="flex items-center gap-2">
              <input type="checkbox" className="w-4 h-4" />
              <label>1. Metrics bar shows 6 metrics at bottom of page</label>
            </div>
            <div className="flex items-center gap-2">
              <input type="checkbox" className="w-4 h-4" />
              <label>
                2. Color coding: CI=0.85 → Green, CI=0.75 → Yellow, CI=0.45 →
                Red
              </label>
            </div>
            <div className="flex items-center gap-2">
              <input type="checkbox" className="w-4 h-4" />
              <label>
                3. Click metric card → "Why this score?" expands with
                explanation
              </label>
            </div>
            <div className="flex items-center gap-2">
              <input type="checkbox" className="w-4 h-4" />
              <label>4. Dashboard button accessible from metrics bar</label>
            </div>
          </div>

          <div className="mt-4 p-4 bg-blue-500/10 border border-blue-500/30 rounded">
            <p className="text-blue-300 text-sm">
              <strong>Instructions:</strong> Use the scenario buttons above to
              test different metric states. Check the metrics bar at the bottom
              of the page. Click on any metric to see the detailed explainability
              card.
            </p>
          </div>
        </div>

        {/* Metric Cards Grid */}
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
          <h2 className="text-2xl font-bold text-white mb-4">
            Current Metrics - Full View
          </h2>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
            {metrics.ci && <MetricCard metric={metrics.ci} />}
            {metrics.ev && <MetricCard metric={metrics.ev} />}
            {metrics.ias && <MetricCard metric={metrics.ias} />}
            {metrics.efi && <MetricCard metric={metrics.efi} />}
            {metrics.sec && <MetricCard metric={metrics.sec} />}
            {metrics.pci && <MetricCard metric={metrics.pci} />}
          </div>
        </div>

        {/* Expected Values Reference */}
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6 mt-8">
          <h2 className="text-2xl font-bold text-white mb-4">
            Threshold Reference
          </h2>

          <div className="overflow-x-auto">
            <table className="w-full text-sm text-gray-300">
              <thead className="text-xs uppercase bg-gray-700">
                <tr>
                  <th className="px-4 py-2 text-left">Metric</th>
                  <th className="px-4 py-2 text-left">Pass</th>
                  <th className="px-4 py-2 text-left">Warning</th>
                  <th className="px-4 py-2 text-left">Halt</th>
                  <th className="px-4 py-2 text-left">Note</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-700">
                <tr>
                  <td className="px-4 py-2 font-medium">CI</td>
                  <td className="px-4 py-2 text-green-500">≥ 0.80</td>
                  <td className="px-4 py-2 text-yellow-500">≥ 0.70</td>
                  <td className="px-4 py-2 text-red-500">&lt; 0.50</td>
                  <td className="px-4 py-2 text-gray-400">Higher is better</td>
                </tr>
                <tr>
                  <td className="px-4 py-2 font-medium">EV</td>
                  <td className="px-4 py-2 text-green-500">≤ 10</td>
                  <td className="px-4 py-2 text-yellow-500">≤ 20</td>
                  <td className="px-4 py-2 text-red-500">&gt; 30</td>
                  <td className="px-4 py-2 text-gray-400">Lower is better</td>
                </tr>
                <tr>
                  <td className="px-4 py-2 font-medium">IAS</td>
                  <td className="px-4 py-2 text-green-500">≥ 0.80</td>
                  <td className="px-4 py-2 text-yellow-500">≥ 0.70</td>
                  <td className="px-4 py-2 text-red-500">&lt; 0.50</td>
                  <td className="px-4 py-2 text-gray-400">Higher is better</td>
                </tr>
                <tr>
                  <td className="px-4 py-2 font-medium">EFI</td>
                  <td className="px-4 py-2 text-green-500">≥ 95</td>
                  <td className="px-4 py-2 text-yellow-500">≥ 90</td>
                  <td className="px-4 py-2 text-red-500">&lt; 80</td>
                  <td className="px-4 py-2 text-gray-400">Higher is better</td>
                </tr>
                <tr>
                  <td className="px-4 py-2 font-medium">SEC</td>
                  <td className="px-4 py-2 text-green-500">= 100</td>
                  <td className="px-4 py-2 text-gray-600">N/A</td>
                  <td className="px-4 py-2 text-gray-600">N/A</td>
                  <td className="px-4 py-2 text-gray-400">
                    Must be perfect
                  </td>
                </tr>
                <tr>
                  <td className="px-4 py-2 font-medium">PCI</td>
                  <td className="px-4 py-2 text-green-500">≥ 0.90</td>
                  <td className="px-4 py-2 text-yellow-500">≥ 0.85</td>
                  <td className="px-4 py-2 text-red-500">&lt; 0.70</td>
                  <td className="px-4 py-2 text-gray-400">Higher is better</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </MainLayout>
  );
}
