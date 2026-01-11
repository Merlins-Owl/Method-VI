import { useState, useEffect } from 'react';
import GatePreview from './GatePreview';
import { calloutApi } from '../utils/calloutApi';
import type { GatePreview as GatePreviewData } from '../types/callouts';

interface GateDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onApprove: (approver: string) => void;
  onReject: (rejector: string, reason: string) => void;
  onStartOver?: () => void;
  stepFrom: number;
  stepTo: number;
}

export default function GateDialog({
  isOpen,
  onClose,
  onApprove,
  onReject,
  onStartOver,
  stepFrom,
  stepTo,
}: GateDialogProps) {
  const [approver, setApprover] = useState('');
  const [preview, setPreview] = useState<GatePreviewData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Fetch preview data when dialog opens
  useEffect(() => {
    if (isOpen) {
      setLoading(true);
      setError(null);
      calloutApi.getGatePreview(stepFrom)
        .then((data) => {
          setPreview(data);
          setLoading(false);
        })
        .catch((err) => {
          console.error('Failed to fetch gate preview:', err);
          setError('Failed to load preview');
          setLoading(false);
        });
    }
  }, [isOpen, stepFrom]);

  if (!isOpen) return null;

  const handleApprove = () => {
    if (approver.trim() && preview && !preview.has_hard_blocks) {
      onApprove(approver);
      setApprover('');
      onClose();
    }
  };

  const handleRequestChanges = (feedback: string) => {
    // Submit the feedback via IPC
    calloutApi.submitGateDecision('request_changes', feedback)
      .then(() => {
        onReject('User', feedback);
        onClose();
      })
      .catch((err) => {
        console.error('Failed to submit feedback:', err);
      });
  };

  const handleStartOver = () => {
    calloutApi.submitGateDecision('start_over')
      .then(() => {
        onStartOver?.();
        onClose();
      })
      .catch((err) => {
        console.error('Failed to start over:', err);
      });
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-900 rounded-lg shadow-xl border border-gray-700 max-w-lg w-full mx-4 max-h-[90vh] overflow-y-auto">
        <div className="p-6">
          {/* Header */}
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-xl font-bold text-white">Gate Approval Required</h2>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-white transition-colors"
            >
              <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          {/* Step transition indicator */}
          <div className="mb-6">
            <div className="flex items-center justify-center space-x-4 text-lg mb-4">
              <span className="text-method-vi-primary font-semibold">Step {stepFrom}</span>
              <svg className="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>
              <span className="text-method-vi-primary font-semibold">Step {stepTo}</span>
            </div>
          </div>

          {/* Loading state */}
          {loading && (
            <div className="flex items-center justify-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-method-vi-primary"></div>
              <span className="ml-3 text-gray-400">Loading preview...</span>
            </div>
          )}

          {/* Error state */}
          {error && (
            <div className="p-4 bg-red-900/20 border border-red-600/50 rounded-lg mb-4">
              <p className="text-red-400">{error}</p>
            </div>
          )}

          {/* Preview content */}
          {!loading && !error && preview && (
            <>
              <GatePreview
                preview={preview}
                onApprove={() => {
                  // Show approver name input before approving
                  if (approver.trim()) {
                    handleApprove();
                  }
                }}
                onRequestChanges={handleRequestChanges}
                onStartOver={handleStartOver}
              />

              {/* Approver name input - only show if no hard blocks */}
              {!preview.has_hard_blocks && (
                <div className="mt-6 pt-4 border-t border-gray-700">
                  <label className="block text-sm font-medium text-gray-300 mb-2">
                    Your Name (required for approval)
                  </label>
                  <input
                    type="text"
                    value={approver}
                    onChange={(e) => setApprover(e.target.value)}
                    placeholder="Enter your name to approve"
                    className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-method-vi-primary"
                    onKeyPress={(e) => e.key === 'Enter' && handleApprove()}
                  />
                  {approver.trim() && (
                    <button
                      onClick={handleApprove}
                      className="mt-3 w-full px-4 py-3 bg-method-vi-success text-white rounded-lg hover:bg-green-600 transition-colors font-medium flex items-center justify-center gap-2"
                    >
                      <span>&#x2713;</span>
                      Approve as {approver}
                    </button>
                  )}
                </div>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}
