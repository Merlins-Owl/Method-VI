import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import MainLayout from '../components/layout/MainLayout';

export default function Home() {
  const navigate = useNavigate();
  const [runLabel, setRunLabel] = useState('');

  const handleNewRun = (e: React.FormEvent) => {
    e.preventDefault();
    if (runLabel.trim()) {
      // In the future, this will call the Tauri backend to create a new run
      const today = new Date().toISOString().split('T')[0];
      const runId = `${today}-${runLabel}`;
      navigate(`/run/${runId}`);
    }
  };

  return (
    <MainLayout showSidebar={false}>
      <div className="flex items-center justify-center min-h-full p-8">
        <div className="max-w-2xl w-full">
          <div className="text-center mb-12">
            <div className="inline-block w-24 h-24 bg-method-vi-primary rounded-2xl flex items-center justify-center mb-6">
              <span className="text-white font-bold text-5xl">M</span>
            </div>
            <h1 className="text-4xl font-bold text-white mb-4">
              Welcome to Method-VI
            </h1>
            <p className="text-xl text-gray-400">
              Structured 7-step reasoning for complex problem solving
            </p>
          </div>

          <div className="bg-gray-900 rounded-lg border border-gray-700 p-8 mb-6">
            <h2 className="text-2xl font-semibold text-white mb-6">Start a New Run</h2>
            <form onSubmit={handleNewRun} className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Run Label
                </label>
                <input
                  type="text"
                  value={runLabel}
                  onChange={(e) => setRunLabel(e.target.value)}
                  placeholder="e.g., Mobile-App-Launch, API-Refactor, Bug-Analysis"
                  className="w-full px-4 py-3 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-method-vi-primary"
                  autoFocus
                />
                <p className="mt-2 text-sm text-gray-500">
                  This label will be combined with today's date to create a unique Run ID
                </p>
              </div>

              <button
                type="submit"
                disabled={!runLabel.trim()}
                className="w-full px-6 py-3 bg-method-vi-primary text-white rounded-lg hover:bg-blue-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed font-semibold text-lg"
              >
                Create New Run →
              </button>
            </form>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <button
              onClick={() => navigate('/sessions')}
              className="p-6 bg-gray-900 border border-gray-700 rounded-lg hover:border-method-vi-primary transition-colors group"
            >
              <div className="text-method-vi-primary text-3xl mb-2">📋</div>
              <h3 className="text-white font-semibold mb-1">Past Sessions</h3>
              <p className="text-sm text-gray-400">Review completed runs</p>
            </button>

            <button
              onClick={() => navigate('/settings')}
              className="p-6 bg-gray-900 border border-gray-700 rounded-lg hover:border-method-vi-primary transition-colors group"
            >
              <div className="text-method-vi-primary text-3xl mb-2">⚙️</div>
              <h3 className="text-white font-semibold mb-1">Settings</h3>
              <p className="text-sm text-gray-400">Configure Method-VI</p>
            </button>
          </div>

          <div className="mt-4">
            <button
              onClick={() => navigate('/metrics-test')}
              className="w-full p-6 bg-blue-500/10 border-2 border-blue-500 rounded-lg hover:bg-blue-500/20 transition-colors"
            >
              <div className="text-blue-500 text-3xl mb-2">📊</div>
              <h3 className="text-blue-500 font-semibold mb-1">Metrics Test Page</h3>
              <p className="text-sm text-blue-400">Verify metrics display and interactions</p>
            </button>
          </div>
        </div>
      </div>
    </MainLayout>
  );
}
