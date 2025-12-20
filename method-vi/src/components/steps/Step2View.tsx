import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Step2ViewProps {
  runId: string;
  onGovernanceCalibrated: () => void;
}

interface Step2Response {
  governance_summary_id: string;
  domain_snapshots_id: string;
  governance_summary: string;
  domain_snapshots: string;
  metrics: {
    ci: number | null;
    ev: number | null;
    ias: number | null;
    efi: number | null;
    sec: number | null;
    pci: number | null;
  } | null;
}

interface DomainStatus {
  name: string;
  icon: string;
  metric: string;
  baseline: string;
  target: string;
  status: 'configured' | 'deferred' | 'measured';
  color: string;
}

type ViewState = 'initializing' | 'calibrating' | 'review' | 'approved' | 'error';

export default function Step2View({ runId, onGovernanceCalibrated }: Step2ViewProps) {
  const [viewState, setViewState] = useState<ViewState>('initializing');
  const [result, setResult] = useState<Step2Response | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [calibrationProgress, setCalibrationProgress] = useState<string[]>([]);
  const executedRef = useRef(false);

  useEffect(() => {
    // Prevent double execution in React Strict Mode
    if (executedRef.current) {
      console.log('[Step2View] Step 2 already executed, skipping duplicate call');
      return;
    }
    executedRef.current = true;
    executeStep2();
  }, []);

  const executeStep2 = async () => {
    setViewState('calibrating');
    setError(null);
    setCalibrationProgress([]);

    try {
      console.log('[Step2View] Starting Step 2 execution for runId:', runId);

      setCalibrationProgress(['Initializing governance calibration...']);

      // Call backend to execute Step 2
      console.log('[Step2View] Calling execute_step_2 backend command...');

      const response = await invoke<Step2Response>('execute_step_2', {
        runId,
      });

      console.log('[Step2View] Step 2 result received:', response);
      setResult(response);
      setCalibrationProgress(prev => [...prev, '✓ Governance calibration complete']);
      setViewState('review');
    } catch (err) {
      console.error('[Step2View] Error executing Step 2:', err);
      setError(`Failed to calibrate governance: ${err}`);
      setViewState('error');
    }
  };

  const handleApprove = async () => {
    try {
      console.log('Approving governance calibration...');

      await invoke('approve_gate', {
        approver: 'User',
      });

      setViewState('approved');

      // Notify parent that governance is calibrated
      setTimeout(() => {
        onGovernanceCalibrated();
      }, 1500);
    } catch (err) {
      console.error('Error approving governance calibration:', err);
      setError(`Failed to approve calibration: ${err}`);
    }
  };

  const getDomainStatuses = (): DomainStatus[] => {
    const eBaseline = result?.governance_summary.match(/E_baseline.*?(\d+)\s*words/)?.[1] || '0';

    return [
      {
        name: 'Clarity Domain',
        icon: '🔍',
        metric: 'CI (Coherence Index)',
        baseline: result?.metrics?.ci ? `${result.metrics.ci.toFixed(2)}` : 'Not yet measured',
        target: '≥ 0.82',
        status: result?.metrics?.ci ? 'measured' : 'configured',
        color: 'blue',
      },
      {
        name: 'Entropy Domain',
        icon: '📊',
        metric: 'EV (Expansion Variance)',
        baseline: `0.0% (at ${eBaseline} words)`,
        target: '≤ ±10%',
        status: 'measured',
        color: 'green',
      },
      {
        name: 'Alignment Domain',
        icon: '🎯',
        metric: 'IAS (Intent Alignment Score)',
        baseline: result?.metrics?.ias ? `${result.metrics.ias.toFixed(2)}` : 'Not yet measured',
        target: '≥ 0.82',
        status: result?.metrics?.ias ? 'measured' : 'configured',
        color: 'purple',
      },
      {
        name: 'Cadence Domain',
        icon: '⏱️',
        metric: 'RCC (Reflection Cadence Compliance)',
        baseline: 'On schedule (Step 2 complete)',
        target: 'Per Architecture Map',
        status: 'configured',
        color: 'yellow',
      },
      {
        name: 'Overhead Domain',
        icon: '⚖️',
        metric: 'GLR (Governance Latency Ratio)',
        baseline: 'Not yet measured (Phase 2 metric)',
        target: '≤ 15%',
        status: 'deferred',
        color: 'gray',
      },
    ];
  };

  const renderCalibratingView = () => (
    <div className="max-w-4xl mx-auto p-8">
      <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
        <h2 className="text-3xl font-bold text-white mb-4">
          Step 2: Governance Calibration
        </h2>
        <p className="text-gray-300 mb-6">
          Configuring active governance controls...
        </p>

        <div className="space-y-2 mb-6">
          {calibrationProgress.map((message, index) => (
            <div key={index} className="flex items-center text-gray-300">
              <div className="mr-3">
                {message.startsWith('✓') ? (
                  <span className="text-green-400">✓</span>
                ) : (
                  <div className="animate-spin h-4 w-4 border-2 border-blue-500 border-t-transparent rounded-full"></div>
                )}
              </div>
              <span>{message}</span>
            </div>
          ))}
        </div>

        {viewState === 'calibrating' && (
          <div className="bg-blue-900/20 border border-blue-700 rounded-lg p-4">
            <p className="text-sm text-blue-300">
              Reviewing Charter and configuring five control domains...
            </p>
          </div>
        )}
      </div>
    </div>
  );

  const renderReviewView = () => {
    if (!result) return null;

    const domainStatuses = getDomainStatuses();

    return (
      <div className="max-w-6xl mx-auto p-8">
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
          <h2 className="text-3xl font-bold text-white mb-2">
            Step 2: Governance Calibration
          </h2>
          <p className="text-gray-300 mb-6">
            Review the configured governance controls and approve to proceed
          </p>

          {/* Governance Status Banner */}
          <div className="bg-blue-900/20 border border-blue-700 rounded-lg p-4 mb-6">
            <h3 className="text-lg font-semibold text-blue-300 mb-2">
              📊 Active Governance Configured
            </h3>
            <p className="text-blue-200">
              Five control domains are now monitoring this run continuously
            </p>
            <div className="mt-3 grid grid-cols-3 gap-4 text-sm">
              <div>
                <span className="text-blue-400">Status:</span>{' '}
                <span className="text-green-400 font-semibold">ENABLED</span>
              </div>
              <div>
                <span className="text-blue-400">Role:</span>{' '}
                <span className="text-blue-200">Conductor</span>
              </div>
              <div>
                <span className="text-blue-400">Strategy:</span>{' '}
                <span className="text-blue-200">Minimal Intervention</span>
              </div>
            </div>
          </div>

          {/* Five Control Domains */}
          <div className="mb-6">
            <h3 className="text-xl font-semibold text-white mb-4">
              Five Control Domains
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {domainStatuses.map((domain, index) => (
                <DomainCard key={index} domain={domain} />
              ))}
            </div>
          </div>

          {/* Threshold Canon Reference */}
          <div className="bg-gray-900 border border-gray-600 rounded-lg p-4 mb-6">
            <h3 className="text-lg font-semibold text-white mb-3">
              📋 Threshold Canon Application
            </h3>
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-gray-700">
                    <th className="text-left py-2 px-3 text-gray-400 font-semibold">Metric</th>
                    <th className="text-center py-2 px-3 text-gray-400 font-semibold">Pass</th>
                    <th className="text-center py-2 px-3 text-gray-400 font-semibold">Warning</th>
                    <th className="text-center py-2 px-3 text-gray-400 font-semibold">HALT</th>
                  </tr>
                </thead>
                <tbody className="text-gray-300">
                  <tr className="border-b border-gray-800">
                    <td className="py-2 px-3">CI</td>
                    <td className="text-center py-2 px-3 text-green-400">≥ 0.80</td>
                    <td className="text-center py-2 px-3 text-yellow-400">0.70</td>
                    <td className="text-center py-2 px-3 text-red-400">0.50</td>
                  </tr>
                  <tr className="border-b border-gray-800">
                    <td className="py-2 px-3">EV</td>
                    <td className="text-center py-2 px-3 text-green-400">≤ ±10%</td>
                    <td className="text-center py-2 px-3 text-yellow-400">±20%</td>
                    <td className="text-center py-2 px-3 text-red-400">±30%</td>
                  </tr>
                  <tr className="border-b border-gray-800">
                    <td className="py-2 px-3">IAS</td>
                    <td className="text-center py-2 px-3 text-green-400">≥ 0.80</td>
                    <td className="text-center py-2 px-3 text-yellow-400">0.70</td>
                    <td className="text-center py-2 px-3 text-red-400">0.50</td>
                  </tr>
                  <tr className="border-b border-gray-800">
                    <td className="py-2 px-3">EFI</td>
                    <td className="text-center py-2 px-3 text-green-400">≥ 95%</td>
                    <td className="text-center py-2 px-3 text-yellow-400">90%</td>
                    <td className="text-center py-2 px-3 text-red-400">80%</td>
                  </tr>
                  <tr className="border-b border-gray-800">
                    <td className="py-2 px-3">SEC</td>
                    <td className="text-center py-2 px-3 text-green-400">100%</td>
                    <td className="text-center py-2 px-3 text-gray-500">-</td>
                    <td className="text-center py-2 px-3 text-gray-500">-</td>
                  </tr>
                  <tr>
                    <td className="py-2 px-3">PCI</td>
                    <td className="text-center py-2 px-3 text-green-400">≥ 0.90</td>
                    <td className="text-center py-2 px-3 text-yellow-400">0.85</td>
                    <td className="text-center py-2 px-3 text-red-400">0.70</td>
                  </tr>
                </tbody>
              </table>
            </div>
            <p className="text-xs text-gray-400 mt-3">
              All thresholds aligned with Method-VI Threshold Canon
            </p>
          </div>

          {/* Current Metrics Snapshot */}
          {result.metrics && (
            <div className="bg-gray-900 border border-gray-600 rounded-lg p-4 mb-6">
              <h3 className="text-lg font-semibold text-white mb-3">
                📸 Initial Metrics Snapshot
              </h3>
              <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
                {result.metrics.ci !== null && (
                  <MetricBadge label="CI" value={result.metrics.ci.toFixed(2)} color="blue" />
                )}
                {result.metrics.ev !== null && (
                  <MetricBadge label="EV" value={`${result.metrics.ev.toFixed(1)}%`} color="green" />
                )}
                {result.metrics.ias !== null && (
                  <MetricBadge label="IAS" value={result.metrics.ias.toFixed(2)} color="purple" />
                )}
                {result.metrics.efi !== null && (
                  <MetricBadge label="EFI" value={`${result.metrics.efi.toFixed(0)}%`} color="yellow" />
                )}
                {result.metrics.sec !== null && (
                  <MetricBadge label="SEC" value={`${result.metrics.sec.toFixed(0)}%`} color="cyan" />
                )}
                {result.metrics.pci !== null && (
                  <MetricBadge label="PCI" value={result.metrics.pci.toFixed(2)} color="pink" />
                )}
              </div>
            </div>
          )}

          {/* Artifacts Created */}
          <div className="bg-gray-900 border border-gray-600 rounded-lg p-4 mb-6">
            <h3 className="text-lg font-semibold text-white mb-3">
              📄 Artifacts Created
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              <div className="bg-gray-800 border border-gray-700 rounded p-3">
                <div className="flex items-center mb-1">
                  <span className="text-xl mr-2">📊</span>
                  <span className="text-white font-semibold">Governance_Summary</span>
                </div>
                <p className="text-xs text-gray-400 mb-2">
                  Active control settings and threshold configurations
                </p>
                <div className="text-xs text-gray-500">
                  ID: {result.governance_summary_id}
                </div>
              </div>
              <div className="bg-gray-800 border border-gray-700 rounded p-3">
                <div className="flex items-center mb-1">
                  <span className="text-xl mr-2">📸</span>
                  <span className="text-white font-semibold">Domain_Snapshots</span>
                </div>
                <p className="text-xs text-gray-400 mb-2">
                  Baseline readings for five control domains
                </p>
                <div className="text-xs text-gray-500">
                  ID: {result.domain_snapshots_id}
                </div>
              </div>
            </div>
          </div>

          {/* Actions */}
          <div className="flex gap-4">
            <button
              onClick={handleApprove}
              className="flex-1 bg-green-600 hover:bg-green-700 text-white font-semibold py-3 px-6 rounded-lg transition-colors"
            >
              ✓ Approve Governance Calibration
            </button>
            <button
              onClick={() => setViewState('review')}
              className="flex-1 bg-gray-700 hover:bg-gray-600 text-white font-semibold py-3 px-6 rounded-lg transition-colors"
              disabled
            >
              Adjust Configuration (Not Available)
            </button>
          </div>

          <p className="text-sm text-gray-400 mt-4 text-center">
            ⚙️ Governance controls will remain active throughout the run for continuous monitoring.
          </p>
        </div>
      </div>
    );
  };

  const renderApprovedView = () => (
    <div className="max-w-4xl mx-auto p-8">
      <div className="bg-gray-800 border border-gray-700 rounded-lg p-6 text-center">
        <div className="text-6xl mb-4">✓</div>
        <h2 className="text-3xl font-bold text-green-400 mb-2">
          Governance Calibrated
        </h2>
        <p className="text-gray-300 mb-4">
          Five control domains are configured and monitoring is active.
        </p>
        <p className="text-gray-400">
          Transitioning to Step 3...
        </p>
      </div>
    </div>
  );

  const renderErrorView = () => (
    <div className="max-w-4xl mx-auto p-8">
      <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
        <h2 className="text-3xl font-bold text-red-400 mb-4">
          Error Calibrating Governance
        </h2>
        <div className="bg-red-900/20 border border-red-700 rounded-lg p-4 mb-6">
          <p className="text-red-300">{error}</p>
        </div>
        <button
          onClick={executeStep2}
          className="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-6 rounded-lg transition-colors"
        >
          Retry
        </button>
      </div>
    </div>
  );

  switch (viewState) {
    case 'initializing':
    case 'calibrating':
      return renderCalibratingView();
    case 'review':
      return renderReviewView();
    case 'approved':
      return renderApprovedView();
    case 'error':
      return renderErrorView();
    default:
      return null;
  }
}

