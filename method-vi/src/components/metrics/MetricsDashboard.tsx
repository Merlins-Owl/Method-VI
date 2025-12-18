import { MetricsState, METRIC_METADATA } from '../../types/metrics';
import {
  Radar,
  RadarChart,
  PolarGrid,
  PolarAngleAxis,
  PolarRadiusAxis,
  ResponsiveContainer,
  Tooltip,
  Legend,
} from 'recharts';

interface MetricsDashboardProps {
  metrics: MetricsState;
  onClose: () => void;
}

export default function MetricsDashboard({ metrics, onClose }: MetricsDashboardProps) {
  // Prepare data for radar chart
  // Normalize all metrics to 0-100 scale for visualization
  const normalizeValue = (metricName: string, value: number): number => {
    const metadata = METRIC_METADATA[metricName as keyof typeof METRIC_METADATA];

    if (!metadata) return 0;

    // For metrics where lower is better (EV), invert the scale
    if (metadata.inverseScale) {
      // EV: 0 = best (100%), 30+ = worst (0%)
      const max = metadata.threshold.halt || 30;
      return Math.max(0, Math.min(100, ((max - value) / max) * 100));
    }

    // For percentage-based metrics (CI, IAS, PCI), multiply by 100
    if (value <= 1) {
      return value * 100;
    }

    // For already 0-100 metrics (EFI, SEC), use as-is
    return value;
  };

  const radarData = [
    {
      metric: 'CI',
      value: metrics.ci ? normalizeValue('CI', metrics.ci.value) : 0,
      fullMark: 100,
      available: !!metrics.ci,
    },
    {
      metric: 'EV',
      value: metrics.ev ? normalizeValue('EV', metrics.ev.value) : 0,
      fullMark: 100,
      available: !!metrics.ev,
    },
    {
      metric: 'IAS',
      value: metrics.ias ? normalizeValue('IAS', metrics.ias.value) : 0,
      fullMark: 100,
      available: !!metrics.ias,
    },
    {
      metric: 'EFI',
      value: metrics.efi ? normalizeValue('EFI', metrics.efi.value) : 0,
      fullMark: 100,
      available: !!metrics.efi,
    },
    {
      metric: 'SEC',
      value: metrics.sec ? normalizeValue('SEC', metrics.sec.value) : 0,
      fullMark: 100,
      available: !!metrics.sec,
    },
    {
      metric: 'PCI',
      value: metrics.pci ? normalizeValue('PCI', metrics.pci.value) : 0,
      fullMark: 100,
      available: !!metrics.pci,
    },
  ];

  // Count available metrics
  const availableCount = radarData.filter((d) => d.available).length;

  // Get overall status
  const getOverallStatus = () => {
    const metricsList = [metrics.ci, metrics.ev, metrics.ias, metrics.efi, metrics.sec, metrics.pci].filter(
      (m) => m !== null
    );

    if (metricsList.length === 0) return 'unknown';

    const hasFailure = metricsList.some((m) => m!.status === 'fail');
    const hasWarning = metricsList.some((m) => m!.status === 'warning');

    if (hasFailure) return 'fail';
    if (hasWarning) return 'warning';
    return 'pass';
  };

  const overallStatus = getOverallStatus();
  const statusColor =
    overallStatus === 'pass'
      ? 'text-green-500'
      : overallStatus === 'warning'
      ? 'text-yellow-500'
      : overallStatus === 'fail'
      ? 'text-red-500'
      : 'text-gray-500';

  const statusBgColor =
    overallStatus === 'pass'
      ? 'bg-green-500/10 border-green-500/30'
      : overallStatus === 'warning'
      ? 'bg-yellow-500/10 border-yellow-500/30'
      : overallStatus === 'fail'
      ? 'bg-red-500/10 border-red-500/30'
      : 'bg-gray-500/10 border-gray-500/30';

  return (
    <div
      className="fixed inset-0 bg-black z-50 flex items-center justify-center p-4"
      onClick={onClose}
    >
      <div
        className="bg-gray-900 rounded-lg max-w-6xl w-full max-h-[90vh] overflow-y-auto border-2 border-gray-500 shadow-2xl relative"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="sticky top-0 bg-gray-900 border-b-2 border-gray-700 p-6 flex items-center justify-between z-10">
          <div>
            <h2 className="text-2xl font-bold text-white mb-1">Metrics Dashboard</h2>
            <p className="text-sm text-gray-400">
              Visualizing {availableCount} of 6 Critical Metrics
            </p>
          </div>
          <button
            onClick={onClose}
            className="p-2 rounded-lg bg-gray-800 border-2 border-gray-600 hover:border-gray-500 hover:bg-gray-700 transition-colors group"
            title="Close (ESC)"
          >
            <svg
              className="w-6 h-6 text-gray-400 group-hover:text-white"
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

        <div className="p-6 space-y-6">
          {/* Overall Status Banner */}
          <div className={`p-4 rounded-lg border-2 ${statusBgColor}`}>
            <div className="flex items-center gap-3">
              <div className={`text-2xl font-bold ${statusColor}`}>
                {overallStatus === 'pass' && '✓'}
                {overallStatus === 'warning' && '⚠'}
                {overallStatus === 'fail' && '✕'}
                {overallStatus === 'unknown' && '○'}
              </div>
              <div>
                <div className={`font-bold ${statusColor}`}>
                  {overallStatus === 'pass' && 'All Metrics Passing'}
                  {overallStatus === 'warning' && 'Some Metrics Need Attention'}
                  {overallStatus === 'fail' && 'Critical Issues Detected'}
                  {overallStatus === 'unknown' && 'No Metrics Available'}
                </div>
                <div className="text-sm text-gray-400">
                  {availableCount === 6
                    ? 'All metrics have been calculated'
                    : `${6 - availableCount} metrics pending (will be available as steps progress)`}
                </div>
              </div>
            </div>
          </div>

          {/* Radar Chart */}
          <div className="bg-gray-800 rounded-lg border-2 border-gray-700 p-6 relative">
            <h3 className="text-lg font-bold text-white mb-4">Radar View</h3>

            {availableCount === 0 ? (
              <div className="h-96 flex items-center justify-center text-gray-400 bg-gray-900 rounded">
                <div className="text-center p-8">
                  <div className="text-4xl mb-2">📊</div>
                  <div className="text-lg font-medium">No metrics available yet</div>
                  <div className="text-sm mt-2">Metrics will appear as you progress through the steps</div>
                </div>
              </div>
            ) : (
              <ResponsiveContainer width="100%" height={400}>
                <RadarChart data={radarData}>
                  <PolarGrid stroke="#4B5563" />
                  <PolarAngleAxis
                    dataKey="metric"
                    tick={{ fill: '#9CA3AF', fontSize: 14 }}
                  />
                  <PolarRadiusAxis
                    angle={90}
                    domain={[0, 100]}
                    tick={{ fill: '#6B7280', fontSize: 12 }}
                  />
                  <Radar
                    name="Current Values"
                    dataKey="value"
                    stroke="#3B82F6"
                    fill="#3B82F6"
                    fillOpacity={0.3}
                    strokeWidth={2}
                  />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: '#1F2937',
                      border: '1px solid #374151',
                      borderRadius: '0.5rem',
                      color: '#F3F4F6',
                    }}
                    formatter={(value: number) => `${value.toFixed(1)}%`}
                  />
                  <Legend
                    wrapperStyle={{ color: '#9CA3AF' }}
                  />
                </RadarChart>
              </ResponsiveContainer>
            )}

            <div className="mt-4 text-xs text-gray-500 text-center">
              Note: All metrics normalized to 0-100% scale for comparison
            </div>
          </div>

          {/* Metric Details Grid */}
          <div className="bg-gray-800 rounded-lg border-2 border-gray-700 p-6">
            <h3 className="text-lg font-bold text-white mb-4">Current Values</h3>

            <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
              {[
                { key: 'ci', name: 'CI - Coherence Index', metric: metrics.ci },
                { key: 'ev', name: 'EV - Expansion Variance', metric: metrics.ev },
                { key: 'ias', name: 'IAS - Intent Alignment Score', metric: metrics.ias },
                { key: 'efi', name: 'EFI - Execution Fidelity Index', metric: metrics.efi },
                { key: 'sec', name: 'SEC - Scope Expansion Count', metric: metrics.sec },
                { key: 'pci', name: 'PCI - Pattern Consistency Index', metric: metrics.pci },
              ].map(({ key, name, metric }) => {
                const status = metric?.status || 'unknown';
                const cardBg =
                  status === 'pass'
                    ? 'bg-green-500/20 border-green-500'
                    : status === 'warning'
                    ? 'bg-yellow-500/20 border-yellow-500'
                    : status === 'fail'
                    ? 'bg-red-500/20 border-red-500'
                    : 'bg-gray-700/50 border-gray-600';

                const valueColor =
                  status === 'pass'
                    ? 'text-green-500'
                    : status === 'warning'
                    ? 'text-yellow-500'
                    : status === 'fail'
                    ? 'text-red-500'
                    : 'text-gray-600';

                return (
                  <div
                    key={key}
                    className={`p-4 rounded-lg border-2 ${cardBg}`}
                  >
                    <div className="text-sm text-gray-400 mb-1">{name}</div>
                    <div className={`text-2xl font-bold ${valueColor}`}>
                      {metric ? metric.value : '-'}
                    </div>
                    {metric && (
                      <div className="text-xs text-gray-500 mt-1 capitalize">
                        {status}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          </div>

          {/* Threshold Reference */}
          <div className="bg-gray-800 rounded-lg border-2 border-gray-700 p-6">
            <h3 className="text-lg font-bold text-white mb-4">Threshold Reference</h3>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 rounded-full bg-green-500"></div>
                  <span className="text-gray-300">Pass</span>
                  <span className="text-gray-500">- Meets or exceeds target</span>
                </div>
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 rounded-full bg-yellow-500"></div>
                  <span className="text-gray-300">Warning</span>
                  <span className="text-gray-500">- Below target, needs attention</span>
                </div>
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 rounded-full bg-red-500"></div>
                  <span className="text-gray-300">Fail</span>
                  <span className="text-gray-500">- Critical issue, must address</span>
                </div>
              </div>

              <div className="text-gray-400 text-xs space-y-1">
                <div>CI (Coherence Index): Pass ≥ 0.80, Warn ≥ 0.70, Fail &lt; 0.50</div>
                <div>EV (Expansion Variance): Pass ≤ 10, Warn ≤ 20, Fail &gt; 30</div>
                <div>IAS (Intent Alignment): Pass ≥ 0.80, Warn ≥ 0.70, Fail &lt; 0.50</div>
                <div>EFI (Execution Fidelity): Pass ≥ 95, Warn ≥ 90, Fail &lt; 80</div>
                <div>SEC (Scope Expansion Count): Pass = 100 (strict)</div>
                <div>PCI (Pattern Consistency): Pass ≥ 0.90, Warn ≥ 0.85, Fail &lt; 0.70</div>
              </div>
            </div>
          </div>

          {/* Note about gradual availability */}
          <div className="bg-blue-500/10 border-2 border-blue-500/30 rounded-lg p-4">
            <div className="flex items-start gap-3">
              <div className="text-blue-500 text-xl">ℹ️</div>
              <div className="text-sm text-blue-300">
                <div className="font-semibold mb-1">Metrics Availability</div>
                <div className="text-blue-400">
                  Metrics become available progressively as you complete steps. Some metrics like IAS
                  and SEC are available early (Step 0-1), while others like CI and PCI require
                  completion of later steps (Step 3+). This is normal and ensures metrics reflect
                  actual analysis progress.
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
