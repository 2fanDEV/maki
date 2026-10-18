## Development

Use pnpm for frontend dependencies and scripts. Run `pnpm install --frozen-lockfile` before checks or builds.

The frontend is a Vite and React application. `src/App.tsx` owns the dashboard shell and board selection. `src/lib/navigation.ts` handles navigation between the three board URLs.

Run `pnpm check` and `pnpm build` after frontend changes.
