# Maki frontend

This React frontend has Documents, Models, and Evaluations boards. The board URLs are `/dashboard/documents`, `/dashboard/models`, and `/dashboard/evaluations`. The root URL and `/dashboard` open Documents. No board API calls are included yet.

## Install and run

Use the Node and pnpm versions in `mise/config.toml`. From this directory:

```sh
pnpm install --frozen-lockfile
pnpm dev
```

Open http://localhost:4321.

## Generate the API specification and types

Start the Rust server from the repository root with `cargo run`, configured by `.env.toml`. With the server running, execute:

```sh
pnpm api:generate
```

The command downloads `http://127.0.0.1:3000/openapi.json` and generates `openapi.json` and `src/api/schema.d.ts`. To select another endpoint, set `OPENAPI_URL` before the command.

Both generated files belong in version control. Refresh them after API changes. Generation is explicit; checks and builds use the saved files.

## UI foundation

Vite builds the React app. Tailwind CSS v4 and shadcn/ui use `src/styles/global.css`. The `@/` import alias resolves to `src/`. The dashboard shell and route selection are in `src/App.tsx`. Browser navigation is in `src/lib/navigation.ts`; the Vite server and preview serve `index.html` for direct board URLs. A production host must also serve `index.html` for these URLs.

The sidebar starts at 15vw on desktop. Its size and collapsed state are saved in cookies. The theme is saved in local storage. Navigation keeps the sidebar and top bar mounted while the board changes.

Add UI components with `pnpm exec shadcn add COMPONENT` from this directory.

## Check and build

```sh
pnpm check
pnpm build
pnpm preview
```

The production files are in `build/`. Serve that directory with a static host that falls back to `index.html` for board URLs.
