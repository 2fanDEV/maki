export const boards = [
  {
    id: 'documents',
    name: 'Documents',
    href: '/dashboard/documents',
    icon: 'file-text',
  },
  {
    id: 'models',
    name: 'Models',
    href: '/dashboard/models',
    icon: 'boxes',
  },
  {
    id: 'evaluations',
    name: 'Evaluations',
    href: '/dashboard/evaluations',
    icon: 'bar-chart',
  }
] as const;

export type BoardId = (typeof boards)[number]['id'];
