import { useEffect, useState } from 'react';
import { Moon, Sun } from 'lucide-react';
import { Switch } from '@/components/ui/switch';

export function DashboardTopbar() {
  const [dark, setDark] = useState(true);

  useEffect(() => {
    setDark(document.documentElement.classList.contains('dark'));
  }, []);

  function changeTheme(next: boolean) {
    setDark(next);
    document.documentElement.classList.toggle('dark', next);

    try {
      localStorage.setItem('maki-theme', next ? 'dark' : 'light');
    } catch {
      // The switch still works for this session when storage is unavailable.
    }
  }

  return (
    <header className="dashboard-topbar">
      <div className="ml-auto flex items-center gap-2" aria-label="Color theme">
        <Sun className="size-4 text-muted-foreground" aria-hidden="true" />
        <Switch checked={dark} onCheckedChange={changeTheme} aria-label="Dark mode" />
        <Moon className="size-4 text-muted-foreground" aria-hidden="true" />
      </div>
    </header>
  );
}
