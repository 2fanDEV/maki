import { useSyncExternalStore } from 'react';

const routeChangeEvent = 'maki:route-change';

function subscribe(onChange: () => void) {
  window.addEventListener('popstate', onChange);
  window.addEventListener(routeChangeEvent, onChange);
  return () => {
    window.removeEventListener('popstate', onChange);
    window.removeEventListener(routeChangeEvent, onChange);
  };
}

export function usePathname() {
  return useSyncExternalStore(subscribe, () => window.location.pathname);
}

export function navigate(path: string) {
  if (window.location.pathname === path) return;
  window.history.pushState(null, '', path);
  window.dispatchEvent(new Event(routeChangeEvent));
}
