import { useState } from 'react';
import type { GatePreview as GatePreviewData, ArtifactFidelity } from '../types/callouts';

interface GatePreviewProps {
  preview: GatePreviewData;
  onApprove: () => void;
  onRequestChanges: (feedback: string) => void;
  onStartOver: () => void;
}

/**
 * Fidelity badge colors
 */
const FIDELITY_COLORS: Record<ArtifactFidelity, { bg: string; text: string }> = {
  Final: { bg: 'bg-green-900/30', text: 'text-green-400' },
  Draft: { bg: 'bg-yellow-900/30', text: 'text-yellow-400' },
  Placeholder: { bg: 'bg-gray-700/50', text: 'text-gray-400' },
};

export default function GatePreview({
  preview,
  onApprove,
  onRequestChanges,
  onStartOver,
}: GatePreviewProps) {
  const [feedback, setFeedback] = useState('');
  const [showFeedback, setShowFeedback] = useState(false);

  const handleSubmitFeedback = () => {
    if (feedback.trim()) {
      onRequestChanges(feedback);
      setFeedback('');
      setShowFeedback(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* Artifacts created section */}
      <div>
        <h3 className="text-lg font-semibold text-white mb-3 flex items-center gap-2">
          <svg className="w-5 h-5 text-method-vi-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          What I've Created
        </h3>

        {preview.artifacts_created.length > 0 ? (
          <ul className="space-y-2">
            {preview.artifacts_created.map((artifact) => (
              <li
                key={artifact.artifact_key}
                className="flex items-center justify-between p-3 bg-gray-800/50 rounded-lg border border-gray-700"
              >
                <div className="flex items-center gap-3">
                  <span className="text-green-400">&#x2713;</span>
                  <span className="text-gray-200">{artifact.display_name}</span>
                </div>
                <span
                  className={`px-2 py-0.5 text-xs rounded ${FIDELITY_COLORS[artifact.fidelity].bg} ${FIDELITY_COLORS[artifact.fidelity].text}`}
                >
                  {artifact.fidelity}
                </span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-gray-500 italic">No artifacts created yet</p>
        )}
      </div>

      {/* Missing required section - only show if there are missing items */}
      {preview.missing_required.length > 0 && (
        <div className="p-4 bg-yellow-900/20 border border-yellow-600/50 rounded-lg">
          <h4 className="text-yellow-400 font-semibold mb-2 flex items-center gap-2">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            Missing Required Deliverables
          </h4>
          <ul className="space-y-1">
            {preview.missing_required.map((item) => (
              <li key={item} className="text-yellow-200 flex items-center gap-2">
                <span className="text-yellow-500">&#x2022;</span>
                {item}
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Action buttons */}
      <div className="space-y-3 pt-2">
        {!preview.has_hard_blocks ? (
          <button
            onClick={onApprove}
            className="w-full px-4 py-3 bg-method-vi-success text-white rounded-lg hover:bg-green-600 transition-colors font-medium flex items-center justify-center gap-2"
          >
            <span>&#x2713;</span>
            Approve and Continue
          </button>
        ) : (
          <div className="p-4 bg-red-900/20 border border-red-600/50 rounded-lg">
            <p className="text-red-400 font-medium flex items-center gap-2">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
              </svg>
              Cannot proceed until required deliverables are created
            </p>
          </div>
        )}

        <div className="flex gap-3">
          <button
            onClick={() => setShowFeedback(!showFeedback)}
            className="flex-1 px-4 py-2 bg-gray-700 text-white rounded-lg hover:bg-gray-600 transition-colors font-medium flex items-center justify-center gap-2"
          >
            <span>&#x270E;</span>
            Request Changes
          </button>

          <button
            onClick={onStartOver}
            className="flex-1 px-4 py-2 bg-gray-700 text-white rounded-lg hover:bg-gray-600 transition-colors font-medium flex items-center justify-center gap-2"
          >
            <span>&#x21BA;</span>
            Start Over
          </button>
        </div>
      </div>

      {/* Feedback input - shown when Request Changes is clicked */}
      {showFeedback && (
        <div className="p-4 bg-gray-800 rounded-lg border border-gray-700 space-y-3">
          <label className="block text-sm font-medium text-gray-300">
            What changes would you like?
          </label>
          <textarea
            value={feedback}
            onChange={(e) => setFeedback(e.target.value)}
            placeholder="Describe the changes you need..."
            rows={3}
            className="w-full px-4 py-2 bg-gray-900 border border-gray-600 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-method-vi-primary resize-none"
          />
          <div className="flex gap-3">
            <button
              onClick={() => setShowFeedback(false)}
              className="flex-1 px-4 py-2 bg-gray-700 text-white rounded-lg hover:bg-gray-600 transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleSubmitFeedback}
              disabled={!feedback.trim()}
              className="flex-1 px-4 py-2 bg-method-vi-primary text-white rounded-lg hover:bg-purple-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Submit Feedback
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
