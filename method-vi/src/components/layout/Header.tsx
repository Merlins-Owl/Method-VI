import { Link } from 'react-router-dom';
import { StatusBar } from '../StatusBar';

interface HeaderProps {
  runId?: string;
  currentStep?: number;
}

export default function Header({ runId, currentStep }: HeaderProps) {
  return (
    <header className="bg-gray-900 border-b border-gray-700 px-6 py-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <Link to="/" className="flex items-center space-x-2">
            <div className="w-8 h-8 bg-method-vi-primary rounded-lg flex items-center justify-center">
              <span className="text-white font-bold text-xl">M</span>
            </div>
            <h1 className="text-xl font-bold text-white">Method-VI</h1>
          </Link>
          {runId && (
            <div className="flex items-center space-x-2 text-sm">
              <span className="text-gray-400">Run:</span>
              <span className="text-white font-mono">{runId}</span>
              {currentStep !== undefined && (
                <>
                  <span className="text-gray-400">•</span>
                  <span className="text-method-vi-primary">Step {currentStep}</span>
                </>
              )}
            </div>
          )}
        </div>

        {runId && <StatusBar />}

        <div className="flex items-center space-x-4">
          <Link
            to="/sessions"
            className="text-gray-400 hover:text-white transition-colors"
          >
            Sessions
          </Link>
          <Link
            to="/settings"
            className="text-gray-400 hover:text-white transition-colors"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </Link>
        </div>
      </div>
    </header>
  );
}
