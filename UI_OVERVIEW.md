# Method-VI React UI Overview

## ✅ Completed Features

### 1. Core Layout
- **MainLayout** - Full-height layout with header, sidebar, content area, and footer
- **Header** - Shows app branding, run ID, current step, and navigation to settings
- **Sidebar** - Step navigator with 7 steps (0-6.5 + Closure), showing current step, completed steps, and role
- **MetricsBar** (Footer) - Displays the Critical 6 Metrics (CI, EV, IAS, EFI, SEC, PCI) with color coding

### 2. Components
- **StepNavigator** - Visual step progression with step numbers, names, roles, and gate indicators (🚦)
- **MetricsBar** - Real-time metrics display with color-coded thresholds (green/yellow/red)
- **GateDialog** - Modal for gate approval/rejection with approver name and rejection reason
- **ChatInterface** - Message interface for user-AI interaction with auto-scroll

### 3. Pages
- **Home** (`/`) - Landing page with "New Run" form and quick links to Sessions/Settings
- **RunView** (`/run/:runId`) - Active run interface with chat, step info, and gate handling
- **Settings** (`/settings`) - Configuration page for API key, model selection, and preferences
- **Sessions** (`/sessions`) - List of past runs with status badges and resume/view actions

### 4. Routing
- React Router DOM configured with 4 main routes
- Browser-based routing (no hash routing)

### 5. Styling
- Tailwind CSS fully configured with custom Method-VI color palette
- Dark theme with gray-900/gray-800 backgrounds
- Responsive design (though desktop-focused for Tauri)
- Custom color scheme:
  - Primary: Blue (#2563eb)
  - Success: Green (#10b981)
  - Warning: Yellow (#f59e0b)
  - Danger: Red (#ef4444)

## 📁 File Structure

```
src/
├── components/
│   ├── layout/
│   │   ├── MainLayout.tsx      # Main app layout wrapper
│   │   ├── Header.tsx          # Top navigation bar
│   │   └── Sidebar.tsx         # Left sidebar with step navigator
│   ├── ChatInterface.tsx        # Message UI component
│   ├── GateDialog.tsx          # Gate approval modal
│   └── MetricsBar.tsx          # Footer metrics display
├── pages/
│   ├── Home.tsx                # Landing page
│   ├── RunView.tsx             # Active run view
│   ├── Settings.tsx            # Settings page
│   └── Sessions.tsx            # Past sessions list
├── types/
│   └── index.ts                # TypeScript type definitions
├── App.tsx                     # Router configuration
├── main.tsx                    # React app entry point
└── index.css                   # Tailwind imports + global styles
```

## 🚀 Running the UI

```bash
# Development mode (hot reload)
cd method-vi
npm run dev

# Build for production
npm run build

# Run with Tauri
npm run tauri dev
```

## 🎨 Design Patterns

### Color Coding
- **Steps**: Active (blue), Completed (green), Future (gray)
- **Metrics**: Good (green ≥ threshold), Warning (yellow), Danger (red)
- **Status**: Active (blue), Completed (green), Halted (red)

### Layout Philosophy
- Header: Global navigation and context
- Sidebar: Step-specific navigation (hidden on non-run pages)
- Main: Primary content area (chat, forms, lists)
- Footer: Always-visible metrics bar

## 🔌 Next Steps (Backend Integration)

The UI is currently using mock data and console.log statements. To connect to the Rust backend:

1. **Create Tauri Commands** - Add `#[tauri::command]` functions in Rust
2. **Invoke from React** - Use `invoke('command_name', { args })` from `@tauri-apps/api/core`
3. **State Management** - Consider adding Zustand or React Query for state
4. **Real-time Updates** - Use Tauri events for backend → frontend communication

### Example Integration Points

```typescript
import { invoke } from '@tauri-apps/api/core';

// Create new run
const runId = await invoke('create_new_run', { label: 'My-Project' });

// Execute Step 0
const intentSummary = await invoke('execute_step_0', {
  runId,
  userIntent: 'Build a mobile app'
});

// Approve gate
await invoke('approve_gate', { runId, approver: 'John Doe' });

// Get metrics
const metrics = await invoke('get_metrics', { runId });
```

## 📊 Current Status

✅ **Complete**:
- All UI components built
- Routing configured
- Tailwind CSS setup
- Mock data for demonstration

⏳ **Pending**:
- Tauri command integration
- Real backend data
- Error handling UI
- Loading states
- Artifact viewing
- Pattern visualization

## 🧪 Testing the UI

The dev server is running at: **http://localhost:1420**

You can:
1. Navigate to the home page
2. Create a new run (simulated)
3. See the chat interface
4. Trigger the gate dialog (appears after first message in Step 0)
5. Navigate to Settings/Sessions pages
6. View the step navigator and metrics bar

All features are functional but using mock/simulated data until connected to the Rust backend.
