import { ReactNode } from 'react';
import Header from './Header';
import Sidebar from './Sidebar';
import MetricsBar from '../MetricsBar';
import { MetricsState } from '../../types/metrics';

interface MainLayoutProps {
  children: ReactNode;
  runId?: string;
  currentStep?: number;
  metrics?: MetricsState;
  showSidebar?: boolean;
  onStepClick?: (step: number) => void;
}

export default function MainLayout({
  children,
  runId,
  currentStep,
  metrics,
  showSidebar = true,
  onStepClick,
}: MainLayoutProps) {
  return (
    <div className="flex flex-col h-screen bg-gray-800">
      <Header runId={runId} currentStep={currentStep} />

      <div className="flex flex-1 overflow-hidden">
        {showSidebar && (
          <Sidebar currentStep={currentStep} onStepClick={onStepClick} />
        )}

        <main className="flex-1 overflow-y-auto bg-gray-800">
          {children}
        </main>
      </div>

      <MetricsBar metrics={metrics} />
    </div>
  );
}
