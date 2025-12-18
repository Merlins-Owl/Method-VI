import { BrowserRouter, Routes, Route } from 'react-router-dom';
import Home from './pages/Home';
import RunView from './pages/RunView';
import Settings from './pages/Settings';
import Sessions from './pages/Sessions';
import MetricsTestPage from './pages/MetricsTestPage';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/run/:runId" element={<RunView />} />
        <Route path="/settings" element={<Settings />} />
        <Route path="/sessions" element={<Sessions />} />
        <Route path="/metrics-test" element={<MetricsTestPage />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
