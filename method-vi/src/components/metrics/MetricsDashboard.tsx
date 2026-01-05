import { MetricsState, METRIC_METADATA } from '../../types/metrics';
import {
  Radar,
  RadarChart,
  PolarGrid,
  PolarAngleAxis,
  PolarRadiusAxis,
  ResponsiveContainer,
  Tooltip,
} from 'recharts';

interface MetricsDashboardProps {
  metrics?: MetricsState;  // Make optional to match MetricsBar
  onClose: () => void;
}

// Color constants for inline styles
const COLORS = {
  green: {
    bg: 'rgba(34, 197, 94, 0.2)',
    border: 'rgba(34, 197, 94, 0.5)',
    text: '#22c55e',
  },
  yellow: {
    bg: 'rgba(234, 179, 8, 0.2)',
    border: 'rgba(234, 179, 8, 0.5)',
    text: '#eab308',
  },
  red: {
    bg: 'rgba(239, 68, 68, 0.2)',
    border: 'rgba(239, 68, 68, 0.5)',
    text: '#ef4444',
  },
  gray: {
    bg: 'rgba(107, 114, 128, 0.5)',
    border: '#4b5563',
    text: '#4b5563',
  },
  blue: {
    bg: 'rgba(59, 130, 246, 0.1)',
    border: 'rgba(59, 130, 246, 0.3)',
    text: '#3b82f6',
  },
};

