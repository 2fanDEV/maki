import { useEffect, useState, type CSSProperties, type KeyboardEvent, type PointerEvent } from 'react';
import { BarChart3, Boxes, FileText, Moon, Sun, WandSparkles } from 'lucide-react';
import {
  Sidebar, SidebarContent, SidebarGroup, SidebarHeader, SidebarInset,
  SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarProvider,
  SidebarRail, SidebarTrigger, useSidebar,
} from '@/components/ui/sidebar';
import { Switch } from '@/components/ui/switch';
import { TooltipProvider } from '@/components/ui/tooltip';
import { useIsMobile } from '@/hooks/use-mobile';
import '@/styles/dashboard.css';

const boards = [
  { id: 'documents', name: 'Documents', icon: FileText },
  { id: 'models', name: 'Models', icon: Boxes },
  { id: 'evaluations', name: 'Evaluations', icon: BarChart3 },
] as const;

export function Dashboard() {
  const isMobile = useIsMobile();
  const [widths, setWidths] = useState({ desktop: 15, mobile: 60 });
  const sizeKey = isMobile ? 'mobile' : 'desktop';
  const width = widths[sizeKey];

  return (
    <TooltipProvider>
      <SidebarProvider
        style={{ '--sidebar-width': `${width}vw`, '--sidebar-width-icon': '3vw' } as CSSProperties}
      >
        <DashboardContent
          width={width}
          onWidthChange={next => setWidths(current => ({ ...current, [sizeKey]: next }))}
        />
      </SidebarProvider>
    </TooltipProvider>
  );
}

function DashboardContent({ width, onWidthChange }: { width: number; onWidthChange: (width: number) => void }) {
  const { open, isMobile, openMobile, setOpenMobile } = useSidebar();
  const [activeBoard, setActiveBoard] = useState<string>('documents');
  const [dark, setDark] = useState(true);
  const [dragging, setDragging] = useState(false);
  const minimum = isMobile ? 40 : 12;
  const maximum = isMobile ? 80 : 32;
  const triggerLabel = isMobile ? 'Close sidebar' : open ? 'Collapse sidebar' : 'Expand sidebar';

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

  function resize(next: number) {
    onWidthChange(Math.min(maximum, Math.max(minimum, next)));
  }

  function drag(event: PointerEvent<HTMLButtonElement>) {
    if (!event.currentTarget.hasPointerCapture(event.pointerId)) return;
    resize((event.clientX / window.innerWidth) * 100);
  }

  function resizeWithKeyboard(event: KeyboardEvent<HTMLButtonElement>) {
    const next = { ArrowLeft: width - 1, ArrowRight: width + 1, Home: minimum, End: maximum }[event.key];
    if (next === undefined) return;
    event.preventDefault();
    resize(next);
  }

  return (
    <>
      <Sidebar
        collapsible="icon"
        style={{ '--sidebar-width': `${width}vw` } as CSSProperties}
        data-resizing={dragging}
      >
        <SidebarHeader className="h-14 flex-row items-center justify-between px-4 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-0">
          <span className="truncate text-sm font-semibold tracking-tight group-data-[collapsible=icon]:hidden">Maki</span>
          <SidebarTrigger className="shrink-0 max-w-full [&_svg]:max-w-full" aria-label={triggerLabel} aria-expanded={isMobile ? openMobile : open} />
        </SidebarHeader>
        <SidebarContent>
          <SidebarGroup className="group-data-[collapsible=icon]:px-0">
            <nav id="board-navigation" aria-label="Boards">
              <SidebarMenu>
                {boards.map(({ id, name, icon: Icon }) => (
                  <SidebarMenuItem key={id} className="group-data-[collapsible=icon]:flex group-data-[collapsible=icon]:justify-center">
                    <SidebarMenuButton
                      className="group-data-[collapsible=icon]:w-full! group-data-[collapsible=icon]:p-0! group-data-[collapsible=icon]:justify-center [&_svg]:max-w-full"
                      isActive={activeBoard === id}
                      aria-label={name}
                      aria-current={activeBoard === id ? 'page' : undefined}
                      aria-controls={`board-${id}`}
                      tooltip={name}
                      onClick={() => {
                        setActiveBoard(id);
                        if (isMobile) setOpenMobile(false);
                      }}
                    >
                      <Icon aria-hidden="true" />
                      <span className="group-data-[collapsible=icon]:hidden">{name}</span>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                ))}
              </SidebarMenu>
            </nav>
          </SidebarGroup>
        </SidebarContent>
        {(isMobile || open) && (
          <SidebarRail
            className="sidebar-resize"
            data-resizing={dragging}
            role="separator"
            tabIndex={0}
            aria-label="Resize sidebar"
            aria-orientation="vertical"
            aria-controls="board-navigation"
            aria-valuemin={minimum}
            aria-valuemax={maximum}
            aria-valuenow={Math.round(width)}
            aria-valuetext={`${Math.round(width)}% of page width`}
            title="Drag to resize; use arrow keys when focused"
            onClick={event => event.preventDefault()}
            onPointerDown={event => {
              if (event.button !== 0) return;
              event.preventDefault();
              event.currentTarget.focus();
              event.currentTarget.setPointerCapture(event.pointerId);
              setDragging(true);
            }}
            onPointerMove={drag}
            onPointerUp={event => {
              if (event.currentTarget.hasPointerCapture(event.pointerId)) {
                event.currentTarget.releasePointerCapture(event.pointerId);
              }
              setDragging(false);
            }}
            onPointerCancel={() => setDragging(false)}
            onLostPointerCapture={() => setDragging(false)}
            onKeyDown={resizeWithKeyboard}
          />
        )}
      </Sidebar>
      <SidebarInset className="min-w-0">
        <header className="dashboard-topbar">
          {isMobile && <SidebarTrigger aria-label="Open sidebar" aria-expanded={openMobile} />}
          <div className="ml-auto flex items-center gap-2" aria-label="Color theme">
            <Sun className="size-4 text-muted-foreground" aria-hidden="true" />
            <Switch checked={dark} onCheckedChange={changeTheme} aria-label="Dark mode" />
            <Moon className="size-4 text-muted-foreground" aria-hidden="true" />
          </div>
        </header>
        <div className="dashboard-main">
          {boards.map(({ id, name }) => (
            <section key={id} id={`board-${id}`} className="board" hidden={activeBoard !== id} aria-labelledby={`board-title-${id}`}>
              <h1 id={`board-title-${id}`}>{name}</h1>
              <div className="board-canvas" />
            </section>
          ))}
        </div>
      </SidebarInset>
    </>
  );
}
