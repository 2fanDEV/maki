import { Moon, Sun } from 'lucide-react';
import { Switch } from '@/components/ui/switch';
import { Theme, useThemeStore } from '@/state/zustand';

export function DashboardTopbar() {
  const theme = useThemeStore(state => state.resolvedTheme);
  const toggleTheme = useThemeStore(state => state.toggleTheme);

  return (
    <header className="dashboard-topbar">
      <div className="ml-auto flex items-center gap-2" aria-label="Color theme">
        <Sun className="size-4 text-muted-foreground" aria-hidden="true" />
        <Switch checked={theme === Theme.DARK} onCheckedChange={() => toggleTheme()} aria-label="Dark mode" />
        <Moon className="size-4 text-muted-foreground" aria-hidden="true" />
      </div>
    </header>
  );
}
