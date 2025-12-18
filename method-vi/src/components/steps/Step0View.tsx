import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Step0ViewProps {
  runId: string;
  onGateReached: (summary: IntentSummary) => void;
}

interface IntentSummary {
  user_intent: string;
  normalized_goal: string;
  success_criteria: string[];
  scope_boundaries: string[];
  assumptions: string[];
}

interface ClarificationQuestion {
  question: string;
  context: string;
}

interface PatternRecommendation {
  pattern_id: string;
  intent_category: string;
  description: string;
  relevance_score: number;
}

interface Step0Result {
  intent_summary: IntentSummary;
  clarification_questions: ClarificationQuestion[];
  pattern_recommendations: PatternRecommendation[];
}

type ViewState = 'input' | 'processing' | 'review' | 'clarifying';

export default function Step0View({ runId, onGateReached }: Step0ViewProps) {
  const [viewState, setViewState] = useState<ViewState>('input');
  const [intentText, setIntentText] = useState('');
  const [result, setResult] = useState<Step0Result | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [answers, setAnswers] = useState<Record<number, string>>({});

  const handleBeginAnalysis = async () => {
    if (!intentText.trim()) {
      setError('Please describe your intent before proceeding.');
      return;
    }

    setViewState('processing');
    setError(null);

    try {
      console.log('Starting Step 0 with intent:', intentText);

      const response = await invoke<Step0Result>('start_step_0', {
        runId,
        userIntent: intentText,
      });

      console.log('Step 0 result:', response);
      setResult(response);

      // If there are clarification questions, go to clarifying state
      if (response.clarification_questions.length > 0) {
        setViewState('clarifying');
      } else {
        setViewState('review');
      }
    } catch (err) {
      console.error('Error starting Step 0:', err);
      setError(`Failed to process intent: ${err}`);
      setViewState('input');
    }
  };

  const handleSubmitAnswers = async () => {
    if (!result) return;

    setViewState('processing');
    setError(null);

    try {
      // Submit answers to clarification questions
      const response = await invoke<Step0Result>('submit_clarifications', {
        runId,
        answers: Object.values(answers),
      });

      console.log('Updated Step 0 result:', response);
      setResult(response);
      setViewState('review');
    } catch (err) {
      console.error('Error submitting clarifications:', err);
      setError(`Failed to process clarifications: ${err}`);
      setViewState('clarifying');
    }
  };

  const handleApprove = () => {
    if (result) {
      onGateReached(result.intent_summary);
    }
  };

  const handleAdjust = () => {
    setViewState('input');
    setResult(null);
    setAnswers({});
  };

  // Input State
  if (viewState === 'input') {
    return (
      <div className="max-w-4xl mx-auto p-8">
        <div className="mb-8">
          <h2 className="text-3xl font-bold text-white mb-4">
            Step 0: Intent Capture
          </h2>
          <div className="bg-gray-800 border border-gray-700 rounded-lg p-6 mb-6">
            <h3 className="text-lg font-semibold text-white mb-3">
              Welcome to Method-VI
            </h3>
            <p className="text-gray-300 mb-3">
              This first step helps you clearly define what you want to accomplish.
              The AI will:
            </p>
            <ul className="list-disc list-inside text-gray-300 space-y-2 mb-4">
              <li>Understand your goal and normalize it into a clear intent</li>
              <li>Identify success criteria and scope boundaries</li>
              <li>Recommend proven patterns from past successful runs</li>
              <li>Ask clarifying questions if needed</li>
            </ul>
            <p className="text-gray-400 text-sm">
              Take your time to describe what you want. The clearer your intent,
              the better Method-VI can help you achieve it.
            </p>
          </div>

          <div className="mb-6">
            <label className="block text-sm font-medium text-gray-300 mb-3">
              Describe your intent or goal
            </label>
            <textarea
              value={intentText}
              onChange={(e) => setIntentText(e.target.value)}
              placeholder="Describe what you want to accomplish...

Example: 'I want to build a real-time chat application that supports group messaging, file sharing, and video calls. It should be scalable to 10,000 concurrent users and work on both web and mobile platforms.'"
              className="w-full h-64 px-4 py-3 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-method-vi-primary resize-none"
            />
          </div>

          {error && (
            <div className="mb-4 p-4 bg-red-900/20 border border-red-700 rounded-lg text-red-400">
              {error}
            </div>
          )}

          <button
            onClick={handleBeginAnalysis}
            disabled={!intentText.trim()}
            className="w-full px-6 py-3 bg-method-vi-primary text-white rounded-lg hover:bg-blue-600 transition-colors font-medium disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Begin Analysis
          </button>
        </div>
      </div>
    );
  }

  // Processing State
  if (viewState === 'processing') {
    return (
      <div className="max-w-4xl mx-auto p-8">
        <div className="flex flex-col items-center justify-center py-16">
          <div className="animate-spin rounded-full h-16 w-16 border-b-2 border-method-vi-primary mb-4"></div>
          <p className="text-white text-lg font-medium">Processing your intent...</p>
          <p className="text-gray-400 text-sm mt-2">
            The Scope & Pattern Agent is analyzing your goal
          </p>
        </div>
      </div>
    );
  }

  // Clarifying State
  if (viewState === 'clarifying' && result) {
    return (
      <div className="max-w-4xl mx-auto p-8">
        <h2 className="text-3xl font-bold text-white mb-6">
          Clarification Needed
        </h2>

        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6 mb-6">
          <p className="text-gray-300 mb-4">
            To better understand your intent, please answer the following questions:
          </p>

          <div className="space-y-6">
            {result.clarification_questions.map((q, idx) => (
              <div key={idx}>
                <label className="block text-sm font-medium text-white mb-2">
                  {q.question}
                </label>
                {q.context && (
                  <p className="text-sm text-gray-400 mb-2">{q.context}</p>
                )}
                <textarea
                  value={answers[idx] || ''}
                  onChange={(e) => setAnswers({ ...answers, [idx]: e.target.value })}
                  className="w-full h-24 px-4 py-2 bg-gray-900 border border-gray-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-method-vi-primary resize-none"
                  placeholder="Your answer..."
                />
              </div>
            ))}
          </div>
        </div>

        {error && (
          <div className="mb-4 p-4 bg-red-900/20 border border-red-700 rounded-lg text-red-400">
            {error}
          </div>
        )}

        <div className="flex space-x-3">
          <button
            onClick={handleAdjust}
            className="flex-1 px-6 py-3 bg-gray-700 text-white rounded-lg hover:bg-gray-600 transition-colors font-medium"
          >
            ← Back to Intent
          </button>
          <button
            onClick={handleSubmitAnswers}
            disabled={Object.keys(answers).length !== result.clarification_questions.length}
            className="flex-1 px-6 py-3 bg-method-vi-primary text-white rounded-lg hover:bg-blue-600 transition-colors font-medium disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Submit Answers
          </button>
        </div>
      </div>
    );
  }

  // Review State
  if (viewState === 'review' && result) {
    return (
      <div className="max-w-4xl mx-auto p-8">
        <h2 className="text-3xl font-bold text-white mb-6">
          Intent Summary - Ready for Review
        </h2>

        {/* Intent Summary */}
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6 mb-6">
          <h3 className="text-xl font-semibold text-white mb-4">
            📋 Captured Intent
          </h3>

          <div className="space-y-4">
            <div>
              <h4 className="text-sm font-medium text-gray-400 mb-2">Original Intent</h4>
              <p className="text-gray-300">{result.intent_summary.user_intent}</p>
            </div>

            <div>
              <h4 className="text-sm font-medium text-gray-400 mb-2">Normalized Goal</h4>
              <p className="text-white font-medium">{result.intent_summary.normalized_goal}</p>
            </div>

            {result.intent_summary.success_criteria.length > 0 && (
              <div>
                <h4 className="text-sm font-medium text-gray-400 mb-2">Success Criteria</h4>
                <ul className="list-disc list-inside text-gray-300 space-y-1">
                  {result.intent_summary.success_criteria.map((criterion, idx) => (
                    <li key={idx}>{criterion}</li>
                  ))}
                </ul>
              </div>
            )}

            {result.intent_summary.scope_boundaries.length > 0 && (
              <div>
                <h4 className="text-sm font-medium text-gray-400 mb-2">Scope Boundaries</h4>
                <ul className="list-disc list-inside text-gray-300 space-y-1">
                  {result.intent_summary.scope_boundaries.map((boundary, idx) => (
                    <li key={idx}>{boundary}</li>
                  ))}
                </ul>
              </div>
            )}

            {result.intent_summary.assumptions.length > 0 && (
              <div>
                <h4 className="text-sm font-medium text-gray-400 mb-2">Assumptions</h4>
                <ul className="list-disc list-inside text-gray-300 space-y-1">
                  {result.intent_summary.assumptions.map((assumption, idx) => (
                    <li key={idx}>{assumption}</li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        </div>

        {/* Pattern Recommendations */}
        {result.pattern_recommendations.length > 0 && (
          <div className="bg-gray-800 border border-gray-700 rounded-lg p-6 mb-6">
            <h3 className="text-xl font-semibold text-white mb-4">
              💡 Recommended Patterns
            </h3>
            <p className="text-gray-400 text-sm mb-4">
              Based on similar successful runs, these patterns might help:
            </p>
            <div className="space-y-3">
              {result.pattern_recommendations.map((pattern) => (
                <div
                  key={pattern.pattern_id}
                  className="bg-gray-900 border border-gray-700 rounded-lg p-4"
                >
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex-1">
                      <span className="text-xs font-semibold text-method-vi-primary uppercase">
                        {pattern.intent_category}
                      </span>
                      <p className="text-gray-300 mt-1">{pattern.description}</p>
                    </div>
                    <div className="ml-4 text-right">
                      <div className="text-xs text-gray-500">Relevance</div>
                      <div className="text-lg font-semibold text-method-vi-success">
                        {Math.round(pattern.relevance_score * 100)}%
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Gate Actions */}
        <div className="bg-blue-900/20 border border-blue-700 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-white mb-3">
            🚦 Gate: Ready for Step 1
          </h3>
          <p className="text-gray-300 mb-6">
            The intent has been captured and normalized. Review the summary above and decide:
          </p>

          {error && (
            <div className="mb-4 p-4 bg-red-900/20 border border-red-700 rounded-lg text-red-400">
              {error}
            </div>
          )}

          <div className="flex space-x-3">
            <button
              onClick={handleAdjust}
              className="flex-1 px-6 py-3 bg-gray-700 text-white rounded-lg hover:bg-gray-600 transition-colors font-medium"
            >
              ← Adjust Intent
            </button>
            <button
              onClick={handleApprove}
              className="flex-1 px-6 py-3 bg-method-vi-success text-white rounded-lg hover:bg-green-600 transition-colors font-medium"
            >
              ✓ Approve & Continue to Step 1
            </button>
          </div>
        </div>
      </div>
    );
  }

  return null;
}
