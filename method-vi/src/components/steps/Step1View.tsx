import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Step1ViewProps {
  runId: string;
  onBaselineFrozen: () => void;
}

interface Step1Artifact {
  artifact_id: string;
  artifact_type: string;
  hash: string;
  is_immutable: boolean;
  content_preview: string;
}

interface Step1Result {
  intent_anchor: Step1Artifact;
  charter: Step1Artifact;
  baseline_report: Step1Artifact;
  architecture_map: Step1Artifact;
  e_baseline: number;
}

type ViewState = 'initializing' | 'creating' | 'review' | 'approved' | 'error';

export default function Step1View({ runId, onBaselineFrozen }: Step1ViewProps) {
  const [viewState, setViewState] = useState<ViewState>('initializing');
  const [result, setResult] = useState<Step1Result | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [creationProgress, setCreationProgress] = useState<string[]>([]);

  useEffect(() => {
    executeStep1();
  }, []);

  const executeStep1 = async () => {
    setViewState('creating');
    setError(null);
    setCreationProgress([]);

    try {
      console.log('Executing Step 1...');

      // Simulate progress updates
      setCreationProgress(['Creating Intent_Anchor...']);
      await new Promise(resolve => setTimeout(resolve, 500));

      setCreationProgress(prev => [...prev, 'Creating Charter...']);
      await new Promise(resolve => setTimeout(resolve, 500));

      setCreationProgress(prev => [...prev, 'Calculating and locking E_baseline...']);
      await new Promise(resolve => setTimeout(resolve, 500));

      setCreationProgress(prev => [...prev, 'Creating Baseline_Report...']);
      await new Promise(resolve => setTimeout(resolve, 500));

      setCreationProgress(prev => [...prev, 'Creating Architecture_Map...']);
      await new Promise(resolve => setTimeout(resolve, 500));

      // Call backend to execute Step 1
      const response = await invoke<Step1Result>('execute_step_1', {
        runId,
      });

      console.log('Step 1 result:', response);
      setResult(response);
      setCreationProgress(prev => [...prev, '✓ All artifacts created successfully']);
      setViewState('review');
    } catch (err) {
      console.error('Error executing Step 1:', err);
      setError(`Failed to create baseline: ${err}`);
      setViewState('error');
    }
  };

  const handleApprove = async () => {
    try {
      console.log('Approving baseline...');

      await invoke('approve_gate', {
        approver: 'User',
      });

      setViewState('approved');

      // Notify parent that baseline is frozen
      setTimeout(() => {
        onBaselineFrozen();
      }, 1500);
    } catch (err) {
      console.error('Error approving baseline:', err);
      setError(`Failed to approve baseline: ${err}`);
    }
  };

  const renderCreatingView = () => (
    <div className="max-w-4xl mx-auto p-8">
      <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
        <h2 className="text-3xl font-bold text-white mb-4">
          Step 1: Baseline Establishment
        </h2>
        <p className="text-gray-300 mb-6">
          Creating immutable baseline artifacts...
        </p>

        <div className="space-y-2 mb-6">
          {creationProgress.map((message, index) => (
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

        {viewState === 'creating' && (
          <div className="bg-blue-900/20 border border-blue-700 rounded-lg p-4">
            <p className="text-sm text-blue-300">
              Please wait while we establish the baseline for your run...
            </p>
          </div>
        )}
      </div>
    </div>
  );

  const renderReviewView = () => {
    if (!result) return null;

    return (
      <div className="max-w-6xl mx-auto p-8">
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
          <h2 className="text-3xl font-bold text-white mb-2">
            Step 1: Baseline Establishment
          </h2>
          <p className="text-gray-300 mb-6">
            Review and approve the immutable baseline artifacts
          </p>

          {/* E_baseline Summary */}
          <div className="bg-blue-900/20 border border-blue-700 rounded-lg p-4 mb-6">
            <h3 className="text-lg font-semibold text-blue-300 mb-2">
              📊 E_baseline Locked
            </h3>
            <p className="text-blue-200">
              Baseline content size: <span className="font-bold">{result.e_baseline.toFixed(0)} words</span>
            </p>
            <p className="text-sm text-blue-400 mt-2">
              This value is now immutable and will be used for Expansion Variance (EV) calculations throughout the run.
            </p>
          </div>

          {/* Artifacts Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
            {/* Intent Anchor */}
            <ArtifactCard
              title="Intent_Anchor"
              icon="⚓"
              description="Immutable root of the Coherence Spine"
              artifact={result.intent_anchor}
            />

            {/* Charter */}
            <ArtifactCard
              title="Charter"
              icon="📜"
              description="Governing document with objectives and scope"
              artifact={result.charter}
            />

            {/* Baseline Report */}
            <ArtifactCard
              title="Baseline_Report"
              icon="📊"
              description="Locked E_baseline and governance checkpoints"
              artifact={result.baseline_report}
            />

            {/* Architecture Map */}
            <ArtifactCard
              title="Architecture_Map"
              icon="🗺️"
              description="Process architecture and telemetry configuration"
              artifact={result.architecture_map}
            />
          </div>

          {/* Actions */}
          <div className="flex gap-4">
            <button
              onClick={handleApprove}
              className="flex-1 bg-green-600 hover:bg-green-700 text-white font-semibold py-3 px-6 rounded-lg transition-colors"
            >
              ✓ Approve Baseline
            </button>
            <button
              onClick={() => setViewState('review')}
              className="flex-1 bg-gray-700 hover:bg-gray-600 text-white font-semibold py-3 px-6 rounded-lg transition-colors"
              disabled
            >
              Adjust Baseline (Not Available)
            </button>
          </div>

          <p className="text-sm text-gray-400 mt-4 text-center">
            ⚠️ Once approved, these artifacts become immutable and cannot be changed.
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
          Baseline Approved
        </h2>
        <p className="text-gray-300 mb-4">
          All 4 immutable artifacts have been created and locked.
        </p>
        <p className="text-gray-400">
          Transitioning to Step 2...
        </p>
      </div>
    </div>
  );

  const renderErrorView = () => (
    <div className="max-w-4xl mx-auto p-8">
      <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
        <h2 className="text-3xl font-bold text-red-400 mb-4">
          Error Creating Baseline
        </h2>
        <div className="bg-red-900/20 border border-red-700 rounded-lg p-4 mb-6">
          <p className="text-red-300">{error}</p>
        </div>
        <button
          onClick={executeStep1}
          className="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-6 rounded-lg transition-colors"
        >
          Retry
        </button>
      </div>
    </div>
  );

  switch (viewState) {
    case 'initializing':
    case 'creating':
      return renderCreatingView();
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

// Artifact Card Component
interface ArtifactCardProps {
  title: string;
  icon: string;
  description: string;
  artifact: Step1Artifact;
}

function ArtifactCard({ title, icon, description, artifact }: ArtifactCardProps) {
  return (
    <div className="bg-gray-900 border border-gray-600 rounded-lg p-4">
      <div className="flex items-center mb-2">
        <span className="text-2xl mr-2">{icon}</span>
        <h4 className="text-lg font-semibold text-white">{title}</h4>
      </div>
      <p className="text-sm text-gray-400 mb-3">{description}</p>

      <div className="space-y-1 text-xs">
        <div className="flex justify-between">
          <span className="text-gray-500">Status:</span>
          <span className="text-green-400 font-semibold">
            {artifact.is_immutable ? '🔒 Immutable' : 'Mutable'}
          </span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-500">Hash:</span>
          <span className="text-gray-300 font-mono">{artifact.hash.substring(0, 12)}...</span>
        </div>
      </div>
    </div>
  );
}
