import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Step4Response, GlossaryEntry } from '../../types';

interface Step4ViewProps {
  runId: string;
  onSynthesisComplete: () => void;
}

type ViewState = 'initializing' | 'synthesizing' | 'review' | 'approved' | 'error';

export default function Step4View({ runId, onSynthesisComplete }: Step4ViewProps) {
  const [viewState, setViewState] = useState<ViewState>('initializing');
  const [result, setResult] = useState<Step4Response | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [glossarySearch, setGlossarySearch] = useState('');
  const [selectedPrinciple, setSelectedPrinciple] = useState<number>(0);
  const executedRef = useRef(false);

  useEffect(() => {
    // Prevent double execution in React Strict Mode
    if (executedRef.current) {
      console.log('[Step4View] Step 4 already executed, skipping duplicate call');
      return;
    }
    executedRef.current = true;
    executeStep4();
  }, []);

  const executeStep4 = async () => {
    setViewState('synthesizing');
    setError(null);

    try {
      console.log('[Step4View] Starting Step 4 synthesis for runId:', runId);

      const response = await invoke<Step4Response>('execute_step_4', {
        runId,
      });

      console.log('[Step4View] Step 4 result received:', response);
      setResult(response);
      setViewState('review');
    } catch (err) {
      console.error('[Step4View] Error executing Step 4:', err);
      setError(`Failed to execute synthesis lock-in: ${err}`);
      setViewState('error');
    }
  };

  const handleApprove = async () => {
    try {
      console.log('Approving synthesis...');

      await invoke('approve_gate', {
        approver: 'User',
      });

      setViewState('approved');

      // Notify parent that synthesis is complete
      setTimeout(() => {
        onSynthesisComplete();
      }, 1500);
    } catch (err) {
      console.error('Error approving synthesis:', err);
      setError(`Failed to approve synthesis: ${err}`);
    }
  };

  const parseOperatingPrinciples = (): string[] => {
    if (!result) return [];
    return result.operating_principles.split('\n').filter(p => p.trim());
  };

  const parseLimitations = (): string[] => {
    if (!result) return [];
    return result.limitations.split('\n').filter(l => l.trim());
  };

  const parseGlossary = (): GlossaryEntry[] => {
    if (!result) return [];
    try {
      return JSON.parse(result.glossary) as GlossaryEntry[];
    } catch {
      return [];
    }
  };

  const getGeometryIcon = (geometry: string): string => {
    if (geometry.toLowerCase().includes('linear')) return '→';
    if (geometry.toLowerCase().includes('cyclic')) return '↻';
    if (geometry.toLowerCase().includes('branching')) return '⑂';
    return '◆';
  };

  const getGeometryColor = (geometry: string): string => {
    if (geometry.toLowerCase().includes('linear')) return 'blue';
    if (geometry.toLowerCase().includes('cyclic')) return 'green';
    if (geometry.toLowerCase().includes('branching')) return 'purple';
    return 'gray';
  };

  const filteredGlossary = parseGlossary().filter(entry =>
    entry.term.toLowerCase().includes(glossarySearch.toLowerCase()) ||
    entry.definition.toLowerCase().includes(glossarySearch.toLowerCase())
  );

  const renderSynthesizingView = () => (
    <div className="max-w-4xl mx-auto p-8">
      <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
        <h2 className="text-3xl font-bold text-white mb-4">
          Step 4: Synthesis Lock-In
        </h2>
        <p className="text-gray-300 mb-6">
          Transforming diagnostic insights into a unified, explainable model...
        </p>

        {/* Synthesis Progress */}
        <div className="space-y-3 mb-6">
          {[
            { name: 'Core Thesis Derivation', icon: '🎯' },
            { name: 'Operating Principles Extraction', icon: '⚙️' },
            { name: 'Model Geometry Selection', icon: '📐' },
            { name: 'Causality Mapping', icon: '🗺️' },
            { name: 'North-Star Narrative Authoring', icon: '⭐' },
            { name: 'Glossary Creation', icon: '📚' },
            { name: 'Limitations Documentation', icon: '⚠️' },
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
            Using Analysis & Synthesis Agent with OBSERVER role...
          </p>
        </div>
      </div>
    </div>
  );

  const renderReviewView = () => {
    if (!result) return null;

    const principles = parseOperatingPrinciples();
    const limitations = parseLimitations();
    const glossary = filteredGlossary;
    const geometryIcon = getGeometryIcon(result.model_geometry);
    const geometryColor = getGeometryColor(result.model_geometry);

    return (
      <div className="max-w-7xl mx-auto p-8">
        <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
          <h2 className="text-3xl font-bold text-white mb-2">
            Step 4: Synthesis Lock-In Complete
          </h2>
          <p className="text-gray-300 mb-6">
            Review the synthesized model and approve to proceed to redesign
          </p>

          {/* North-Star Narrative Banner */}
          <div className="bg-gradient-to-r from-yellow-900/30 to-orange-900/30 border border-yellow-700 rounded-lg p-6 mb-6">
            <div className="flex items-start">
              <span className="text-4xl mr-4">⭐</span>
              <div className="flex-1">
                <h3 className="text-xl font-bold text-yellow-300 mb-3">
                  North-Star Narrative
                </h3>
                <p className="text-yellow-100 text-lg leading-relaxed">
                  {result.north_star_narrative}
                </p>
              </div>
            </div>
          </div>

          {/* Core Thesis - Prominently Displayed */}
          <div className="bg-gray-900 border-2 border-purple-600 rounded-lg p-6 mb-6">
            <div className="flex items-start mb-4">
              <span className="text-3xl mr-3">🎯</span>
              <h3 className="text-2xl font-bold text-purple-300">
                Core Thesis
              </h3>
            </div>
            <div className="bg-gray-800 border border-purple-700 rounded-lg p-5">
              <p className="text-white text-lg leading-relaxed">
                {result.core_thesis}
              </p>
            </div>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
            {/* Model Geometry */}
            <div className="bg-gray-900 border border-gray-600 rounded-lg p-5">
              <div className="flex items-center mb-4">
                <span className="text-3xl mr-3">📐</span>
                <h3 className="text-xl font-bold text-white">
                  Model Geometry
                </h3>
              </div>
              <div className={`bg-${geometryColor}-900/20 border border-${geometryColor}-700 rounded-lg p-4`}>
                <div className="flex items-center justify-center mb-3">
                  <span className="text-6xl">{geometryIcon}</span>
                </div>
                <p className="text-gray-300 text-sm">
                  {result.model_geometry}
                </p>
              </div>
            </div>

            {/* Operating Principles */}
            <div className="bg-gray-900 border border-gray-600 rounded-lg p-5">
              <div className="flex items-center mb-4">
                <span className="text-3xl mr-3">⚙️</span>
                <h3 className="text-xl font-bold text-white">
                  Operating Principles
                </h3>
              </div>
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {principles.map((principle, index) => (
                  <button
                    key={index}
                    onClick={() => setSelectedPrinciple(index)}
                    className={`w-full text-left p-3 rounded-lg transition-colors ${
                      selectedPrinciple === index
                        ? 'bg-blue-900/40 border border-blue-600'
                        : 'bg-gray-800 border border-gray-700 hover:bg-gray-750'
                    }`}
                  >
                    <div className="flex items-start">
                      <span className="text-blue-400 mr-2">{index + 1}.</span>
                      <span className="text-gray-300 text-sm flex-1">
                        {principle}
                      </span>
                    </div>
                  </button>
                ))}
              </div>
            </div>
          </div>

          {/* Causal Spine */}
          <div className="bg-gray-900 border border-gray-600 rounded-lg p-5 mb-6">
            <div className="flex items-center mb-4">
              <span className="text-3xl mr-3">🗺️</span>
              <h3 className="text-xl font-bold text-white">
                Causal Spine (Causality Map)
              </h3>
            </div>
            <div className="bg-gray-800 border border-gray-700 rounded-lg p-4 max-h-96 overflow-y-auto">
              <pre className="text-gray-300 text-sm whitespace-pre-wrap font-mono">
                {result.causal_spine}
              </pre>
            </div>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
            {/* Glossary */}
            <div className="bg-gray-900 border border-gray-600 rounded-lg p-5">
              <div className="flex items-center mb-4">
                <span className="text-3xl mr-3">📚</span>
                <h3 className="text-xl font-bold text-white">
                  Glossary ({glossary.length} terms)
                </h3>
              </div>
              <input
                type="text"
                placeholder="Search terms..."
                value={glossarySearch}
                onChange={(e) => setGlossarySearch(e.target.value)}
                className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-white mb-3 focus:outline-none focus:border-blue-500"
              />
              <div className="space-y-3 max-h-80 overflow-y-auto">
                {glossary.map((entry, index) => (
                  <div
                    key={index}
                    className="bg-gray-800 border border-gray-700 rounded-lg p-3"
                  >
                    <div className="font-bold text-cyan-400 mb-1">
                      {entry.term}
                    </div>
                    <div className="text-gray-300 text-sm">
                      {entry.definition}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Limitations */}
            <div className="bg-gray-900 border border-gray-600 rounded-lg p-5">
              <div className="flex items-center mb-4">
                <span className="text-3xl mr-3">⚠️</span>
                <h3 className="text-xl font-bold text-white">
                  Limitations
                </h3>
              </div>
              <div className="space-y-2">
                {limitations.map((limitation, index) => (
                  <div
                    key={index}
                    className="bg-orange-900/20 border border-orange-700 rounded-lg p-3"
                  >
                    <div className="flex items-start">
                      <span className="text-orange-400 mr-2">•</span>
                      <span className="text-gray-300 text-sm flex-1">
                        {limitation}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Metrics Snapshot */}
          {result.metrics && (
            <div className="bg-gray-900 border border-gray-600 rounded-lg p-5 mb-6">
              <h3 className="text-lg font-semibold text-white mb-3">
                📸 Metrics Snapshot
              </h3>
              <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
                {result.metrics.ci !== null && (
                  <MetricBadge label="CI" value={result.metrics.ci.toFixed(2)} color="blue" />
                )}
                {result.metrics.ev !== null && (
                  <MetricBadge label="EV" value={`${result.metrics.ev.toFixed(1)}%`} color="green" />
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
                icon="🎯"
                name="Core_Thesis"
                description="Central claim unifying all insights"
                id={result.core_thesis_id}
              />
              <ArtifactCard
                icon="⭐"
                name="North_Star_Narrative"
                description="Guiding paragraph for all future work"
                id={result.north_star_narrative_id}
              />
              <ArtifactCard
                icon="⚙️"
                name="Operating_Principles"
                description={`${principles.length} governing rules`}
                id={`${runId}-operating-principles`}
              />
              <ArtifactCard
                icon="📐"
                name="Model_Geometry"
                description="Structural pattern selection"
                id={`${runId}-model-geometry`}
              />
              <ArtifactCard
                icon="🗺️"
                name="Causal_Spine"
                description="Element relationships and influences"
                id={`${runId}-causal-spine`}
              />
              <ArtifactCard
                icon="📚"
                name="Glossary"
                description={`${glossary.length} key terms defined`}
                id={`${runId}-glossary`}
              />
            </div>
          </div>

          {/* Actions */}
          <div className="flex gap-4">
            <button
              onClick={handleApprove}
              className="flex-1 bg-green-600 hover:bg-green-700 text-white font-semibold py-3 px-6 rounded-lg transition-colors"
            >
              ✓ Approve Synthesis - Proceed to Redesign
            </button>
            <button
              onClick={() => setViewState('review')}
              className="flex-1 bg-gray-700 hover:bg-gray-600 text-white font-semibold py-3 px-6 rounded-lg transition-colors"
              disabled
            >
              Re-run Synthesis (Not Available)
            </button>
          </div>

          <p className="text-sm text-gray-400 mt-4 text-center">
            🚀 Ready_for_Redesign gate - Approve to proceed to Step 5 (Structure & Redesign)
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
          Synthesis Lock-In Approved
        </h2>
        <p className="text-gray-300 mb-4">
          Model is locked and ready for redesign phase.
        </p>
        <p className="text-gray-400">
          Transitioning to Step 5...
        </p>
      </div>
    </div>
  );

  const renderErrorView = () => (
    <div className="max-w-4xl mx-auto p-8">
      <div className="bg-gray-800 border border-gray-700 rounded-lg p-6">
        <h2 className="text-3xl font-bold text-red-400 mb-4">
          Error Performing Synthesis
        </h2>
        <div className="bg-red-900/20 border border-red-700 rounded-lg p-4 mb-6">
          <p className="text-red-300">{error}</p>
        </div>
        <button
          onClick={executeStep4}
          className="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-6 rounded-lg transition-colors"
        >
          Retry
        </button>
      </div>
    </div>
  );

  switch (viewState) {
    case 'initializing':
    case 'synthesizing':
      return renderSynthesizingView();
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
