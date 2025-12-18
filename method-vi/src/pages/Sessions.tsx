import { useNavigate } from 'react-router-dom';
import MainLayout from '../components/layout/MainLayout';

interface Session {
  run_id: string;
  label: string;
  created_at: string;
  status: 'completed' | 'active' | 'halted';
  final_step: number;
}

// Mock data for demonstration
const MOCK_SESSIONS: Session[] = [
  {
    run_id: '2025-12-17-Mobile-App-Launch',
    label: 'Mobile-App-Launch',
    created_at: '2025-12-17T20:38:19Z',
    status: 'active',
    final_step: 0,
  },
  {
    run_id: '2025-12-16-API-Refactor',
    label: 'API-Refactor',
    created_at: '2025-12-16T14:22:10Z',
    status: 'completed',
    final_step: 6.5,
  },
  {
    run_id: '2025-12-15-Bug-Analysis',
    label: 'Bug-Analysis',
    created_at: '2025-12-15T09:15:30Z',
    status: 'halted',
    final_step: 3,
  },
];

export default function Sessions() {
  const navigate = useNavigate();

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'completed':
        return 'bg-method-vi-success text-white';
      case 'active':
        return 'bg-method-vi-primary text-white';
      case 'halted':
        return 'bg-method-vi-danger text-white';
      default:
        return 'bg-gray-700 text-gray-300';
    }
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleString();
  };

  return (
    <MainLayout showSidebar={false}>
      <div className="max-w-6xl mx-auto p-8">
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold text-white mb-2">Past Sessions</h1>
            <p className="text-gray-400">
              Review and resume your Method-VI runs
            </p>
          </div>
          <button
            onClick={() => navigate('/')}
            className="px-6 py-2 bg-method-vi-primary text-white rounded-lg hover:bg-blue-600 transition-colors font-medium"
          >
            + New Run
          </button>
        </div>

        {MOCK_SESSIONS.length === 0 ? (
          <div className="bg-gray-900 rounded-lg border border-gray-700 p-12 text-center">
            <svg className="w-16 h-16 mx-auto mb-4 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <h3 className="text-xl font-semibold text-white mb-2">No sessions yet</h3>
            <p className="text-gray-400 mb-6">
              Start your first Method-VI run to see it here
            </p>
            <button
              onClick={() => navigate('/')}
              className="px-6 py-2 bg-method-vi-primary text-white rounded-lg hover:bg-blue-600 transition-colors font-medium"
            >
              Create New Run
            </button>
          </div>
        ) : (
          <div className="space-y-4">
            {MOCK_SESSIONS.map((session) => (
              <div
                key={session.run_id}
                className="bg-gray-900 rounded-lg border border-gray-700 p-6 hover:border-method-vi-primary transition-colors cursor-pointer"
                onClick={() => navigate(`/run/${session.run_id}`)}
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center space-x-3 mb-2">
                      <h3 className="text-xl font-semibold text-white">
                        {session.label}
                      </h3>
                      <span className={`px-3 py-1 rounded-full text-xs font-semibold ${getStatusBadge(session.status)}`}>
                        {session.status.toUpperCase()}
                      </span>
                    </div>
                    <p className="text-sm font-mono text-gray-400 mb-2">
                      Run ID: {session.run_id}
                    </p>
                    <div className="flex items-center space-x-6 text-sm text-gray-500">
                      <span>
                        Created: {formatDate(session.created_at)}
                      </span>
                      <span>
                        Final Step: {session.final_step}
                      </span>
                    </div>
                  </div>

                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      navigate(`/run/${session.run_id}`);
                    }}
                    className="px-4 py-2 bg-gray-800 text-white rounded-lg hover:bg-gray-700 transition-colors font-medium"
                  >
                    {session.status === 'active' ? 'Resume' : 'View'}
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </MainLayout>
  );
}