// Domain Card Component
interface DomainCardProps {
  domain: DomainStatus;
}

function DomainCard({ domain }: DomainCardProps) {
  const statusColors = {
    configured: 'bg-blue-900/30 border-blue-700 text-blue-300',
    measured: 'bg-green-900/30 border-green-700 text-green-300',
    deferred: 'bg-gray-900/30 border-gray-700 text-gray-400',
  };

  const iconColors = {
    blue: 'text-blue-400',
    green: 'text-green-400',
    purple: 'text-purple-400',
    yellow: 'text-yellow-400',
    gray: 'text-gray-400',
  };

  return (
    <div className={`border rounded-lg p-4 ${statusColors[domain.status]}`}>
      <div className="flex items-center mb-2">
        <span className={`text-2xl mr-2 ${iconColors[domain.color as keyof typeof iconColors]}`}>
          {domain.icon}
        </span>
        <h4 className="text-sm font-semibold text-white">{domain.name}</h4>
      </div>

      <div className="space-y-1 text-xs">
        <div>
          <span className="text-gray-400">Metric:</span>{' '}
          <span className="text-white font-mono">{domain.metric}</span>
        </div>
        <div>
          <span className="text-gray-400">Baseline:</span>{' '}
          <span className="text-gray-200">{domain.baseline}</span>
        </div>
        <div>
          <span className="text-gray-400">Target:</span>{' '}
          <span className="text-gray-200">{domain.target}</span>
        </div>
        <div className="pt-1">
          <span className={`inline-block px-2 py-0.5 rounded text-xs font-semibold ${
            domain.status === 'configured' ? 'bg-blue-600 text-white' :
            domain.status === 'measured' ? 'bg-green-600 text-white' :
            'bg-gray-600 text-gray-300'
          }`}>
            {domain.status === 'configured' ? '✓ Configured' :
             domain.status === 'measured' ? '✓ Measured' :
             'Phase 2'}
          </span>
        </div>
      </div>
    </div>
  );
}

// Metric Badge Component
interface MetricBadgeProps {
  label: string;
  value: string;
  color: string;
}

function MetricBadge({ label, value, color }: MetricBadgeProps) {
  const colorClasses = {
    blue: 'bg-blue-900/50 border-blue-700 text-blue-300',
    green: 'bg-green-900/50 border-green-700 text-green-300',
    purple: 'bg-purple-900/50 border-purple-700 text-purple-300',
    yellow: 'bg-yellow-900/50 border-yellow-700 text-yellow-300',
    cyan: 'bg-cyan-900/50 border-cyan-700 text-cyan-300',
    pink: 'bg-pink-900/50 border-pink-700 text-pink-300',
  };

  return (
    <div className={`border rounded-lg p-2 text-center ${colorClasses[color as keyof typeof colorClasses] || colorClasses.blue}`}>
      <div className="text-xs text-gray-400 mb-1">{label}</div>
      <div className="text-lg font-bold">{value}</div>
    </div>
  );
}
