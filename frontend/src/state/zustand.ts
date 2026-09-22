import { Sidebar } from "@/components/ui/sidebar";
import { create } from "zustand";

enum Theme {
  "LIGHT" = "light",
  "DARK" = "dark",
  "SYSTEM" = "system"
}

export type ThemeType = {
  theme: Theme | null
  key: string
}

export const useThemeStore = create<ThemeType>((set) => ({
  theme: null,
  key: "maki-theme",
  initTheme: () => set((state) => {
    const storedTheme = localStorage.getItem(state.key);
    const theme = storedTheme === Theme.DARK || storedTheme === Theme.LIGHT ? storedTheme : Theme.SYSTEM;
    return { theme }
  }),
  toggleTheme: () => set((state) => {
    const theme = state.theme === Theme.DARK ? Theme.LIGHT : Theme.DARK;
    localStorage.setItem(state.key, theme);
    return { theme }
  }),
}));

export enum SidebarState {
  "OPENED" = "opened",
  "CLOSED" = "closed"
};
export type WebpageState = {
  sidebar: SidebarState | null
  key: string,
  initSidebarState: () => void,
  toggleSidebar: () => void
}

export const useWebpageStore = create<WebpageState>((set) => ({
  sidebar: null,
  key: "sidebar-state",
  initSidebarState: () => set((state) => {
    let storedSidebarState = localStorage.getItem(state.key);
    const sidebar = storedSidebarState === SidebarState.OPENED || storedSidebarState === SidebarState.CLOSED ? storedSidebarState : SidebarState.OPENED;
    localStorage.setItem('sidebar-state', sidebar)
    return { sidebar };
  }),
  toggleSidebar: () => set((state) => {
    const sidebar = state.sidebar === SidebarState.OPENED ? SidebarState.CLOSED : SidebarState.OPENED;
    localStorage.setItem(state.key, sidebar)
    return { sidebar }
  })
}));