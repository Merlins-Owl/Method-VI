import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Step5Response } from '../../types';

interface Step5ViewProps {
  runId: string;
  onFrameworkComplete: () => void;
}

type ViewState = 'initializing' | 'designing' | 'review' | 'approved' | 'error';

export default function Step5View({ runId, onFrameworkComplete }: Step5ViewProps) {
  const [viewState, setViewState] = useState<ViewState>('initializing');
  const [result, setResult] = useState<Step5Response | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selectedSection, setSelectedSection] = useState<number>(0);
  const executedRef = useRef(false);

  useEffect(() => {
    // Prevent double execution in React Strict Mode
    if (executedRef.current) {
      console.log('[Step5View] Step 5 already executed, skipping duplicate call');
      return;
    }
    executedRef.current = true;
    executeStep5();
  }, []);

  const executeStep5 = async () => {
    setViewState('designing');
    setError(null);

    try {
      console.log('[Step5View] Starting Step 5 framework design for runId:', runId);

      const response = await invoke<Step5Response>('execute_step_5', {
        runId,
      });

      console.log('[Step5View] Step 5 result received:', response);
      setResult(response);
      setViewState('review');
    } catch (err) {
      console.error('[Step5View] Error executing Step 5:', err);
      setError(`Failed to execute framework design: ${err}`);
      setViewState('error');
    }
  };

  const handleApprove = async () => {
    try {
      console.log('Approving framework architecture...');

      await invoke('approve_gate', {
        approver: 'User',
      });

      setViewState('approved');

      // Notify parent that framework is complete
      setTimeout(() => {
        onFrameworkComplete();
      }, 1500);
    } catch (err) {
      console.error('Error approving framework:', err);
      setError(`Failed to approve framework: ${err}`);
    }
  };

  const extractSections = (): { title: string; content: string }[] => {
    if (!result) return [];

    const architecture = result.framework_architecture;
    const sections: { title: string; content: string }[] = [];

    // Parse sections from the framework architecture
    // Look for section headers like "### Section N:" or "## Section N:"
    const sectionRegex = /###?\s+(Section\s+\d+|SECTION\s+\d+)[:\s]+([^\n]+)/gi;
    const matches = Array.from(architecture.matchAll(sectionRegex));

    if (matches.length > 0) {
      matches.forEach((match, index) => {
        const title = match[2].trim();
        const startIdx = match.index! + match[0].length;
        const endIdx = matches[index + 1]?.index || architecture.length;
        const content = architecture.substring(startIdx, endIdx).trim();

        sections.push({ title, content: content.substring(0, 300) + (content.length > 300 ? '...' : '') });
      });
    }

    return sections;
  };

  const renderDesigningView = () => (
    <div className="max-w-4xl mx-auto p-8">
      <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
        <h2 className="text-3xl font-bold text-white mb-4">
          Step 5: Structure & Redesign
        </h2>
        <p className="text-gray-300 mb-6">
          Designing framework architecture from synthesis artifacts...
        </p>

        {/* Framework Design Progress */}
        <div className="space-y-3 mb-6">
          {[
            { name: 'Framework Overview Design', icon: '🏗️' },
            { name: 'Section Boundary Definition', icon: '📐' },
            { name: 'Section Function Mapping', icon: '🗺️' },
            { name: 'Content Organization', icon: '📋' },
            { name: 'Transition Logic Design', icon: '🔄' },
            { name: 'Governance Coherence Audit', icon: '✓' },
            { name: 'Architecture Outline Creation', icon: '📊' },
          ].map((step, index) => (
            <div
              key={index}
              className="bg-blue-900/20 border border-blue-700 rounded-lg p-3 animate-pulse"
            >
              <div className="flex items-center">
                <span className="text-2xl mr-3">{step.icon}</span>
                <span className="text-blue-300">{step.name}...</span>
              </div>
            </div>
          ))}
        </div>

        <div className="bg-purple-900/20 border border-purple-700 rounded-lg p-4">
          <p className="text-sm text-purple-300">
            Using Structure & Redesign Agent (REUSED from Step 1) with AUDITOR role...
          </p>
          <p className="text-xs text-purple-400 mt-2">
            Core Thesis is IMMUTABLE - framework architecture serves the synthesis
          </p>
        </div>
      </div>
    </div>
  );

  const renderReviewView = () => {
    if (!result) return null;

    const sections = extractSections();

    return (
      <div className="max-w-7xl mx-auto p-8">
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
          <h2 className="text-3xl font-bold text-white mb-2">
            Step 5: Framework Architecture Complete
          </h2>
          <p className="text-gray-300 mb-6">
            Review the framework design and approve to proceed to validation
          </p>

          {/* Framework Architecture Banner */}
          <div className="bg-gradient-to-r from-purple-900/30 to-blue-900/30 border border-purple-700 rounded-lg p-6 mb-6">
            <div className="flex items-start">
              <span className="text-4xl mr-4">🏗️</span>
              <div className="flex-1">
                <h3 className="text-xl font-bold text-purple-300 mb-3">
                  Framework Architecture
                </h3>
                <div className="bg-gray-900 border border-purple-700 rounded-lg p-4 max-h-96 overflow-y-auto">
                  <pre className="text-purple-100 text-sm whitespace-pre-wrap font-mono">
                    {result.framework_architecture}
                  </pre>
                </div>
              </div>
            </div>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
            {/* Section Preview */}
            {sections.length > 0 && (
              <div className="bg-gray-900 border border-gray-600 rounded-lg p-5">
                <div className="flex items-center mb-4">
                  <span className="text-3xl mr-3">📋</span>
                  <h3 className="text-xl font-bold text-white">
                    Sections ({sections.length})
                  </h3>
                </div>
                <div className="space-y-2 max-h-64 overflow-y-auto">
                  {sections.map((section, index) => (
                    <button
                      key={index}
                      onClick={() => setSelectedSection(index)}
                      className={`w-full text-left p-3 rounded-lg transition-colors ${
                        selectedSection === index
                          ? 'bg-purple-900/40 border border-purple-600'
                          : 'bg-gray-800 border border-gray-700 hover:bg-gray-750'
                      }`}
                    >
                      <div className="flex items-start">
                        <span className="text-purple-400 mr-2">{index + 1}.</span>
                        <div className="flex-1">
                          <div className="text-white font-semibold text-sm mb-1">
                            {section.title}
                          </div>
                          <div className="text-gray-400 text-xs">
                            {section.content.substring(0, 80)}...
                          </div>
                        </div>
                      </div>
                    </button>
                  ))}
                </div>
              </div>
            )}

            {/* Framework Type & Info */}
            <div className="bg-gray-900 border border-gray-600 rounded-lg p-5">
              <div className="flex items-center mb-4">
                <span className="text-3xl mr-3">📐</span>
                <h3 className="text-xl font-bold text-white">
                  Framework Overview
                </h3>
              </div>
              <div className="space-y-3">
                <div className="bg-gray-800 border border-gray-700 rounded-lg p-3">
                  <div className="text-xs text-gray-400 mb-1">Execution Mode</div>
                  <div className="text-white font-semibold">Standard</div>
                </div>
                <div className="bg-gray-800 border border-gray-700 rounded-lg p-3">
                  <div className="text-xs text-gray-400 mb-1">Governance Role</div>
                  <div className="text-white font-semibold">Auditor</div>
                </div>
                <div className="bg-gray-800 border border-gray-700 rounded-lg p-3">
                  <div className="text-xs text-gray-400 mb-1">Agent Used</div>
                  <div className="text-purple-300 font-semibold text-sm">
                    Structure & Redesign (Step 1)
                  </div>
                </div>
                {sections.length > 0 && (
                  <div className="bg-gray-800 border border-gray-700 rounded-lg p-3">
                    <div className="text-xs text-gray-400 mb-1">Section Count</div>
                    <div className="text-white font-semibold">{sections.length}</div>
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* Metrics */}
          {result.metrics && (
            <div className="bg-gray-900 border border-gray-600 rounded-lg p-5 mb-6">
              <h3 className="text-lg font-semibold text-white mb-3">
                📊 Critical Metrics
              </h3>
              <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
                {result.metrics.ci !== null && (
                  <MetricBadge label="CI" value={result.metrics.ci.toFixed(2)} color="blue" />
                )}
                {result.metrics.ev !== null && (
                  <MetricBadge label="EV" value={`${result.metrics.ev > 0 ? '+' : ''}${result.metrics.ev.toFixed(1)}%`} color="green" />
                )}
                {result.metrics.ias !== null && (
                  <MetricBadge label="IAS" value={result.metrics.ias.toFixed(2)} color="purple" />
                )}
                {result.metrics.efi !== null && (
                  <MetricBadge label="EFI" value={`${result.metrics.efi.toFixed(0)}%`} color="yellow" />
                )}
                {result.metrics.sec !== null && (
                  <MetricBadge label="SEC" value={`${result.metrics.sec.toFixed(0)}%`} color="cyan" />
                )}
                {result.metrics.pci !== null && (
                  <MetricBadge label="PCI" value={result.metrics.pci.toFixed(2)} color="pink" />
                )}
              </div>
            </div>
          )}

          {/* Artifacts Created */}
          <div className="bg-gray-900 border border-gray-600 rounded-lg p-5 mb-6">
            <h3 className="text-lg font-semibold text-white mb-3">
              📄 Artifacts Created
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              <ArtifactCard
                icon="🏗️"
                name="Framework_Architecture"
                description="Complete framework design with sections and transitions"
                id={result.framework_architecture_id}
              />
            </div>
          </div>

          {/* Governance Audit Note */}
          <div className="bg-blue-900/20 border border-blue-700 rounded-lg p-4 mb-6">
            <div className="flex items-start">
              <span className="text-2xl mr-3">✓</span>
              <div>
                <h4 className="text-blue-300 font-semibold mb-1">Governance Coherence Audit</h4>
                <p className="text-blue-200 text-sm">
                  Framework architecture has been audited for Purpose Coherence (PCI),
                  Content Identity (CI), Structural Coherence (SEC), and Glossary Respect (GLR).
                </p>
              </div>
            </div>
          </div>

          {/* Actions */}
          <div className="flex gap-4">
            <button
              onClick={handleApprove}
              className="flex-1 bg-green-600 hover:bg-green-700 text-white font-semibold py-3 px-6 rounded-lg transition-colors"
            >
              ✓ Approve Framework - Proceed to Validation
            </button>
            <button
              onClick={() => setViewState('review')}
              className="flex-1 bg-gray-700 hover:bg-gray-600 text-white font-semibold py-3 px-6 rounded-lg transition-colors"
              disabled
            >
              Revise Framework (Not Available)
            </button>
          </div>

          <p className="text-sm text-gray-400 mt-4 text-center">
            🚀 Ready_for_Validation gate - Approve to proceed to Step 6 (Validation & Assurance)
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
          Framework Architecture Approved
        </h2>
        <p className="text-gray-300 mb-4">
          Framework design is locked and ready for validation phase.
        </p>
        <p className="text-gray-400">
          Transitioning to Step 6...
        </p>
      </div>
    </div>
  );

  const renderErrorView = () => (
    <div className="max-w-4xl mx-auto p-8">
      <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
        <h2 className="text-3xl font-bold text-red-400 mb-4">
          Error Designing Framework
        </h2>
        <div className="bg-red-900/20 border border-red-700 rounded-lg p-4 mb-6">
          <p className="text-red-300">{error}</p>
        </div>
        <button
          onClick={executeStep5}
          className="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-6 rounded-lg transition-colors"
        >
          Retry
        </button>
      </div>
    </div>
  );

  switch (viewState) {
    case 'initializing':
    case 'designing':
      return renderDesigningView();
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

// Metric Badge Component
interface MetricBadgeProps {
  label: string;
  value: string;
  color: string;
}

function MetricBadge({ label, value, color }: MetricBadgeProps) {
  const colorClasses = {
    blue: 'bg-blue-900/50 border-blue-700 text-blue-300',
    green: 'bg-green-900/50 border-green-700 text-green-300',
    purple: 'bg-purple-900/50 border-purple-700 text-purple-300',
    yellow: 'bg-yellow-900/50 border-yellow-700 text-yellow-300',
    cyan: 'bg-cyan-900/50 border-cyan-700 text-cyan-300',
    pink: 'bg-pink-900/50 border-pink-700 text-pink-300',
  };

  return (
    <div className={`border rounded-lg p-2 text-center ${colorClasses[color as keyof typeof colorClasses] || colorClasses.blue}`}>
      <div className="text-xs text-gray-400 mb-1">{label}</div>
      <div className="text-lg font-bold">{value}</div>
    </div>
  );
}

// Artifact Card Component
interface ArtifactCardProps {
  icon: string;
  name: string;
  description: string;
  id: string;
}

function ArtifactCard({ icon, name, description, id }: ArtifactCardProps) {
  return (
    <div className="bg-gray-800 border border-gray-700 rounded p-3">
      <div className="flex items-center mb-1">
        <span className="text-xl mr-2">{icon}</span>
        <span className="text-white font-semibold">{name}</span>
      </div>
      <p className="text-xs text-gray-400 mb-2">
        {description}
      </p>
      <div className="text-xs text-gray-500">
        ID: {id}
      </div>
    </div>
  );
}
