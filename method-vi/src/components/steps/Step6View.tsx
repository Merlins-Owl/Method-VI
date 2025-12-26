import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Step6Response, Critical6Scores, DimensionResult } from '../../types';

interface Step6ViewProps {
  runId: string;
  onValidationComplete: (exceptionalResult?: boolean) => void;
}

type ViewState = 'initializing' | 'validating' | 'review' | 'approved' | 'error';

export default function Step6View({ runId, onValidationComplete }: Step6ViewProps) {
  const [viewState, setViewState] = useState<ViewState>('initializing');
  const [result, setResult] = useState<Step6Response | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selectedDimension, setSelectedDimension] = useState<number>(0);
  const executedRef = useRef(false);

  useEffect(() => {
    // Prevent double execution in React Strict Mode
    if (executedRef.current) {
      console.log('[Step6View] Step 6 already executed, skipping duplicate call');
      return;
    }
    executedRef.current = true;
    executeStep6();
  }, []);

  const executeStep6 = async () => {
    setViewState('validating');
    setError(null);

    try {
      console.log('[Step6View] Starting Step 6 validation for runId:', runId);

      const response = await invoke<Step6Response>('execute_step_6', {
        runId,
      });

      console.log('[Step6View] Step 6 result received:', response);
      setResult(response);
      setViewState('review');
    } catch (err) {
      console.error('[Step6View] Error executing Step 6:', err);
      setError(`Failed to execute validation: ${err}`);
      setViewState('error');
    }
  };

  const handleApprove = async () => {
    try {
      console.log('Approving validation results...');

      await invoke('approve_gate', {
        approver: 'User',
      });

      setViewState('approved');

      // Notify parent that validation is complete
      // Pass exceptional_flag to determine routing to Step 6.5 or Closure
      const exceptionalResult = result?.exceptional_flag || false;
      console.log('Exceptional result flag:', exceptionalResult);

      setTimeout(() => {
        onValidationComplete(exceptionalResult);
      }, 1500);
    } catch (err) {
      console.error('Error approving validation:', err);
      setError(`Failed to approve validation: ${err}`);
    }
  };

  const renderValidatingView = () => (
    <div className="max-w-4xl mx-auto p-8">
      <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
        <h2 className="text-3xl font-bold text-white mb-4">
          Step 6: Validation & Assurance
        </h2>
        <p className="text-gray-300 mb-6">
          Running comprehensive validation across 6 dimensions...
        </p>

        {/* Validation Progress */}
        <div className="space-y-3 mb-6">
          {[
            { name: 'Logic Validation', icon: '🔍', desc: 'Testing reasoning chains' },
            { name: 'Semantic Validation', icon: '📚', desc: 'Checking Glossary consistency' },
            { name: 'Clarity Assessment', icon: '💡', desc: 'Measuring readability' },
            { name: 'Evidence Audit', icon: '📊', desc: 'Verifying sources' },
            { name: 'Scope Compliance', icon: '🎯', desc: 'Checking boundaries' },
            { name: 'Process Coherence', icon: '⚙️', desc: 'Validating Architecture Map' },
          ].map((step, index) => (
            <div
              key={index}
              className="bg-blue-900/20 border border-blue-700 rounded-lg p-3 animate-pulse"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center">
                  <span className="text-2xl mr-3">{step.icon}</span>
                  <div>
                    <span className="text-blue-300 font-semibold">{step.name}</span>
                    <p className="text-xs text-blue-400">{step.desc}</p>
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>

        <div className="bg-purple-900/20 border border-purple-700 rounded-lg p-4">
          <p className="text-sm text-purple-300">
            Using Validation & Learning Agent (NEW - created in Step 6) with EXAMINER stance...
          </p>
          <p className="text-xs text-purple-400 mt-2">
            Enforcing Critical 6 metrics: CI, EV, IAS, EFI, SEC, PCI
          </p>
        </div>
      </div>
    </div>
  );

  const renderReviewView = () => {
    if (!result) return null;

    const { validation_outcome, critical_6_scores, dimension_results, exceptional_flag } = result;

    // Determine overall color scheme
    const outcomeColor =
      validation_outcome === 'PASS' ? 'green' :
      validation_outcome === 'WARNING' ? 'yellow' :
      'red';

    return (
      <div className="max-w-7xl mx-auto p-8">
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
          <h2 className="text-3xl font-bold text-white mb-2">
            Step 6: Validation Results
          </h2>
          <p className="text-gray-300 mb-6">
            Review the validation outcome and Critical 6 metrics
          </p>

          {/* Result Banner */}
          <ResultBanner outcome={validation_outcome} outcomeColor={outcomeColor} />

          {/* Critical 6 Metrics Cards */}
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3 mb-6">
            <MetricCard label="CI" value={critical_6_scores.ci} target="≥ 0.80" color="blue" />
            <MetricCard label="EV" value={critical_6_scores.ev} target="± 0.10" color="green" isPercentage={false} />
            <MetricCard label="IAS" value={critical_6_scores.ias} target="≥ 0.80" color="purple" />
            <MetricCard label="EFI" value={critical_6_scores.efi} target="≥ 0.95" color="yellow" />
            <MetricCard label="SEC" value={critical_6_scores.sec} target="= 1.00" color="cyan" />
            <MetricCard label="PCI" value={critical_6_scores.pci} target="≥ 0.90" color="pink" />
          </div>

          {/* Exceptional Flag */}
          {exceptional_flag && (
            <div className="bg-gradient-to-r from-green-900/30 to-emerald-900/30 border border-green-700 rounded-lg p-4 mb-6">
              <div className="flex items-center">
                <span className="text-3xl mr-3">🌟</span>
                <div>
                  <h4 className="text-green-300 font-bold text-lg mb-1">
                    EXCEPTIONAL RESULT
                  </h4>
                  <p className="text-green-200 text-sm">
                    CI ≥ 0.85 detected! Step 6.5 Learning Harvest will be available after approval.
                  </p>
                </div>
              </div>
            </div>
          )}

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
            {/* Validation Dashboard */}
            <ValidationDashboard
              dimensions={dimension_results}
              selectedDimension={selectedDimension}
              setSelectedDimension={setSelectedDimension}
            />

            {/* Critical 6 Summary */}
            <Critical6Summary scores={critical_6_scores} />
          </div>

          {/* Validation Matrix Preview */}
          <div className="bg-gray-900 border border-gray-600 rounded-lg p-5 mb-6">
            <h3 className="text-lg font-semibold text-white mb-3 flex items-center">
              <span className="text-2xl mr-2">📋</span>
              Validation Matrix
            </h3>
            <div className="bg-black border border-gray-700 rounded-lg p-4 max-h-64 overflow-y-auto">
              <pre className="text-gray-300 text-xs whitespace-pre-wrap font-mono">
                {result.validation_matrix}
              </pre>
            </div>
          </div>

          {/* HALT Warnings (if any) */}
          {!critical_6_scores.all_pass && (
            <HALTWarning scores={critical_6_scores} />
          )}

          {/* Next Step Indicator */}
          <NextStepIndicator exceptional={exceptional_flag} allPass={critical_6_scores.all_pass} />

          {/* Actions */}
          <div className="flex gap-4">
            <button
              onClick={handleApprove}
              className="flex-1 bg-green-600 hover:bg-green-700 text-white font-semibold py-3 px-6 rounded-lg transition-colors"
            >
              ✓ Approve Validation - Proceed
            </button>
            <button
              onClick={() => alert('Revision not available in MVP')}
              className="flex-1 bg-gray-700 hover:bg-gray-600 text-white font-semibold py-3 px-6 rounded-lg transition-colors"
              disabled
            >
              Return to Step 5 (Not Available)
            </button>
          </div>

          <p className="text-sm text-gray-400 mt-4 text-center">
            🚀 Validation_Complete gate - Approve to complete run or proceed to learning harvest
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
          Validation Approved
        </h2>
        <p className="text-gray-300 mb-4">
          Validation results have been approved. Run is complete.
        </p>
        <p className="text-gray-400">
          Transitioning to completion...
        </p>
      </div>
    </div>
  );

  const renderErrorView = () => (
    <div className="max-w-4xl mx-auto p-8">
      <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
        <h2 className="text-3xl font-bold text-red-400 mb-4">
          Error During Validation
        </h2>
        <div className="bg-red-900/20 border border-red-700 rounded-lg p-4 mb-6">
          <p className="text-red-300">{error}</p>
        </div>
        <button
          onClick={executeStep6}
          className="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-6 rounded-lg transition-colors"
        >
          Retry
        </button>
      </div>
    </div>
  );

  switch (viewState) {
    case 'initializing':
    case 'validating':
      return renderValidatingView();
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

// Result Banner Component
function ResultBanner({ outcome, outcomeColor }: { outcome: string; outcomeColor: string }) {
  const bgColor = {
    green: 'bg-gradient-to-r from-green-900/30 to-emerald-900/30 border-green-700',
    yellow: 'bg-gradient-to-r from-yellow-900/30 to-amber-900/30 border-yellow-700',
    red: 'bg-gradient-to-r from-red-900/30 to-rose-900/30 border-red-700',
  }[outcomeColor];

  const textColor = {
    green: 'text-green-300',
    yellow: 'text-yellow-300',
    red: 'text-red-300',
  }[outcomeColor];

  const icon = {
    green: '✓',
    yellow: '⚠',
    red: '✗',
  }[outcomeColor];

  return (
    <div className={`${bgColor} border rounded-lg p-6 mb-6`}>
      <div className="flex items-center">
        <span className="text-5xl mr-4">{icon}</span>
        <div>
          <h3 className={`text-2xl font-bold ${textColor} mb-1`}>
            Validation Outcome: {outcome}
          </h3>
          <p className="text-gray-300 text-sm">
            {outcome === 'PASS' && 'All validation dimensions passed. Framework meets quality standards.'}
            {outcome === 'WARNING' && 'Validation passed with minor concerns. Review metrics before proceeding.'}
            {outcome === 'FAIL' && 'Validation failed. Critical metrics below threshold. Review required.'}
          </p>
        </div>
      </div>
    </div>
  );
}

// Metric Card Component
interface MetricCardProps {
  label: string;
  value: number;
  target: string;
  color: string;
  isPercentage?: boolean;
}

function MetricCard({ label, value, target, color, isPercentage = true }: MetricCardProps) {
  const colorClasses = {
    blue: 'bg-blue-900/50 border-blue-700 text-blue-300',
    green: 'bg-green-900/50 border-green-700 text-green-300',
    purple: 'bg-purple-900/50 border-purple-700 text-purple-300',
    yellow: 'bg-yellow-900/50 border-yellow-700 text-yellow-300',
    cyan: 'bg-cyan-900/50 border-cyan-700 text-cyan-300',
    pink: 'bg-pink-900/50 border-pink-700 text-pink-300',
  };

  const displayValue = isPercentage ? `${(value * 100).toFixed(0)}%` : value.toFixed(2);

  return (
    <div className={`border rounded-lg p-3 ${colorClasses[color as keyof typeof colorClasses] || colorClasses.blue}`}>
      <div className="text-xs text-gray-400 mb-1">{label}</div>
      <div className="text-2xl font-bold mb-1">{displayValue}</div>
      <div className="text-xs text-gray-500">{target}</div>
    </div>
  );
}

// Validation Dashboard Component
interface ValidationDashboardProps {
  dimensions: DimensionResult[];
  selectedDimension: number;
  setSelectedDimension: (index: number) => void;
}

function ValidationDashboard({ dimensions, selectedDimension, setSelectedDimension }: ValidationDashboardProps) {
  return (
    <div className="bg-gray-900 border border-gray-600 rounded-lg p-5">
      <h3 className="text-lg font-semibold text-white mb-4">
        📊 Validation Dimensions
      </h3>
      <div className="space-y-2">
        {dimensions.map((dim, index) => (
          <button
            key={index}
            onClick={() => setSelectedDimension(index)}
            className={`w-full text-left p-3 rounded-lg transition-colors ${
              selectedDimension === index
                ? 'bg-blue-900/40 border border-blue-600'
                : 'bg-gray-800 border border-gray-700 hover:bg-gray-750'
            }`}
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center">
                <span className="text-lg mr-2">
                  {dim.status === 'Pass' && '✓'}
                  {dim.status === 'Warning' && '⚠'}
                  {dim.status === 'Fail' && '✗'}
                </span>
                <div>
                  <div className="text-white font-semibold text-sm">{dim.dimension_name}</div>
                  <div className="text-xs text-gray-400">Score: {dim.score.toFixed(2)}</div>
                </div>
              </div>
              <div className={`text-xs px-2 py-1 rounded ${
                dim.status === 'Pass' ? 'bg-green-900/50 text-green-300' :
                dim.status === 'Warning' ? 'bg-yellow-900/50 text-yellow-300' :
                'bg-red-900/50 text-red-300'
              }`}>
                {dim.status}
              </div>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}

// Critical 6 Summary Component
function Critical6Summary({ scores }: { scores: Critical6Scores }) {
  return (
    <div className="bg-gray-900 border border-gray-600 rounded-lg p-5">
      <h3 className="text-lg font-semibold text-white mb-4">
        🎯 Critical 6 Status
      </h3>
      <div className="space-y-3">
        <div className={`p-3 rounded-lg ${scores.all_pass ? 'bg-green-900/30 border border-green-700' : 'bg-red-900/30 border border-red-700'}`}>
          <div className="flex items-center justify-between mb-2">
            <span className="text-white font-semibold">Overall Status</span>
            <span className={`px-3 py-1 rounded text-sm font-bold ${
              scores.all_pass ? 'bg-green-700 text-white' : 'bg-red-700 text-white'
            }`}>
              {scores.all_pass ? 'ALL PASS' : 'FAILURES'}
            </span>
          </div>
          {!scores.all_pass && (
            <p className="text-xs text-gray-300">
              One or more metrics below threshold. Review required before proceeding.
            </p>
          )}
        </div>

        <div className="text-xs text-gray-400 space-y-1">
          <div>CI: Coherence Index ({scores.ci.toFixed(2)})</div>
          <div>EV: Expected Value ({scores.ev.toFixed(2)})</div>
          <div>IAS: Intent Alignment ({scores.ias.toFixed(2)})</div>
          <div>EFI: Efficacy Index ({scores.efi.toFixed(2)})</div>
          <div>SEC: Scope Compliance ({scores.sec.toFixed(2)})</div>
          <div>PCI: Pattern Confidence ({scores.pci.toFixed(2)})</div>
        </div>
      </div>
    </div>
  );
}

// HALT Warning Component
function HALTWarning({ scores }: { scores: Critical6Scores }) {
  const failures = [];

  if (scores.ci < 0.80) failures.push(`CI (${scores.ci.toFixed(2)}) below 0.80`);
  if (scores.ev < -0.10 || scores.ev > 0.10) failures.push(`EV (${scores.ev.toFixed(2)}) outside ± 0.10`);
  if (scores.ias < 0.80) failures.push(`IAS (${scores.ias.toFixed(2)}) below 0.80`);
  if (scores.efi < 0.95) failures.push(`EFI (${scores.efi.toFixed(2)}) below 0.95`);
  if (scores.sec < 1.00) failures.push(`SEC (${scores.sec.toFixed(2)}) below 1.00`);
  if (scores.pci < 0.90) failures.push(`PCI (${scores.pci.toFixed(2)}) below 0.90`);

  if (failures.length === 0) return null;

  return (
    <div className="bg-yellow-900/20 border border-yellow-700 rounded-lg p-4 mb-6">
      <div className="flex items-start">
        <span className="text-3xl mr-3">⚠</span>
        <div className="flex-1">
          <h4 className="text-yellow-300 font-bold mb-2">HALT Condition Detected</h4>
          <p className="text-yellow-200 text-sm mb-3">
            The following metrics are below threshold. Human decision required:
          </p>
          <ul className="text-yellow-100 text-sm space-y-1 list-disc list-inside">
            {failures.map((failure, idx) => (
              <li key={idx}>{failure}</li>
            ))}
          </ul>
          <p className="text-yellow-200 text-xs mt-3 italic">
            Note: HALT conditions warn but don't block gates in MVP. You can approve or return to Step 5.
          </p>
        </div>
      </div>
    </div>
  );
}

// Next Step Indicator Component
function NextStepIndicator({ exceptional, allPass }: { exceptional: boolean; allPass: boolean }) {
  return (
    <div className="bg-blue-900/20 border border-blue-700 rounded-lg p-4 mb-6">
      <div className="flex items-center">
        <span className="text-2xl mr-3">🚀</span>
        <div>
          <h4 className="text-blue-300 font-semibold mb-1">Next Step</h4>
          {exceptional && (
            <p className="text-blue-200 text-sm">
              After approval, Step 6.5 (Learning Harvest) will be available to extract success patterns.
            </p>
          )}
          {!exceptional && allPass && (
            <p className="text-blue-200 text-sm">
              After approval, the run will proceed to Closure (Step 7).
            </p>
          )}
          {!allPass && (
            <p className="text-blue-200 text-sm">
              Consider returning to Step 5 to revise the framework, or approve to complete the run with warnings.
            </p>
          )}
        </div>
      </div>
    </div>
  );
}
