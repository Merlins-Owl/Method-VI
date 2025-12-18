import { useState } from 'react';
import MainLayout from '../components/layout/MainLayout';

export default function Settings() {
  const [apiKey, setApiKey] = useState('');
  const [defaultModel, setDefaultModel] = useState('claude-sonnet-4-20250514');
  const [maxTokens, setMaxTokens] = useState('4096');

  const handleSave = (e: React.FormEvent) => {
    e.preventDefault();
    console.log('Saving settings...');
    // In the future, this will call Tauri commands to save settings
    alert('Settings saved! (This is a demo - no actual save yet)');
  };

  return (
    <MainLayout showSidebar={false}>
      <div className="max-w-4xl mx-auto p-8">
        <h1 className="text-3xl font-bold text-white mb-2">Settings</h1>
        <p className="text-gray-400 mb-8">
          Configure Method-VI application settings
        </p>

        <form onSubmit={handleSave} className="space-y-6">
          {/* API Configuration */}
          <div className="bg-gray-900 rounded-lg border border-gray-700 p-6">
            <h2 className="text-xl font-semibold text-white mb-4">
              API Configuration
            </h2>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Anthropic API Key
                </label>
                <input
                  type="password"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder="sk-ant-api03-..."
                  className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-method-vi-primary"
                />
                <p className="mt-2 text-sm text-gray-500">
                  Your API key is stored securely and never sent to external servers
                </p>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Default Model
                </label>
                <select
                  value={defaultModel}
                  onChange={(e) => setDefaultModel(e.target.value)}
                  className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-method-vi-primary"
                >
                  <option value="claude-sonnet-4-20250514">Claude Sonnet 4</option>
                  <option value="claude-opus-4-20250514">Claude Opus 4</option>
                  <option value="claude-haiku-3-20250307">Claude Haiku 3</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Max Tokens
                </label>
                <input
                  type="number"
                  value={maxTokens}
                  onChange={(e) => setMaxTokens(e.target.value)}
                  min="100"
                  max="8192"
                  className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-method-vi-primary"
                />
              </div>
            </div>
          </div>

          {/* Display Preferences */}
          <div className="bg-gray-900 rounded-lg border border-gray-700 p-6">
            <h2 className="text-xl font-semibold text-white mb-4">
              Display Preferences
            </h2>

            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <label className="text-sm font-medium text-gray-300">
                    Enable API Logging
                  </label>
                  <p className="text-sm text-gray-500">
                    Log API calls for debugging
                  </p>
                </div>
                <input
                  type="checkbox"
                  className="w-5 h-5 rounded border-gray-700 bg-gray-800 text-method-vi-primary focus:ring-method-vi-primary"
                  defaultChecked
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <label className="text-sm font-medium text-gray-300">
                    Show Steno-Ledger
                  </label>
                  <p className="text-sm text-gray-500">
                    Display context string in header
                  </p>
                </div>
                <input
                  type="checkbox"
                  className="w-5 h-5 rounded border-gray-700 bg-gray-800 text-method-vi-primary focus:ring-method-vi-primary"
                  defaultChecked
                />
              </div>
            </div>
          </div>

          {/* Save Button */}
          <div className="flex justify-end space-x-3">
            <button
              type="button"
              className="px-6 py-2 bg-gray-700 text-white rounded-lg hover:bg-gray-600 transition-colors font-medium"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="px-6 py-2 bg-method-vi-primary text-white rounded-lg hover:bg-blue-600 transition-colors font-medium"
            >
              Save Settings
            </button>
          </div>
        </form>
      </div>
    </MainLayout>
  );
}
