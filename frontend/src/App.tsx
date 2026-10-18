import { useEffect, useState } from 'react';
import { Documents } from '@/components/boards/documents/Documents';
import { Evaluations } from '@/components/boards/evaluations/Evaluations';
import { Models } from '@/components/boards/models/Models';
import { DashboardSidebar } from '@/components/dashboard/DashboardSidebar';
import { DashboardTopbar } from '@/components/dashboard/DashboardTopbar';
import { boards, type BoardId } from '@/lib/boards';
import { usePathname } from '@/lib/navigation';
import {
  DEFAULT_WIDTH,
  MAXIMUM_WIDTH,
  MINIMUM_WIDTH,
  SIDEBAR_COOKIE,
  WIDTH_COOKIE,
  SidebarState,
  type SidebarPreferences,
} from '@/state/zustand';

function readCookie(name: string) {
  const value = document.cookie.split('; ').find((cookie) => cookie.startsWith(`${name}=`));
  return value ? decodeURIComponent(value.slice(name.length + 1)) : undefined;
}

function readSidebarPreferences(): SidebarPreferences {
  const width = Number(readCookie(WIDTH_COOKIE));
  return {
    sidebar: readCookie(SIDEBAR_COOKIE) === SidebarState.CLOSED
      ? SidebarState.CLOSED
      : SidebarState.OPENED,
    width: Number.isFinite(width) && width >= MINIMUM_WIDTH && width <= MAXIMUM_WIDTH
      ? width
      : DEFAULT_WIDTH,
  };
}

function Board({ id }: { id: BoardId }) {
  switch (id) {
    case 'documents': return <Documents />;
    case 'models': return <Models />;
    case 'evaluations': return <Evaluations />;
  }
}

export function App() {
  const pathname = usePathname();
  const [initialSidebarState] = useState(readSidebarPreferences);
  const activeBoard = boards.find((board) => board.href === pathname);

  useEffect(() => {
    document.title = activeBoard ? `${activeBoard.name} | Maki` : 'Maki';
  }, [activeBoard]);

  if (!activeBoard) {
    return <main><h1>Page not found</h1><a href="/dashboard/documents">Documents</a></main>;
  }

  return (
    <div className="dashboard-layout">
      <DashboardSidebar activeBoard={activeBoard.id} initialState={initialSidebarState} />
      <div className="dashboard-content">
        <DashboardTopbar />
        <main className="dashboard-main">
          <section className="board" aria-labelledby={`board-title-${activeBoard.id}`}>
            <Board id={activeBoard.id} />
          </section>
        </main>
      </div>
    </div>
  );
}
