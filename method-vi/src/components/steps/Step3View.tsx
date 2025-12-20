import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Step3ViewProps {
  runId: string;
  onAnalysisComplete: () => void;
}

interface Step3Response {
  integrated_diagnostic_id: string;
  lens_efficacy_report_id: string;
  integrated_diagnostic: string;
  lens_efficacy_report: string;
  metrics: {
    ci: number | null;
    ev: number | null;
    ias: number | null;
    efi: number | null;
    sec: number | null;
    pci: number | null;
  } | null;
}

interface LensResult {
  lens_name: string;
  analysis: string;
  key_findings: string[];
  efficacy_score: number;
  tokens_used: number;
}

interface LensEfficacyReport {
  lens_results: LensResult[];
  total_insights: number;
  high_value_combinations: number;
  estimated_cost: number;
  actual_cost: number;
}

interface LensProgress {
  name: string;
  icon: string;
  status: 'pending' | 'analyzing' | 'complete';
  efficacy?: number;
}

type ViewState = 'initializing' | 'analyzing' | 'review' | 'approved' | 'error';

export default function Step3View({ runId, onAnalysisComplete }: Step3ViewProps) {
  const [viewState, setViewState] = useState<ViewState>('initializing');
  const [result, setResult] = useState<Step3Response | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [lensProgress, setLensProgress] = useState<LensProgress[]>([
    { name: 'Structural', icon: '🏗️', status: 'pending' },
    { name: 'Thematic', icon: '🎨', status: 'pending' },
    { name: 'Logic', icon: '🧠', status: 'pending' },
    { name: 'Evidence', icon: '📊', status: 'pending' },
    { name: 'Expression', icon: '✍️', status: 'pending' },
    { name: 'Intent', icon: '🎯', status: 'pending' },
  ]);
  const [selectedLens, setSelectedLens] = useState<number>(0);
  const [efficacyReport, setEfficacyReport] = useState<LensEfficacyReport | null>(null);
  const [estimatedCost, setEstimatedCost] = useState<number>(0);
  const [actualCost, setActualCost] = useState<number>(0);
  const executedRef = useRef(false);

  useEffect(() => {
    // Prevent double execution in React Strict Mode
    if (executedRef.current) {
      console.log('[Step3View] Step 3 already executed, skipping duplicate call');
      return;
    }
    executedRef.current = true;
    executeStep3();
  }, []);

  const executeStep3 = async () => {
    setViewState('analyzing');
    setError(null);

    // Estimate cost: 6 lenses × ~1500 tokens input + 1000 tokens output × $3/1M input, $15/1M output
    // Rough estimate: ~$0.10 per run
    setEstimatedCost(0.10);

    try {
      console.log('[Step3View] Starting Step 3 execution for runId:', runId);

      // Call backend to execute Step 3
      console.log('[Step3View] Calling execute_step_3 backend command...');

      const response = await invoke<Step3Response>('execute_step_3', {
        runId,
      });

      console.log('[Step3View] Step 3 result received:', response);
      setResult(response);

      // Parse lens efficacy report
      try {
        const parsedReport = JSON.parse(response.lens_efficacy_report) as LensEfficacyReport;
        setEfficacyReport(parsedReport);

        // Update lens progress with efficacy scores
        const updatedProgress = lensProgress.map(lens => {
          const lensResult = parsedReport.lens_results.find(
            r => r.lens_name.toLowerCase() === lens.name.toLowerCase()
          );
          return {
            ...lens,
            status: 'complete' as const,
            efficacy: lensResult?.efficacy_score,
          };
        });
        setLensProgress(updatedProgress);

        // Calculate actual cost
        setActualCost(parsedReport.actual_cost);
      } catch (e) {
        console.error('[Step3View] Error parsing lens efficacy report:', e);
      }

      setViewState('review');
    } catch (err) {
      console.error('[Step3View] Error executing Step 3:', err);
      setError(`Failed to execute multi-angle analysis: ${err}`);
      setViewState('error');
    }
  };

  const handleApprove = async () => {
    try {
      console.log('Approving diagnostic analysis...');

      await invoke('approve_gate', {
        approver: 'User',
      });

      setViewState('approved');

      // Notify parent that analysis is complete
      setTimeout(() => {
        onAnalysisComplete();
      }, 1500);
    } catch (err) {
      console.error('Error approving analysis:', err);
      setError(`Failed to approve analysis: ${err}`);
    }
  };

  const getLensResults = (): LensResult[] => {
    if (!efficacyReport) return [];
    return efficacyReport.lens_results;
  };

  const renderAnalyzingView = () => (
    <div className="max-w-4xl mx-auto p-8">
      <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
        <h2 className="text-3xl font-bold text-white mb-4">
          Step 3: Multi-Angle Analysis (Six Lenses)
        </h2>
        <p className="text-gray-300 mb-6">
          Applying six analytical lenses to the Charter...
        </p>

        {/* Lens Progress Grid */}
        <div className="grid grid-cols-2 md:grid-cols-3 gap-4 mb-6">
          {lensProgress.map((lens, index) => (
            <div
              key={index}
              className={`border rounded-lg p-4 transition-all ${
                lens.status === 'complete'
                  ? 'bg-green-900/20 border-green-700'
                  : lens.status === 'analyzing'
                  ? 'bg-blue-900/20 border-blue-700 animate-pulse'
                  : 'bg-gray-900/20 border-gray-700'
              }`}
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center">
                  <span className="text-2xl mr-2">{lens.icon}</span>
                  <span className="text-white font-semibold text-sm">{lens.name}</span>
                </div>
                <div>
                  {lens.status === 'complete' && <span className="text-green-400">✓</span>}
                  {lens.status === 'analyzing' && (
                    <div className="animate-spin h-4 w-4 border-2 border-blue-500 border-t-transparent rounded-full"></div>
                  )}
                  {lens.status === 'pending' && <span className="text-gray-500">○</span>}
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Cost Estimate */}
        <div className="bg-blue-900/20 border border-blue-700 rounded-lg p-4">
          <div className="flex items-center justify-between">
            <p className="text-sm text-blue-300">
              Performing six-lens diagnostic analysis with OBSERVER role...
            </p>
            <div className="text-right">
              <div className="text-xs text-blue-400">Estimated Cost</div>
              <div className="text-blue-300 font-mono">${estimatedCost.toFixed(2)}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );

  const renderReviewView = () => {
    if (!result) return null;

    const lensResults = getLensResults();

    return (
      <div className="max-w-6xl mx-auto p-8">
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
          <h2 className="text-3xl font-bold text-white mb-2">
            Step 3: Multi-Angle Analysis Complete
          </h2>
          <p className="text-gray-300 mb-6">
            Review the integrated diagnostic and approve to proceed to synthesis
          </p>

          {/* Analysis Status Banner */}
          <div className="bg-purple-900/20 border border-purple-700 rounded-lg p-4 mb-6">
            <h3 className="text-lg font-semibold text-purple-300 mb-2">
              🔍 Six-Lens Analysis Complete
            </h3>
            <p className="text-purple-200 mb-3">
              All analytical lenses have been applied to the Charter
            </p>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
              <div>
                <span className="text-purple-400">Role:</span>{' '}
                <span className="text-purple-200">OBSERVER</span>
              </div>
              <div>
                <span className="text-purple-400">Lenses Applied:</span>{' '}
                <span className="text-green-400 font-semibold">6/6</span>
              </div>
              <div>
                <span className="text-purple-400">Total Insights:</span>{' '}
                <span className="text-purple-200">{efficacyReport?.total_insights || 0}</span>
              </div>
              <div>
                <span className="text-purple-400">Actual Cost:</span>{' '}
                <span className="text-purple-200 font-mono">${actualCost.toFixed(2)}</span>
              </div>
            </div>
          </div>

          {/* Lens Progress Overview */}
          <div className="mb-6">
            <h3 className="text-xl font-semibold text-white mb-4">
              Six Analytical Lenses
            </h3>
            <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
              {lensProgress.map((lens, index) => (
                <div
                  key={index}
                  className="bg-green-900/20 border border-green-700 rounded-lg p-3"
                >
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center">
                      <span className="text-2xl mr-2">{lens.icon}</span>
                      <span className="text-white font-semibold text-sm">{lens.name}</span>
                    </div>
                    <span className="text-green-400">✓</span>
                  </div>
                  {lens.efficacy !== undefined && (
                    <div className="text-xs">
                      <span className="text-gray-400">Efficacy:</span>{' '}
                      <span className={`font-mono ${
                        lens.efficacy >= 0.8 ? 'text-green-400' :
                        lens.efficacy >= 0.6 ? 'text-yellow-400' :
                        'text-orange-400'
                      }`}>
                        {(lens.efficacy * 100).toFixed(0)}%
                      </span>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>

          {/* Lens Results Tabs */}
          {lensResults.length > 0 && (
            <div className="mb-6">
              <h3 className="text-xl font-semibold text-white mb-4">
                Lens Results
              </h3>

              {/* Tab Headers */}
              <div className="flex space-x-2 mb-4 overflow-x-auto">
                {lensResults.map((lens, index) => (
                  <button
                    key={index}
                    onClick={() => setSelectedLens(index)}
                    className={`px-4 py-2 rounded-t-lg font-semibold text-sm whitespace-nowrap transition-colors ${
                      selectedLens === index
                        ? 'bg-gray-900 text-white border-t border-l border-r border-gray-600'
                        : 'bg-gray-800 text-gray-400 hover:text-gray-200'
                    }`}
                  >
                    {lensProgress.find(p => p.name.toLowerCase() === lens.lens_name.toLowerCase())?.icon}{' '}
                    {lens.lens_name}
                  </button>
                ))}
              </div>

              {/* Tab Content */}
              <div className="bg-gray-900 border border-gray-600 rounded-lg p-4">
                {lensResults[selectedLens] && (
                  <div>
                    <div className="flex items-center justify-between mb-3">
                      <h4 className="text-lg font-semibold text-white">
                        {lensResults[selectedLens].lens_name} Analysis
                      </h4>
                      <div className="flex items-center space-x-4 text-sm">
                        <div>
                          <span className="text-gray-400">Efficacy:</span>{' '}
                          <span className={`font-mono ${
                            lensResults[selectedLens].efficacy_score >= 0.8 ? 'text-green-400' :
                            lensResults[selectedLens].efficacy_score >= 0.6 ? 'text-yellow-400' :
                            'text-orange-400'
                          }`}>
                            {(lensResults[selectedLens].efficacy_score * 100).toFixed(0)}%
                          </span>
                        </div>
                        <div>
                          <span className="text-gray-400">Tokens:</span>{' '}
                          <span className="text-gray-300 font-mono">
                            {lensResults[selectedLens].tokens_used.toLocaleString()}
                          </span>
                        </div>
                      </div>
                    </div>

                    {/* Key Findings */}
                    <div className="mb-4">
                      <h5 className="text-sm font-semibold text-gray-400 mb-2">Key Findings:</h5>
                      <ul className="list-disc list-inside space-y-1">
                        {lensResults[selectedLens].key_findings.map((finding, i) => (
                          <li key={i} className="text-gray-300 text-sm">{finding}</li>
                        ))}
                      </ul>
                    </div>

                    {/* Full Analysis */}
                    <div>
                      <h5 className="text-sm font-semibold text-gray-400 mb-2">Full Analysis:</h5>
                      <div className="bg-gray-800 border border-gray-700 rounded p-3 text-gray-300 text-sm whitespace-pre-wrap max-h-96 overflow-y-auto">
                        {lensResults[selectedLens].analysis}
                      </div>
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Integrated Diagnostic Summary */}
          <div className="bg-gray-900 border border-gray-600 rounded-lg p-4 mb-6">
            <h3 className="text-lg font-semibold text-white mb-3">
              🎯 Integrated Diagnostic Summary
            </h3>
            <div className="bg-gray-800 border border-gray-700 rounded p-3 text-gray-300 text-sm whitespace-pre-wrap max-h-96 overflow-y-auto">
              {result.integrated_diagnostic}
            </div>
          </div>

          {/* Lens Efficacy Report */}
          {efficacyReport && (
            <div className="bg-gray-900 border border-gray-600 rounded-lg p-4 mb-6">
              <h3 className="text-lg font-semibold text-white mb-3">
                📊 Lens Efficacy Analysis
              </h3>
              <div className="grid grid-cols-2 md:grid-cols-3 gap-4 mb-3">
                <div className="bg-gray-800 border border-gray-700 rounded p-3">
                  <div className="text-xs text-gray-400 mb-1">Total Insights</div>
                  <div className="text-2xl font-bold text-white">{efficacyReport.total_insights}</div>
                </div>
                <div className="bg-gray-800 border border-gray-700 rounded p-3">
                  <div className="text-xs text-gray-400 mb-1">High-Value Combinations</div>
                  <div className="text-2xl font-bold text-green-400">{efficacyReport.high_value_combinations}</div>
                </div>
                <div className="bg-gray-800 border border-gray-700 rounded p-3">
                  <div className="text-xs text-gray-400 mb-1">Cost</div>
                  <div className="text-2xl font-bold text-purple-400 font-mono">
                    ${efficacyReport.actual_cost.toFixed(2)}
                  </div>
                </div>
              </div>
              <p className="text-xs text-gray-400">
                Efficacy scores measure the value each lens provided to the overall analysis
              </p>
            </div>
          )}

          {/* Current Metrics Snapshot */}
          {result.metrics && (
            <div className="bg-gray-900 border border-gray-600 rounded-lg p-4 mb-6">
              <h3 className="text-lg font-semibold text-white mb-3">
                📸 Current Metrics Snapshot
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
                  <span className="text-xl mr-2">🎯</span>
                  <span className="text-white font-semibold">Integrated_Diagnostic</span>
                </div>
                <p className="text-xs text-gray-400 mb-2">
                  Cross-lens synthesis of all analytical findings
                </p>
                <div className="text-xs text-gray-500">
                  ID: {result.integrated_diagnostic_id}
                </div>
              </div>
              <div className="bg-gray-800 border border-gray-700 rounded p-3">
                <div className="flex items-center mb-1">
                  <span className="text-xl mr-2">📊</span>
                  <span className="text-white font-semibold">Lens_Efficacy_Report</span>
                </div>
                <p className="text-xs text-gray-400 mb-2">
                  Pattern learning data for lens combinations
                </p>
                <div className="text-xs text-gray-500">
                  ID: {result.lens_efficacy_report_id}
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
              ✓ Approve Analysis - Proceed to Synthesis
            </button>
            <button
              onClick={() => setViewState('review')}
              className="flex-1 bg-gray-700 hover:bg-gray-600 text-white font-semibold py-3 px-6 rounded-lg transition-colors"
              disabled
            >
              Re-run Analysis (Not Available)
            </button>
          </div>

          <p className="text-sm text-gray-400 mt-4 text-center">
            🔍 Ready_for_Synthesis gate - Approve to proceed to Step 4 (Synthesis Lock-In)
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
          Multi-Angle Analysis Approved
        </h2>
        <p className="text-gray-300 mb-4">
          Integrated diagnostic is ready for synthesis lock-in.
        </p>
        <p className="text-gray-400">
          Transitioning to Step 4...
        </p>
      </div>
    </div>
  );

  const renderErrorView = () => (
    <div className="max-w-4xl mx-auto p-8">
      <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
        <h2 className="text-3xl font-bold text-red-400 mb-4">
          Error Performing Analysis
        </h2>
        <div className="bg-red-900/20 border border-red-700 rounded-lg p-4 mb-6">
          <p className="text-red-300">{error}</p>
        </div>
        <button
          onClick={executeStep3}
          className="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-6 rounded-lg transition-colors"
        >
          Retry
        </button>
      </div>
    </div>
  );

  switch (viewState) {
    case 'initializing':
    case 'analyzing':
      return renderAnalyzingView();
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
