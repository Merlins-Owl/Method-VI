import { STEPS } from '../../types';

interface SidebarProps {
  currentStep?: number;
  onStepClick?: (step: number) => void;
}

export default function Sidebar({ currentStep = 0, onStepClick }: SidebarProps) {
  return (
    <aside className="w-64 bg-gray-900 border-r border-gray-700 p-4">
      <div className="mb-6">
        <h2 className="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-4">
          Step Navigator
        </h2>
      </div>

      <nav className="space-y-2">
        {STEPS.map((step) => {
          const isActive = step.number === currentStep;
          const isPast = step.number < currentStep;
          const isFuture = step.number > currentStep;

          return (
            <button
              key={step.number}
              onClick={() => onStepClick?.(step.number)}
              disabled={isFuture && !onStepClick}
              className={`
                w-full text-left p-3 rounded-lg transition-all
                ${isActive
                  ? 'bg-method-vi-primary text-white shadow-lg'
                  : isPast
                  ? 'bg-gray-800 text-gray-300 hover:bg-gray-700'
                  : 'bg-gray-800/50 text-gray-500 cursor-not-allowed'
                }
              `}
            >
              <div className="flex items-center space-x-3">
                <div className={`
                  w-8 h-8 rounded-full flex items-center justify-center font-semibold
                  ${isActive
                    ? 'bg-white text-method-vi-primary'
                    : isPast
                    ? 'bg-method-vi-success text-white'
                    : 'bg-gray-700 text-gray-500'
                  }
                `}>
                  {step.number}
                </div>
                <div className="flex-1">
                  <div className="font-medium text-sm">{step.name}</div>
                  <div className="text-xs opacity-75">{step.role}</div>
                </div>
                {step.isGateStep && (
                  <div className="text-yellow-500" title="Gate Step">
                    🚦
                  </div>
                )}
              </div>
            </button>
          );
        })}
      </nav>
    </aside>
  );
}
