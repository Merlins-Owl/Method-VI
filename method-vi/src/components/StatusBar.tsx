import React, { useState, useEffect } from 'react';
import { CalloutBadge } from './CalloutBadge';
import { ModeBadge } from './ModeBadge';
import { CalloutPanel } from './CalloutPanel';
import type { CalloutSummary } from '../types/callouts';
import { calloutApi } from '../utils/calloutApi';

interface StatusBarProps {
  className?: string;
  pollInterval?: number;
}

export const StatusBar: React.FC<StatusBarProps> = ({
  className = '',
  pollInterval = 5000
}) => {
  const [summary, setSummary] = useState<CalloutSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [panelOpen, setPanelOpen] = useState(false);

  const fetchSummary = async () => {
    try {
      const data = await calloutApi.getCalloutSummary();
      setSummary(data);
    } catch (error) {
      console.error('Failed to fetch callout summary:', error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchSummary();

    if (pollInterval > 0) {
      const interval = setInterval(fetchSummary, pollInterval);
      return () => clearInterval(interval);
    }
  }, [pollInterval]);

  return (
    <>
      <div className={`flex items-center gap-3 ${className}`}>
        <ModeBadge showDetails />
        <div className="w-px h-6 bg-gray-700" />
        <CalloutBadge
          summary={summary}
          loading={loading}
          onClick={() => setPanelOpen(true)}
        />
      </div>

      <CalloutPanel
        isOpen={panelOpen}
        onClose={() => setPanelOpen(false)}
        onAcknowledge={fetchSummary}
      />
    </>
  );
};

export default StatusBar;
