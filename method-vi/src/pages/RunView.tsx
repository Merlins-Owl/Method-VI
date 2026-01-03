import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import { calloutApi } from '../utils/calloutApi';
import MainLayout from '../components/layout/MainLayout';
import Step0View from '../components/steps/Step0View';
import Step1View from '../components/steps/Step1View';
import Step2View from '../components/steps/Step2View';
import Step3View from '../components/steps/Step3View';
import Step4View from '../components/steps/Step4View';
import Step5View from '../components/steps/Step5View';
import Step6View from '../components/steps/Step6View';
import Step6_5View from '../components/steps/Step6_5View';
import ClosureView from '../components/steps/ClosureView';
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
  const navigate = useNavigate();
  const [currentStep, setCurrentStep] = useState(0);
  const [isStep6_5, setIsStep6_5] = useState(false); // Track if we're in Step 6.5
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

    // Check if critical callouts have been acknowledged
    try {
      const canProceed = await calloutApi.canProceed();
      if (!canProceed) {
        alert('Please acknowledge Critical callouts before proceeding');
        return;
      }
    } catch (error) {
      console.error('[RunView] Failed to check can_proceed:', error);
      // Continue anyway if check fails (fail-open to prevent blocking)
    }

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

  const handleFrameworkComplete = async () => {
    console.log('Framework architecture complete, moving to Step 6');
    setCurrentStep(6);
  };

  const handleValidationComplete = async (exceptionalResult: boolean = false) => {
    console.log('Validation complete - exceptional result:', exceptionalResult);

    // The approve_gate backend handles routing:
    // - If exceptional (CI ≥ 0.85): routes to Step6_5Active
    // - Otherwise: routes to Completed

    if (exceptionalResult) {
      console.log('Exceptional result detected - transitioning to Step 6.5 Learning Harvest');
      setIsStep6_5(true);  // Show Step6_5View
    } else {
      console.log('Normal result - run completed');
      setCurrentStep(7);  // Show completion screen
    }
  };

  const handleLearningComplete = async () => {
    console.log('Learning harvest complete, run finished');
    setIsStep6_5(false);
    setCurrentStep(7);  // Completed state -> trigger Closure
  };

  const handleClosureComplete = async () => {
    console.log('Closure complete, returning to home');
    navigate('/');  // Return to home page
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

      case 5:
        return (
          <Step5View
            runId={runId || ''}
            onFrameworkComplete={handleFrameworkComplete}
          />
        );

      case 6:
        // Check if we're in Step 6.5 (Learning Harvest) or Step 6 (Validation)
        if (isStep6_5) {
          return (
            <Step6_5View
              runId={runId || ''}
              onLearningComplete={handleLearningComplete}
            />
          );
        } else {
          return (
            <Step6View
              runId={runId || ''}
              onValidationComplete={handleValidationComplete}
            />
          );
        }

      case 7:
        // Closure: Final ledger, archival, and export
        return (
          <ClosureView
            runId={runId || ''}
            onClosureComplete={handleClosureComplete}
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
