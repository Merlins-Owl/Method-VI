import { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import MainLayout from '../components/layout/MainLayout';
import Step0View from '../components/steps/Step0View';
import Step1View from '../components/steps/Step1View';
import Step2View from '../components/steps/Step2View';
import Step3View from '../components/steps/Step3View';
import Step4View from '../components/steps/Step4View';
import { MetricsState } from '../types/metrics';
import { MOCK_SCENARIOS } from '../utils/mockMetrics';

interface IntentSummary {
  user_intent: string;
  normalized_goal: string;
  success_criteria: string[];
  scope_boundaries: string[];
  assumptions: string[];
}

export default function RunView() {
  const { runId } = useParams<{ runId: string }>();
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
    console.log('[RunView] Gate reached with summary:', summary);

    // Show a confirmation dialog (or use the built-in approval flow)
    const confirmed = window.confirm(
      `Ready to proceed to Step 1?\n\nNormalized Goal: ${summary.normalized_goal}\n\nClick OK to approve, Cancel to go back.`
    );

    if (confirmed) {
      try {
        console.log('[RunView] Calling approve_gate backend command...');
        console.log('[RunView] Current runId:', runId);

        // Call Tauri backend to approve gate
        await invoke('approve_gate', {
          approver: 'User', // In a real app, get this from user profile/settings
        });

        console.log('[RunView] Gate approved successfully');
        console.log('[RunView] Transitioning to Step 1...');
        setCurrentStep(1);
        console.log('[RunView] Current step set to 1');
      } catch (error) {
        console.error('[RunView] Failed to approve gate:', error);
        alert(`Failed to approve gate: ${error}`);
      }
    } else {
      console.log('[RunView] User chose to adjust intent');
    }
  };

  const handleBaselineFrozen = async () => {
    console.log('Baseline frozen, moving to Step 2');
    setCurrentStep(2);
  };

  const handleGovernanceCalibrated = async () => {
    console.log('Governance calibrated, moving to Step 3');
    setCurrentStep(3);
  };

  const handleAnalysisComplete = async () => {
    console.log('Multi-angle analysis complete, moving to Step 4');
    setCurrentStep(4);
  };

  const handleSynthesisComplete = async () => {
    console.log('Synthesis lock-in complete, moving to Step 5');
    setCurrentStep(5);
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
          <Step1View
            runId={runId || ''}
            onBaselineFrozen={handleBaselineFrozen}
          />
        );

      case 2:
        return (
          <Step2View
            runId={runId || ''}
            onGovernanceCalibrated={handleGovernanceCalibrated}
          />
        );

      case 3:
        return (
          <Step3View
            runId={runId || ''}
            onAnalysisComplete={handleAnalysisComplete}
          />
        );

      case 4:
        return (
          <Step4View
            runId={runId || ''}
            onSynthesisComplete={handleSynthesisComplete}
          />
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