export default function MetricsDashboard({ metrics, onClose }: MetricsDashboardProps) {
  // Early return if metrics is null/undefined (safety check)
  if (!metrics) {
    return (
      <div
        className="fixed inset-0 z-50 flex items-center justify-center p-4"
        style={{ backgroundColor: 'rgba(0, 0, 0, 0.9)' }}
        onClick={onClose}
      >
        <div className="text-white text-center p-8">
          <div className="text-4xl mb-4">⚠️</div>
          <div className="text-xl">No metrics data available</div>
        </div>
      </div>
    );
  }

  // Prepare data for radar chart
  // Normalize all metrics to 0-100 scale for visualization
  const normalizeValue = (metricName: string, value: number): number => {
    const metadata = METRIC_METADATA[metricName as keyof typeof METRIC_METADATA];

    if (!metadata) return 0;

    // For metrics where lower is better (EV), invert the scale
    if (metadata.inverseScale) {
      // EV: 0 = best (100%), 30+ = worst (0%)
      const max = 30; // EV halt threshold from DEFAULT_THRESHOLDS
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
  ].map(item => ({
    ...item,
    // Ensure value is always a valid number
    value: Number.isFinite(item.value) ? item.value : 0,
  }));

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
  
  // Get status colors using inline styles
  const getStatusColors = (status: string) => {
    switch (status) {
      case 'pass':
        return COLORS.green;
      case 'warning':
        return COLORS.yellow;
      case 'fail':
        return COLORS.red;
      default:
        return COLORS.gray;
    }
  };

  const statusColors = getStatusColors(overallStatus);

  // Get metric card styles
  const getMetricCardStyles = (status: string | undefined) => {
    if (!status || status === 'unknown') {
      return {
        bg: COLORS.gray.bg,
        border: COLORS.gray.border,
        text: COLORS.gray.text,
      };
    }
    return getStatusColors(status);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4"
      style={{ backgroundColor: 'rgba(0, 0, 0, 0.9)' }}
      onClick={onClose}
    >
      <div
        className="rounded-lg max-w-6xl w-full max-h-[90vh] overflow-y-auto shadow-2xl relative"
        style={{
          backgroundColor: '#111827', // gray-900
          border: '2px solid #6b7280', // gray-500
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div
          className="sticky top-0 p-6 flex items-center justify-between z-10"
          style={{
            backgroundColor: '#111827', // gray-900
            borderBottom: '2px solid #374151', // gray-700
          }}
        >
          <div>
            <h2 className="text-2xl font-bold text-white mb-1">Metrics Dashboard</h2>
            <p className="text-sm text-gray-400">
              Visualizing {availableCount} of 6 Critical Metrics
            </p>
          </div>
          <button
            onClick={onClose}
            className="p-2 rounded-lg transition-colors"
            style={{
              backgroundColor: '#374151',
              border: '2px solid #6b7280',
              color: '#e5e7eb',
              fontSize: '24px',
              fontWeight: 'bold',
              lineHeight: '1',
              width: '44px',
              height: '44px',
            }}
            title="Close (ESC)"
          >
            ✕
          </button>
        </div>

        <div className="p-6 space-y-6">
          {/* Overall Status Banner */}
          <div
            className="p-4 rounded-lg"
            style={{
              backgroundColor: statusColors.bg,
              border: `2px solid ${statusColors.border}`,
            }}
          >
            <div className="flex items-center gap-3">
              <div className="text-2xl font-bold" style={{ color: statusColors.text }}>
                {overallStatus === 'pass' && '✓'}
                {overallStatus === 'warning' && '⚠'}
                {overallStatus === 'fail' && '✕'}
                {overallStatus === 'unknown' && '○'}
              </div>
              <div>
                <div className="font-bold" style={{ color: statusColors.text }}>
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
          <div
            className="rounded-lg p-6 relative"
            style={{
              backgroundColor: '#1f2937', // gray-800
              border: '2px solid #374151', // gray-700
            }}
          >
            <h3 className="text-lg font-bold text-white mb-4">Radar View</h3>

            {/* Fixed-size container to prevent layout issues */}
            <div
              style={{
                width: '100%',
                height: '384px', // h-96 equivalent
                backgroundColor: '#111827',
                border: '2px solid #374151',
                borderRadius: '0.5rem',
                position: 'relative',
                overflow: 'hidden',
              }}
            >
              {/* Only render RadarChart if we have valid metrics data */}
            {metrics && (metrics.ci || metrics.ev || metrics.ias) && availableCount >= 2 ? (
                <ResponsiveContainer width="100%" height={380}>
                  <RadarChart
                    cx="50%"
                    cy="50%"
                    outerRadius="70%"
                    data={radarData}
                    margin={{ top: 20, right: 30, bottom: 20, left: 30 }}
                  >
                    <PolarGrid stroke="#374151" />
                    <PolarAngleAxis
                      dataKey="metric"
                      tick={{ fill: '#9ca3af', fontSize: 12 }}
                    />
                    <PolarRadiusAxis
                      angle={90}
                      domain={[0, 100]}
                      tick={{ fill: '#6b7280', fontSize: 10 }}
                      tickCount={5}
                    />
                    <Radar
                      name="Current"
                      dataKey="value"
                      stroke="#22c55e"
                      fill="#22c55e"
                      fillOpacity={0.3}
                      strokeWidth={2}
                      isAnimationActive={false}
                    />
                    <Tooltip
                      contentStyle={{
                        backgroundColor: '#1f2937',
                        border: '1px solid #374151',
                        borderRadius: '0.5rem',
                        color: '#e5e7eb',
                      }}
                      formatter={(value: number | undefined) => [
                        `${typeof value === 'number' ? value.toFixed(1) : 0}%`,
                        'Value',
                      ]}
                    />
                  </RadarChart>
                </ResponsiveContainer>
              ) : (
                <div className="h-full flex items-center justify-center text-gray-400">
                  <div className="text-center p-8">
                    <div className="text-4xl mb-2">📊</div>
                    <div className="text-lg font-medium">No Metrics Available Yet</div>
                    <div className="text-sm mt-2">Complete steps to see the radar chart</div>
                  </div>
                </div>
              )}
            </div>

            <div className="mt-4 text-xs text-gray-500 text-center">
              Note: All metrics normalized to 0-100% scale for comparison
            </div>
          </div>

          {/* Metric Details Grid */}
          <div
            className="rounded-lg p-6"
            style={{
              backgroundColor: '#1f2937', // gray-800
              border: '2px solid #374151', // gray-700
            }}
          >
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
                const cardStyles = getMetricCardStyles(status);

                return (
                  <div
                    key={key}
                    className="p-4 rounded-lg"
                    style={{
                      backgroundColor: cardStyles.bg,
                      border: `2px solid ${cardStyles.border}`,
                    }}
                  >
                    <div className="text-sm text-gray-400 mb-1">{name}</div>
                    <div
                      className="text-2xl font-bold"
                      style={{ color: cardStyles.text }}
                    >
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
          <div
            className="rounded-lg p-6"
            style={{
              backgroundColor: '#1f2937', // gray-800
              border: '2px solid #374151', // gray-700
            }}
          >
            <h3 className="text-lg font-bold text-white mb-4">Threshold Reference</h3>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  <div
                    className="w-3 h-3 rounded-full"
                    style={{ backgroundColor: '#22c55e' }}
                  ></div>
                  <span className="text-gray-300">Pass</span>
                  <span className="text-gray-500">- Meets or exceeds target</span>
                </div>
                <div className="flex items-center gap-2">
                  <div
                    className="w-3 h-3 rounded-full"
                    style={{ backgroundColor: '#eab308' }}
                  ></div>
                  <span className="text-gray-300">Warning</span>
                  <span className="text-gray-500">- Below target, needs attention</span>
                </div>
                <div className="flex items-center gap-2">
                  <div
                    className="w-3 h-3 rounded-full"
                    style={{ backgroundColor: '#ef4444' }}
                  ></div>
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
          <div
            className="rounded-lg p-4"
            style={{
              backgroundColor: COLORS.blue.bg,
              border: `2px solid ${COLORS.blue.border}`,
            }}
          >
            <div className="flex items-start gap-3">
              <div style={{ color: COLORS.blue.text }} className="text-xl">ℹ️</div>
              <div className="text-sm">
                <div className="font-semibold mb-1" style={{ color: '#93c5fd' }}>
                  Metrics Availability
                </div>
                <div style={{ color: '#60a5fa' }}>
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
