import { create } from 'zustand';
import { createStore } from 'zustand/vanilla';

export enum Theme {
  LIGHT = 'light',
  DARK = 'dark',
  SYSTEM = 'system',
}

function readPreference(key: string) {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

function savePreference(key: string, value: string) {
  try {
    localStorage.setItem(key, value);
  } catch {
    // Preferences still work for this session when storage is unavailable.
  }
}

export type ThemeType = {
  theme: Theme;
  resolvedTheme: Theme.LIGHT | Theme.DARK;
  key: string;
  initTheme: () => void;
  toggleTheme: () => void;
};

export const useThemeStore = create<ThemeType>((set) => ({
  theme: Theme.SYSTEM,
  resolvedTheme: Theme.LIGHT,
  key: 'maki-theme',
  initTheme: () => set((state) => {
    const storedTheme = readPreference(state.key);
    const theme = storedTheme === Theme.DARK || storedTheme === Theme.LIGHT
      ? storedTheme
      : Theme.SYSTEM;
    const systemTheme = window.matchMedia('(prefers-color-scheme: dark)').matches
      ? Theme.DARK
      : Theme.LIGHT;
    return { theme, resolvedTheme: theme === Theme.SYSTEM ? systemTheme : theme };
  }),
  toggleTheme: () => set((state) => {
    const theme = state.resolvedTheme === Theme.DARK ? Theme.LIGHT : Theme.DARK;
    savePreference(state.key, theme);
    return { theme, resolvedTheme: theme };
  }),
}));

export enum SidebarState {
  OPENED = 'opened',
  CLOSED = 'closed',
}

export const DEFAULT_WIDTH = 15;
export const MINIMUM_WIDTH = 12;
export const MAXIMUM_WIDTH = 32;
export const SIDEBAR_COOKIE = 'sidebar-state';
export const WIDTH_COOKIE = 'maki-sidebar-width';

function saveSidebarPreference(key: string, value: string) {
  try {
    const secure = window.location.protocol === 'https:' ? '; Secure' : '';
    document.cookie = `${key}=${encodeURIComponent(value)}; Path=/; Max-Age=31536000; SameSite=Lax${secure}`;
  } catch {
    // The sidebar still works for this page when cookies are unavailable.
  }
}

export type SidebarPreferences = {
  sidebar: SidebarState;
  width: number;
};

export type WebpageState = SidebarPreferences & {
  toggleSidebar: () => void;
  setWidth: (width: number) => void;
};

export const createWebpageStore = (initialState: SidebarPreferences) => createStore<WebpageState>((set) => ({
  ...initialState,
  toggleSidebar: () => set((state) => {
    const sidebar = state.sidebar === SidebarState.OPENED ? SidebarState.CLOSED : SidebarState.OPENED;
    saveSidebarPreference(SIDEBAR_COOKIE, sidebar);
    return { sidebar };
  }),
  setWidth: (next) => set(() => {
    const width = Number.isFinite(next)
      ? Math.min(MAXIMUM_WIDTH, Math.max(MINIMUM_WIDTH, next))
      : DEFAULT_WIDTH;
    saveSidebarPreference(WIDTH_COOKIE, String(width));
    return { width };
  }),
}));
