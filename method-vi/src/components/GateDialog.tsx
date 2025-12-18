import { useState } from 'react';

interface GateDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onApprove: (approver: string) => void;
  onReject: (rejector: string, reason: string) => void;
  stepFrom: number;
  stepTo: number;
}

export default function GateDialog({
  isOpen,
  onClose,
  onApprove,
  onReject,
  stepFrom,
  stepTo,
}: GateDialogProps) {
  const [approver, setApprover] = useState('');
  const [rejector, setRejector] = useState('');
  const [rejectionReason, setRejectionReason] = useState('');
  const [showRejectionForm, setShowRejectionForm] = useState(false);

  if (!isOpen) return null;

  const handleApprove = () => {
    if (approver.trim()) {
      onApprove(approver);
      setApprover('');
      onClose();
    }
  };

  const handleReject = () => {
    if (rejector.trim() && rejectionReason.trim()) {
      onReject(rejector, rejectionReason);
      setRejector('');
      setRejectionReason('');
      setShowRejectionForm(false);
      onClose();
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-900 rounded-lg shadow-xl border border-gray-700 max-w-md w-full mx-4">
        <div className="p-6">
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

          <div className="mb-6">
            <div className="flex items-center justify-center space-x-4 text-lg mb-4">
              <span className="text-method-vi-primary font-semibold">Step {stepFrom}</span>
              <svg className="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>
              <span className="text-method-vi-primary font-semibold">Step {stepTo}</span>
            </div>
            <p className="text-gray-400 text-center text-sm">
              Human approval is required to proceed to the next step.
            </p>
          </div>

          {!showRejectionForm ? (
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Approver Name
                </label>
                <input
                  type="text"
                  value={approver}
                  onChange={(e) => setApprover(e.target.value)}
                  placeholder="Your name"
                  className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-method-vi-primary"
                  onKeyPress={(e) => e.key === 'Enter' && handleApprove()}
                />
              </div>

              <div className="flex space-x-3">
                <button
                  onClick={handleApprove}
                  disabled={!approver.trim()}
                  className="flex-1 px-4 py-2 bg-method-vi-success text-white rounded-lg hover:bg-green-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed font-medium"
                >
                  ✓ Approve
                </button>
                <button
                  onClick={() => setShowRejectionForm(true)}
                  className="flex-1 px-4 py-2 bg-method-vi-danger text-white rounded-lg hover:bg-red-600 transition-colors font-medium"
                >
                  ✗ Reject
                </button>
              </div>
            </div>
          ) : (
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Rejector Name
                </label>
                <input
                  type="text"
                  value={rejector}
                  onChange={(e) => setRejector(e.target.value)}
                  placeholder="Your name"
                  className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-method-vi-danger"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Rejection Reason
                </label>
                <textarea
                  value={rejectionReason}
                  onChange={(e) => setRejectionReason(e.target.value)}
                  placeholder="Explain why you're rejecting this step..."
                  rows={3}
                  className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-method-vi-danger resize-none"
                />
              </div>

              <div className="flex space-x-3">
                <button
                  onClick={() => setShowRejectionForm(false)}
                  className="flex-1 px-4 py-2 bg-gray-700 text-white rounded-lg hover:bg-gray-600 transition-colors font-medium"
                >
                  Cancel
                </button>
                <button
                  onClick={handleReject}
                  disabled={!rejector.trim() || !rejectionReason.trim()}
                  className="flex-1 px-4 py-2 bg-method-vi-danger text-white rounded-lg hover:bg-red-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed font-medium"
                >
                  Confirm Rejection
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
