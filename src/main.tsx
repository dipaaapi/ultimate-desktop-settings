import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';
import { ToastProvider } from './components/Toasts';
import { PreferencesProvider } from './lib/theme';
import './styles.css';

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <PreferencesProvider>
      <ToastProvider>
        <App />
      </ToastProvider>
    </PreferencesProvider>
  </StrictMode>,
);
