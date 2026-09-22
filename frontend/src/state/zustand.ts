import { create } from 'zustand';

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

export type WebpageState = {
  sidebar: SidebarState;
  width: number;
  sidebarKey: string;
  widthKey: string;
  initState: () => void;
  toggleSidebar: () => void;
  setWidth: (width: number) => void;
};

export const useWebpageStore = create<WebpageState>((set) => ({
  sidebar: SidebarState.OPENED,
  width: DEFAULT_WIDTH,
  sidebarKey: 'sidebar-state',
  widthKey: 'maki-sidebar-width',
  initState: () => set((state) => {
    const storedSidebar = readPreference(state.sidebarKey);
    const sidebar = storedSidebar === SidebarState.OPENED || storedSidebar === SidebarState.CLOSED
      ? storedSidebar
      : SidebarState.OPENED;
    const storedWidth = Number(readPreference(state.widthKey));
    const width = Number.isFinite(storedWidth) && storedWidth >= MINIMUM_WIDTH && storedWidth <= MAXIMUM_WIDTH
      ? storedWidth
      : DEFAULT_WIDTH;
    return { sidebar, width };
  }),
  toggleSidebar: () => set((state) => {
    const sidebar = state.sidebar === SidebarState.OPENED ? SidebarState.CLOSED : SidebarState.OPENED;
    savePreference(state.sidebarKey, sidebar);
    return { sidebar };
  }),
  setWidth: (next) => set((state) => {
    const width = Number.isFinite(next)
      ? Math.min(MAXIMUM_WIDTH, Math.max(MINIMUM_WIDTH, next))
      : DEFAULT_WIDTH;
    savePreference(state.widthKey, String(width));
    return { width };
  }),
}));
