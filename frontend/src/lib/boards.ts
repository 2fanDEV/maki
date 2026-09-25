type SidebarBoard = {
  type: 'board';
  id: string;
  name: string;
  href: string;
  icon: 'file-text' | 'boxes' | 'bar-chart';
};

type SidebarSeparator = {
  type: 'separator';
  id: string;
  label?: string;
};

export type SidebarElement = SidebarBoard | SidebarSeparator;

export const sidebarElements = [
  {
    type: 'board',
    id: 'documents',
    name: 'Documents',
    href: '/dashboard/documents',
    icon: 'file-text',
  },
  {
    type: 'board',
    id: 'models',
    name: 'Models',
    href: '/dashboard/models',
    icon: 'boxes',
  },
  {
    type: 'separator',
    id: 'evaluation-separator',
    label: 'Evaluations',
  },
  {
    type: 'board',
    id: 'evaluations',
    name: 'Evaluations',
    href: '/dashboard/evaluations',
    icon: 'bar-chart',
  }
] as const satisfies readonly SidebarElement[];

type Board = Extract<(typeof sidebarElements)[number], { type: 'board' }>;

export const boards = sidebarElements.filter(
  (element): element is Board => element.type === 'board',
);

export type BoardId = Board['id'];
