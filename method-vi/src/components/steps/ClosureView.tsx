import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ClosureResponse } from '../../types';

interface ClosureViewProps {
  runId: string;
  onClosureComplete: () => void;
}

type ViewState = 'initializing' | 'closing' | 'summary' | 'completed' | 'error';

export default function ClosureView({ runId, onClosureComplete }: ClosureViewProps) {
  const [viewState, setViewState] = useState<ViewState>('initializing');
  const [result, setResult] = useState<ClosureResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [expandedSection, setExpandedSection] = useState<string | null>(null);
  const [exporting, setExporting] = useState(false);
  const executedRef = useRef(false);

  useEffect(() => {
    // Prevent double execution in React Strict Mode
    if (executedRef.current) {
      console.log('[ClosureView] Closure already executed, skipping duplicate call');
      return;
    }
    executedRef.current = true;
    executeClosure();
  }, []);

  const executeClosure = async () => {
    setViewState('closing');
    setError(null);

    try {
      console.log('[ClosureView] Starting Closure for runId:', runId);

      const response = await invoke<ClosureResponse>('execute_closure', {
        runId,
      });

      console.log('[ClosureView] Closure result received:', response);
      setResult(response);
      setViewState('summary');
    } catch (err) {
      console.error('[ClosureView] Error executing Closure:', err);
      setError(`Failed to execute Closure: ${err}`);
      setViewState('error');
    }
  };

  const handleExportMarkdown = async () => {
    if (!result) return;

    setExporting(true);
    try {
      console.log('[ClosureView] Exporting Markdown...');

      const markdown = await invoke<string>('export_markdown', {
        runId: result.run_id,
      });

      // Copy to clipboard
      await navigator.clipboard.writeText(markdown);
      alert('Markdown report copied to clipboard!');
      console.log('[ClosureView] Markdown copied to clipboard');
    } catch (err) {
      console.error('[ClosureView] Export failed:', err);
      setError(`Export failed: ${err}`);
    } finally {
      setExporting(false);
    }
  };

  const handleExportJSON = async () => {
    if (!result) return;

    setExporting(true);
    try {
      console.log('[ClosureView] Exporting JSON...');

      const json = await invoke<string>('export_json', {
        runId: result.run_id,
      });

      // Copy to clipboard
      await navigator.clipboard.writeText(json);
      alert('JSON export copied to clipboard!');
      console.log('[ClosureView] JSON copied to clipboard');
    } catch (err) {
      console.error('[ClosureView] Export failed:', err);
      setError(`Export failed: ${err}`);
    } finally {
      setExporting(false);
    }
  };

  const handleComplete = () => {
    setViewState('completed');
    setTimeout(() => {
      onClosureComplete();
    }, 2000);
  };

  const toggleSection = (section: string) => {
    setExpandedSection(expandedSection === section ? null : section);
  };

  const getSuccessIcon = (success: boolean): string => {
    return success ? '✅' : '⚠️';
  };

  const getSuccessColor = (success: boolean): string => {
    return success ? 'text-green-400' : 'text-yellow-400';
  };

  const renderClosingView = () => (
    <div className="max-w-5xl mx-auto p-8">
      <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
        <h2 className="text-3xl font-bold text-white mb-4">
          📦 Closure in Progress
        </h2>
        <p className="text-gray-300 mb-6">
          Finalizing run artifacts and generating closure report...
        </p>

        {/* Closing Animation */}
        <div className="space-y-3 mb-6">
          {[
            { task: 'Generating final ledger', icon: '📋' },
            { task: 'Creating audit trail', icon: '🔍' },
            { task: 'Archiving artifacts', icon: '💾' },
            { task: 'Calculating statistics', icon: '📊' },
            { task: 'Updating database', icon: '🗄️' },
          ].map((item, index) => (
            <div
              key={index}
              className="bg-blue-900/20 border border-blue-700 rounded-lg p-3 animate-pulse"
            >
              <div className="flex items-center">
                <span className="text-2xl mr-3">{item.icon}</span>
                <span className="text-blue-300 font-semibold">{item.task}</span>
              </div>
            </div>
          ))}
        </div>

        <div className="text-center text-gray-400">
          This may take a moment...
        </div>
      </div>
    </div>
  );

  const renderSummaryView = () => {
    if (!result) return null;

    return (
      <div className="max-w-7xl mx-auto p-8 space-y-6">
        {/* Header with Success Status */}
        <div className={`bg-gradient-to-r ${
          result.success
            ? 'from-green-900/40 to-blue-900/40 border-green-700'
            : 'from-yellow-900/40 to-orange-900/40 border-yellow-700'
        } border rounded-lg p-6`}>
          <div className="flex items-center justify-between mb-4">
            <div>
              <h2 className="text-3xl font-bold text-white mb-2 flex items-center gap-3">
                <span>{getSuccessIcon(result.success)}</span>
                Run Complete
              </h2>
              <p className="text-gray-300 text-lg">
                {result.success
                  ? 'All validation criteria met. Run archived successfully.'
                  : 'Run completed with warnings. Review metrics below.'}
              </p>
            </div>
            <div className="text-right">
              <div className={`text-4xl font-bold ${getSuccessColor(result.success)}`}>
                {result.success ? 'SUCCESS' : 'WARNING'}
              </div>
              <div className="text-sm text-gray-400 mt-1">
                Run ID: {result.run_id}
              </div>
            </div>
          </div>

          {/* Quick Stats */}
          <div className="grid grid-cols-4 gap-4 mt-6">
            <div className="bg-blue-900/30 border border-blue-700 rounded-lg p-4">
              <div className="text-blue-400 text-2xl font-bold">{result.statistics.steps_completed}</div>
              <div className="text-blue-300 text-sm">Steps Completed</div>
            </div>
            <div className="bg-purple-900/30 border border-purple-700 rounded-lg p-4">
              <div className="text-purple-400 text-2xl font-bold">{result.statistics.total_signals}</div>
              <div className="text-purple-300 text-sm">Total Signals</div>
            </div>
            <div className="bg-indigo-900/30 border border-indigo-700 rounded-lg p-4">
              <div className="text-indigo-400 text-2xl font-bold">{result.statistics.total_gates}</div>
              <div className="text-indigo-300 text-sm">Gates Passed</div>
            </div>
            <div className="bg-green-900/30 border border-green-700 rounded-lg p-4">
              <div className="text-green-400 text-2xl font-bold">
                {result.statistics.exceptional_run ? 'YES' : 'NO'}
              </div>
              <div className="text-green-300 text-sm">Exceptional Run</div>
            </div>
          </div>
        </div>

        {/* Final Metrics (Critical 6) */}
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
          <h3 className="text-xl font-bold text-white mb-4 flex items-center">
            <span className="mr-2">📊</span>
            Final Metrics (Critical 6)
          </h3>
          <div className="grid grid-cols-3 gap-4">
            {Object.entries(result.final_metrics).map(([key, value]) => (
              <div key={key} className="bg-gray-900/50 border border-gray-600 rounded-lg p-3">
                <div className="text-xs text-gray-400 mb-1">{key.toUpperCase()}</div>
                <div className={`text-2xl font-bold ${
                  value >= 0.85 ? 'text-green-400' :
                  value >= 0.70 ? 'text-yellow-400' :
                  'text-red-400'
                }`}>
                  {value.toFixed(2)}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Archived Artifacts */}
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
          <h3 className="text-xl font-bold text-white mb-4 flex items-center">
            <span className="mr-2">💾</span>
            Archived Artifacts ({result.archived_artifacts.length})
          </h3>
          <div className="space-y-2">
            {result.archived_artifacts.map((artifact, index) => (
              <div
                key={index}
                className="bg-gray-900/50 border border-gray-600 rounded-lg p-3 cursor-pointer hover:bg-gray-700/50 transition-colors"
                onClick={() => toggleSection(`artifact-${index}`)}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <span className="text-xl">📄</span>
                    <div>
                      <div className="font-semibold text-white">{artifact.artifact_type}</div>
                      <div className="text-xs text-gray-400">
                        {artifact.content.length} characters
                      </div>
                    </div>
                  </div>
                  <div className="text-gray-400">
                    {expandedSection === `artifact-${index}` ? '▼' : '▶'}
                  </div>
                </div>
                {expandedSection === `artifact-${index}` && (
                  <div className="mt-3 pt-3 border-t border-gray-600">
                    <pre className="text-xs text-gray-300 overflow-x-auto max-h-64 overflow-y-auto">
                      {artifact.content}
                    </pre>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>

        {/* Audit Trail */}
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
          <h3 className="text-xl font-bold text-white mb-4 flex items-center">
            <span className="mr-2">🔍</span>
            Audit Trail ({result.audit_trail.length} entries)
          </h3>
          <div
            className="bg-gray-900/50 border border-gray-600 rounded-lg p-3 cursor-pointer hover:bg-gray-700/50 transition-colors"
            onClick={() => toggleSection('audit-trail')}
          >
            <div className="flex items-center justify-between">
              <div className="text-white font-semibold">View Complete Audit Trail</div>
              <div className="text-gray-400">
                {expandedSection === 'audit-trail' ? '▼' : '▶'}
              </div>
            </div>
            {expandedSection === 'audit-trail' && (
              <div className="mt-3 pt-3 border-t border-gray-600 space-y-2 max-h-96 overflow-y-auto">
                {result.audit_trail.map((entry, index) => (
                  <div key={index} className="bg-gray-800 border border-gray-700 rounded p-2">
                    <div className="flex items-start justify-between mb-1">
                      <div className="text-sm font-semibold text-blue-400">{entry.entry_type}</div>
                      <div className="text-xs text-gray-500">
                        {new Date(entry.timestamp).toLocaleString()}
                      </div>
                    </div>
                    <div className="text-sm text-gray-300">{entry.description}</div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* Final Ledger */}
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
          <h3 className="text-xl font-bold text-white mb-4 flex items-center">
            <span className="mr-2">📋</span>
            Final Ledger (Steno Format)
          </h3>
          <div
            className="bg-gray-900/50 border border-gray-600 rounded-lg p-3 cursor-pointer hover:bg-gray-700/50 transition-colors"
            onClick={() => toggleSection('final-ledger')}
          >
            <div className="flex items-center justify-between">
              <div className="text-white font-semibold">View Final Ledger</div>
              <div className="text-gray-400">
                {expandedSection === 'final-ledger' ? '▼' : '▶'}
              </div>
            </div>
            {expandedSection === 'final-ledger' && (
              <div className="mt-3 pt-3 border-t border-gray-600">
                <pre className="text-xs text-green-400 font-mono overflow-x-auto max-h-96 overflow-y-auto">
                  {result.final_ledger}
                </pre>
              </div>
            )}
          </div>
        </div>

        {/* Export Options */}
        <div className="bg-gradient-to-r from-blue-900/30 to-purple-900/30 border border-blue-700 rounded-lg p-6">
          <h3 className="text-xl font-bold text-white mb-4 flex items-center">
            <span className="mr-2">📤</span>
            Export Options
          </h3>
          <div className="grid grid-cols-2 gap-4">
            <button
              onClick={handleExportMarkdown}
              disabled={exporting}
              className="bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 text-white py-3 px-6 rounded-lg font-semibold transition-colors flex items-center justify-center gap-2"
            >
              <span>📄</span>
              {exporting ? 'Copying...' : 'Copy Markdown Report'}
            </button>
            <button
              onClick={handleExportJSON}
              disabled={exporting}
              className="bg-purple-600 hover:bg-purple-700 disabled:bg-gray-600 text-white py-3 px-6 rounded-lg font-semibold transition-colors flex items-center justify-center gap-2"
            >
              <span>📦</span>
              {exporting ? 'Copying...' : 'Copy JSON Export'}
            </button>
          </div>
        </div>

        {/* Celebratory Message (for successful runs) */}
        {result.success && (
          <div className="bg-gradient-to-r from-green-900/40 to-emerald-900/40 border border-green-600 rounded-lg p-6 text-center">
            <div className="text-6xl mb-4">🎉</div>
            <h3 className="text-2xl font-bold text-green-300 mb-2">
              Exceptional Work!
            </h3>
            <p className="text-gray-300">
              This run achieved all validation criteria and has been archived for future reference.
              {result.statistics.exceptional_run && (
                <span className="block mt-2 text-green-400 font-semibold">
                  Pattern cards from this run are now available in the knowledge repository!
                </span>
              )}
            </p>
          </div>
        )}

        {/* Complete Button */}
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
          <button
            onClick={handleComplete}
            className="w-full bg-green-600 hover:bg-green-700 text-white py-3 px-6 rounded-lg font-semibold transition-colors"
          >
            ✓ Finish & Return to Home
          </button>
        </div>
      </div>
    );
  };

  const renderCompletedView = () => (
    <div className="max-w-4xl mx-auto p-8">
      <div className="bg-gradient-to-r from-green-900/40 to-blue-900/40 border border-green-700 rounded-lg p-8 text-center">
        <div className="text-6xl mb-4">✅</div>
        <h2 className="text-3xl font-bold text-white mb-4">
          Closure Complete!
        </h2>
        <p className="text-gray-300">
          Returning to home...
        </p>
      </div>
    </div>
  );

  const renderErrorView = () => (
    <div className="max-w-4xl mx-auto p-8">
      <div className="bg-red-900/20 border border-red-700 rounded-lg p-6">
        <h2 className="text-2xl font-bold text-red-400 mb-4">Error</h2>
        <p className="text-gray-300 mb-4">{error}</p>
        <button
          onClick={() => executeClosure()}
          className="bg-red-600 hover:bg-red-700 text-white py-2 px-4 rounded-lg"
        >
          Retry
        </button>
      </div>
    </div>
  );

  // Render appropriate view based on state
  if (viewState === 'initializing' || viewState === 'closing') {
    return renderClosingView();
  }

  if (viewState === 'summary') {
    return renderSummaryView();
  }

  if (viewState === 'completed') {
    return renderCompletedView();
  }

  if (viewState === 'error') {
    return renderErrorView();
  }

  return null;
}
