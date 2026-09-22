import {
  useEffect,
  useState,
  type CSSProperties,
  type KeyboardEvent,
  type PointerEvent,
} from 'react';
import { BarChart3, Boxes, FileText } from 'lucide-react';
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarRail,
  SidebarTrigger,
  useSidebar,
} from '@/components/ui/sidebar';
import { TooltipProvider } from '@/components/ui/tooltip';
import { boards, type BoardId } from '@/lib/boards';

type Props = {
  activeBoard: BoardId;
};

const boardIcons = {
  'file-text': FileText,
  boxes: Boxes,
  'bar-chart': BarChart3,
};

const DEFAULT_WIDTH = 15;
const MINIMUM_WIDTH = 12;
const MAXIMUM_WIDTH = 32;
const WIDTH_KEY = 'maki-sidebar-width';

function storedWidth() {
  try {
    const width = Number(sessionStorage.getItem(WIDTH_KEY));
    if (width >= MINIMUM_WIDTH && width <= MAXIMUM_WIDTH) return width;
  } catch {
    // The default remains available when session storage is unavailable.
  }

  return DEFAULT_WIDTH;
}

function storedSidebarOpen() {
  const entry = document.cookie
    .split('; ')
    .find((part) => part.startsWith('sidebar_state='));
  return entry?.split('=')[1] !== 'false';
}

export function DashboardSidebar({ activeBoard }: Props) {
  const [width, setWidth] = useState(DEFAULT_WIDTH);
  const [open, setOpen] = useState(true);

  useEffect(() => {
    setWidth(storedWidth());
    setOpen(storedSidebarOpen());
  }, []);

  function changeWidth(next: number) {
    const width = Math.min(MAXIMUM_WIDTH, Math.max(MINIMUM_WIDTH, next));
    setWidth(width);

    try {
      sessionStorage.setItem(WIDTH_KEY, String(width));
    } catch {
      // Resizing still works for this page when storage is unavailable.
    }
  }

  return (
    <TooltipProvider>
      <SidebarProvider
        className="w-auto shrink-0"
        open={open}
        onOpenChange={setOpen}
        style={{ '--sidebar-width': `${width}vw`, '--sidebar-width-icon': '3vw' } as CSSProperties}
      >
        <DashboardSidebarContent
          activeBoard={activeBoard}
          width={width}
          onWidthChange={changeWidth}
        />
      </SidebarProvider>
    </TooltipProvider>
  );
}

function DashboardSidebarContent({
  activeBoard,
  width,
  onWidthChange,
}: Props & { width: number; onWidthChange: (width: number) => void }) {
  const { open } = useSidebar();
  const [dragging, setDragging] = useState(false);

  function resize(next: number) {
    onWidthChange(Math.min(MAXIMUM_WIDTH, Math.max(MINIMUM_WIDTH, next)));
  }

  function drag(event: PointerEvent<HTMLButtonElement>) {
    if (!event.currentTarget.hasPointerCapture(event.pointerId)) return;
    resize((event.clientX / window.innerWidth) * 100);
  }

  function resizeWithKeyboard(event: KeyboardEvent<HTMLButtonElement>) {
    const next = {
      ArrowLeft: width - 1,
      ArrowRight: width + 1,
      Home: MINIMUM_WIDTH,
      End: MAXIMUM_WIDTH,
    }[event.key];

    if (next === undefined) return;
    event.preventDefault();
    resize(next);
  }

  return (
    <Sidebar
      collapsible="icon"
      style={{ '--sidebar-width': `${width}vw` } as CSSProperties}
      data-resizing={dragging}
    >
      <SidebarHeader className="h-14 flex-row items-center justify-between px-4 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-0">
        <span className="truncate text-sm font-semibold tracking-tight group-data-[collapsible=icon]:hidden">
          Maki
        </span>
        <SidebarTrigger
          className="shrink-0 max-w-full [&_svg]:max-w-full"
          aria-label={open ? 'Collapse sidebar' : 'Expand sidebar'}
          aria-expanded={open}
        />
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup className="group-data-[collapsible=icon]:px-0">
          <nav id="board-navigation" aria-label="Boards">
            <SidebarMenu>
              {boards.map((board) => {
                const Icon = boardIcons[board.icon];
                const active = activeBoard === board.id;

                return (
                  <SidebarMenuItem
                    key={board.id}
                    className="group-data-[collapsible=icon]:flex group-data-[collapsible=icon]:justify-center"
                  >
                    <SidebarMenuButton
                      render={<a href={board.href} />}
                      className="group-data-[collapsible=icon]:w-full! group-data-[collapsible=icon]:p-0! group-data-[collapsible=icon]:justify-center [&_svg]:max-w-full"
                      isActive={active}
                      aria-label={board.name}
                      aria-current={active ? 'page' : undefined}
                      tooltip={board.name}
                    >
                      <Icon aria-hidden="true" />
                      <span className="group-data-[collapsible=icon]:hidden">{board.name}</span>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                );
              })}
            </SidebarMenu>
          </nav>
        </SidebarGroup>
      </SidebarContent>

      {open && (
        <SidebarRail
          className="sidebar-resize"
          data-resizing={dragging}
          role="separator"
          tabIndex={0}
          aria-label="Resize sidebar"
          aria-orientation="vertical"
          aria-controls="board-navigation"
          aria-valuemin={MINIMUM_WIDTH}
          aria-valuemax={MAXIMUM_WIDTH}
          aria-valuenow={Math.round(width)}
          aria-valuetext={`${Math.round(width)}% of page width`}
          title="Drag to resize; use arrow keys when focused"
          onClick={(event) => event.preventDefault()}
          onPointerDown={(event) => {
            if (event.button !== 0) return;
            event.preventDefault();
            event.currentTarget.focus();
            event.currentTarget.setPointerCapture(event.pointerId);
            setDragging(true);
          }}
          onPointerMove={drag}
          onPointerUp={(event) => {
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
  );
}
