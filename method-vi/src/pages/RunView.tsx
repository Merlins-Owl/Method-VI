import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import MainLayout from '../components/layout/MainLayout';
import Step0View from '../components/steps/Step0View';
import { MetricsState } from '../types/metrics';
import { generateMockMetrics, MOCK_SCENARIOS } from '../utils/mockMetrics';

interface IntentSummary {
  user_intent: string;
  normalized_goal: string;
  success_criteria: string[];
  scope_boundaries: string[];
  assumptions: string[];
}

export default function RunView() {
  const { runId } = useParams<{ runId: string }>();
  const navigate = useNavigate();
  const [currentStep, setCurrentStep] = useState(0);
  const [metrics, setMetrics] = useState<MetricsState>(MOCK_SCENARIOS.step0Start);

  // Update metrics as steps progress
  useEffect(() => {
    if (currentStep === 0) {
      setMetrics(MOCK_SCENARIOS.step0Start);
    } else if (currentStep === 1) {
      setMetrics(MOCK_SCENARIOS.step1Progress);
    } else {
      setMetrics(MOCK_SCENARIOS.allPass);
    }
  }, [currentStep]);

  const handleGateReached = async (summary: IntentSummary) => {
    console.log('Gate reached with summary:', summary);

    // Show a confirmation dialog (or use the built-in approval flow)
    const confirmed = window.confirm(
      `Ready to proceed to Step 1?\n\nNormalized Goal: ${summary.normalized_goal}\n\nClick OK to approve, Cancel to go back.`
    );

    if (confirmed) {
      try {
        // Call Tauri backend to approve gate
        await invoke('approve_gate', {
          approver: 'User', // In a real app, get this from user profile/settings
        });

        console.log('Gate approved, moving to Step 1');
        setCurrentStep(1);

        // For now, show a placeholder for Step 1
        // In future, this will render Step1View component
      } catch (error) {
        console.error('Failed to approve gate:', error);
        alert(`Failed to approve gate: ${error}`);
      }
    } else {
      console.log('User chose to adjust intent');
    }
  };

  // Render step-specific view
  const renderStepView = () => {
    switch (currentStep) {
      case 0:
        return (
          <Step0View
            runId={runId || ''}
            onGateReached={handleGateReached}
          />
        );

      case 1:
        return (
          <div className="max-w-4xl mx-auto p-8">
            <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
              <h2 className="text-3xl font-bold text-white mb-4">
                Step 1: Charter & Baseline
              </h2>
              <p className="text-gray-300">
                This step is not yet implemented.
              </p>
              <p className="text-gray-400 mt-4">
                Coming soon: Charter creation and baseline freezing.
              </p>
            </div>
          </div>
        );

      default:
        return (
          <div className="max-w-4xl mx-auto p-8">
            <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
              <h2 className="text-3xl font-bold text-white mb-4">
                Step {currentStep}
              </h2>
              <p className="text-gray-300">
                This step is not yet implemented.
              </p>
            </div>
          </div>
        );
    }
  };

  return (
    <MainLayout
      runId={runId}
      currentStep={currentStep}
      metrics={metrics}
      onStepClick={(step) => {
        console.log('Navigate to step:', step);
        // For now, only allow navigating back to Step 0
        if (step === 0) {
          setCurrentStep(0);
        }
      }}
    >
      {renderStepView()}
    </MainLayout>
  );
}
