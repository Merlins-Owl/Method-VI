import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Step6_5Response } from '../../types';

interface Step6_5ViewProps {
  runId: string;
  onLearningComplete: () => void;
}

type ViewState = 'initializing' | 'harvesting' | 'review' | 'completed' | 'error';

export default function Step6_5View({ runId, onLearningComplete }: Step6_5ViewProps) {
  const [viewState, setViewState] = useState<ViewState>('initializing');
  const [result, setResult] = useState<Step6_5Response | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selectedPattern, setSelectedPattern] = useState<number | null>(null);
  const executedRef = useRef(false);

  useEffect(() => {
    // Prevent double execution in React Strict Mode
    if (executedRef.current) {
      console.log('[Step6_5View] Step 6.5 already executed, skipping duplicate call');
      return;
    }
    executedRef.current = true;
    executeStep6_5();
  }, []);

  const executeStep6_5 = async () => {
    setViewState('harvesting');
    setError(null);

    try {
      console.log('[Step6_5View] Starting Step 6.5 learning harvest for runId:', runId);

      const response = await invoke<Step6_5Response>('execute_step_6_5', {
        runId,
      });

      console.log('[Step6_5View] Step 6.5 result received:', response);
      setResult(response);
      setViewState('review');
    } catch (err) {
      console.error('[Step6_5View] Error executing Step 6.5:', err);
      setError(`Failed to execute learning harvest: ${err}`);
      setViewState('error');
    }
  };

  const handleProceedToClosure = async () => {
    setViewState('completed');

    // Auto-proceed to closure after brief display
    setTimeout(() => {
      onLearningComplete();
    }, 1500);
  };

  const getVitalityColor = (efficacy: number): string => {
    if (efficacy >= 0.85) return 'text-green-400';
    if (efficacy >= 0.70) return 'text-yellow-400';
    return 'text-orange-400';
  };

  const getVitalityLabel = (efficacy: number): string => {
    if (efficacy >= 0.85) return 'High Vitality';
    if (efficacy >= 0.70) return 'Medium Vitality';
    return 'Low Vitality';
  };

  const getCategoryIcon = (category: string): string => {
    switch (category.toLowerCase()) {
      case 'success': return '✅';
      case 'failure': return '❌';
      case 'optimization': return '⚡';
      default: return '📝';
    }
  };

  const getCategoryColor = (category: string): string => {
    switch (category.toLowerCase()) {
      case 'success': return 'bg-green-900/30 border-green-700';
      case 'failure': return 'bg-red-900/30 border-red-700';
      case 'optimization': return 'bg-yellow-900/30 border-yellow-700';
      default: return 'bg-gray-900/30 border-gray-700';
    }
  };

  const renderHarvestingView = () => (
    <div className="max-w-5xl mx-auto p-8">
      <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
        <h2 className="text-3xl font-bold text-white mb-4">
          🌱 Step 6.5: Learning Harvest
        </h2>
        <p className="text-gray-300 mb-6">
          Extracting reusable patterns from this exceptional run (CI ≥ 0.85)...
        </p>

        {/* Harvesting Animation */}
        <div className="space-y-3 mb-6">
          {[
            { task: 'Analyzing validation dimensions', icon: '🔍' },
            { task: 'Identifying success patterns', icon: '✨' },
            { task: 'Cataloging failure patterns', icon: '📋' },
            { task: 'Extracting optimization opportunities', icon: '⚡' },
            { task: 'Updating knowledge repository', icon: '💾' },
          ].map((item, index) => (
            <div
              key={index}
              className="bg-purple-900/20 border border-purple-700 rounded-lg p-3 animate-pulse"
            >
              <div className="flex items-center">
                <span className="text-2xl mr-3">{item.icon}</span>
                <span className="text-purple-300 font-semibold">{item.task}</span>
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

  const renderReviewView = () => {
    if (!result) return null;

    return (
      <div className="max-w-6xl mx-auto p-8 space-y-6">
        {/* Header */}
        <div className="bg-gradient-to-r from-purple-900/40 to-blue-900/40 border border-purple-700 rounded-lg p-6">
          <h2 className="text-3xl font-bold text-white mb-3">
            🌱 Learning Harvest Complete
          </h2>
          <p className="text-gray-300 text-lg mb-4">
            {result.knowledge_update}
          </p>

          {/* Harvest Summary */}
          <div className="grid grid-cols-3 gap-4 mt-6">
            <div className="bg-green-900/20 border border-green-700 rounded-lg p-4">
              <div className="text-green-400 text-2xl font-bold">{result.success_count}</div>
              <div className="text-green-300 text-sm">Success Patterns</div>
            </div>
            <div className="bg-red-900/20 border border-red-700 rounded-lg p-4">
              <div className="text-red-400 text-2xl font-bold">{result.failure_count}</div>
              <div className="text-red-300 text-sm">Failure Patterns</div>
            </div>
            <div className="bg-yellow-900/20 border border-yellow-700 rounded-lg p-4">
              <div className="text-yellow-400 text-2xl font-bold">{result.optimization_count}</div>
              <div className="text-yellow-300 text-sm">Optimization Patterns</div>
            </div>
          </div>
        </div>

        {/* Pattern Cards */}
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
          <h3 className="text-xl font-bold text-white mb-4 flex items-center">
            <span className="mr-2">📚</span>
            Extracted Patterns ({result.pattern_cards.length})
          </h3>

          {result.pattern_cards.length === 0 ? (
            <p className="text-gray-400">No patterns extracted.</p>
          ) : (
            <div className="space-y-4">
              {result.pattern_cards.map((card, index) => (
                <div
                  key={card.pattern_id}
                  className={`border rounded-lg p-4 cursor-pointer transition-all ${
                    getCategoryColor(card.category)
                  } ${
                    selectedPattern === index ? 'ring-2 ring-purple-500' : ''
                  }`}
                  onClick={() => setSelectedPattern(selectedPattern === index ? null : index)}
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2">
                        <span className="text-2xl">{getCategoryIcon(card.category)}</span>
                        <h4 className="text-lg font-semibold text-white">{card.pattern_name}</h4>
                        <span className="text-xs px-2 py-1 rounded-full bg-gray-700 text-gray-300">
                          {card.category}
                        </span>
                      </div>
                      <p className="text-gray-300 text-sm mb-2">{card.context}</p>

                      {/* Vitality Indicator */}
                      <div className="flex items-center gap-2">
                        <div className={`text-sm font-semibold ${getVitalityColor(card.efficacy)}`}>
                          {getVitalityLabel(card.efficacy)}
                        </div>
                        <div className="text-xs text-gray-400">
                          • Efficacy: {(card.efficacy * 100).toFixed(0)}%
                        </div>
                        <div className="text-xs text-gray-400">
                          • Reusability: {card.reusability}
                        </div>
                      </div>
                    </div>
                  </div>

                  {/* Expanded Details */}
                  {selectedPattern === index && (
                    <div className="mt-4 pt-4 border-t border-gray-600 space-y-3">
                      <div>
                        <div className="text-xs text-gray-400 mb-1">MECHANICS</div>
                        <div className="text-sm text-gray-300">{card.mechanics}</div>
                      </div>
                      <div>
                        <div className="text-xs text-gray-400 mb-1">RECOMMENDATION</div>
                        <div className="text-sm text-green-300">{card.recommendation}</div>
                      </div>
                      <div className="text-xs text-gray-500">
                        Pattern ID: {card.pattern_id}
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Repository Status */}
        <div className="bg-green-900/20 border border-green-700 rounded-lg p-6">
          <div className="flex items-center gap-3 mb-2">
            <span className="text-3xl">💾</span>
            <div>
              <h3 className="text-lg font-bold text-green-300">Knowledge Repository Updated</h3>
              <p className="text-sm text-green-400">
                All {result.pattern_cards.length} patterns have been stored in the knowledge repository for future reuse.
              </p>
            </div>
          </div>
        </div>

        {/* Proceed to Closure */}
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
          <p className="text-gray-300 mb-4">
            Learning harvest complete. The run will now proceed to Closure.
          </p>
          <button
            onClick={handleProceedToClosure}
            className="w-full bg-purple-600 hover:bg-purple-700 text-white py-3 px-6 rounded-lg font-semibold transition-colors"
          >
            ✓ Proceed to Closure
          </button>
        </div>
      </div>
    );
  };

  const renderCompletedView = () => (
    <div className="max-w-4xl mx-auto p-8">
      <div className="bg-gradient-to-r from-green-900/40 to-blue-900/40 border border-green-700 rounded-lg p-8 text-center">
        <div className="text-6xl mb-4">🎓</div>
        <h2 className="text-3xl font-bold text-white mb-4">
          Learning Harvest Complete!
        </h2>
        <p className="text-gray-300">
          Transitioning to Closure...
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
          onClick={() => executeStep6_5()}
          className="bg-red-600 hover:bg-red-700 text-white py-2 px-4 rounded-lg"
        >
          Retry
        </button>
      </div>
    </div>
  );

  // Render appropriate view based on state
  if (viewState === 'initializing' || viewState === 'harvesting') {
    return renderHarvestingView();
  }

  if (viewState === 'review') {
    return renderReviewView();
  }

  if (viewState === 'completed') {
    return renderCompletedView();
  }

  if (viewState === 'error') {
    return renderErrorView();
  }

  return null;
}
