import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import { Theme, useThemeStore } from './state/zustand';
import './styles/global.css';
import './styles/dashboard.css';

if (window.location.pathname === '/' || window.location.pathname === '/dashboard') {
  window.history.replaceState(null, '', '/dashboard/documents');
}

useThemeStore.getState().initTheme();
document.documentElement.classList.toggle(
  'dark',
  useThemeStore.getState().resolvedTheme === Theme.DARK,
);
useThemeStore.subscribe((state) => {
  document.documentElement.classList.toggle('dark', state.resolvedTheme === Theme.DARK);
});
window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', ({ matches }) => {
  if (useThemeStore.getState().theme === Theme.SYSTEM) {
    useThemeStore.setState({ resolvedTheme: matches ? Theme.DARK : Theme.LIGHT });
  }
});

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
