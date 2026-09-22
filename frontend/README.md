# Maki frontend

The Astro frontend renders a dashboard with Documents, Models, and Evaluations
boards. Each board has a direct `/dashboard/...` route and is composed into the
Astro-owned dashboard shell. No API calls or board-specific data features are
included yet.

The shadcn sidebar starts at 15vw on desktop. Drag its rail to resize, or focus
the separator and use the left/right arrow keys (Home/End select the
minimum/maximum). Its trigger collapses it to icons; expanding restores its
previous width. The desktop width is maintained for the current session.
Cmd/Ctrl+B also toggles the sidebar. The dashboard uses a desktop-only layout.

Dark mode is the default. The switch between the sun/moon icons at the top right
controls dark mode and saves the choice locally for the next visit.

## Install and run

Use the Node version configured by the repository’s Mise setup (Node 22.12 or
newer is required). Run these commands from the `frontend` directory:

```sh
npm ci
npm run dev -- --background
```

Open http://localhost:4321. Manage the background server with
`npm run astro -- dev status`, `npm run astro -- dev logs`, and
`npm run astro -- dev stop`.

## Generate the API specification and types

Start the Rust server from the repository root with `cargo run`, configured by
`.env.toml`. See the root README for database setup. OpenAPI generation uses the
server’s route registrations and does not issue application database queries.

With the server running, execute from `frontend`:

```sh
npm run api:generate
```

The command downloads `http://127.0.0.1:3000/openapi.json` and generates:

- `openapi.json`: the server’s OpenAPI specification.
- `src/api/schema.d.ts`: TypeScript types generated with `openapi-typescript`.

To select another endpoint:

```sh
OPENAPI_URL=http://127.0.0.1:3000/openapi.json npm run api:generate
```

Both generated files belong in version control. Refresh them after API changes;
do not edit them by hand. Fetching has a 10-second timeout. Network, HTTP, JSON,
and type-generation failures exit unsuccessfully without replacing existing
outputs.

Generation is explicit. Development, checking, and builds use the saved files
and do not require the backend. No runtime API client is included.

## UI foundation

React is enabled through `@astrojs/react`. Tailwind CSS v4 and shadcn/ui use the
shared stylesheet at `src/styles/global.css`, imported by the dashboard layout.
The shadcn configuration is in `components.json`: Base UI, Nova style, neutral
colors, Lucide icons, and the locally bundled Geist font.

The `@/` import alias resolves to `src/`. Astro owns the board routes and composes
the dashboard layout. The shadcn sidebar and top bar are separate React islands
with `transition:persist`: navigation updates the active sidebar link without
remounting either island or resetting collapse, width, or theme state. The board
slot is outside both islands, so only the selected board is replaced. Navigation
uses Astro's client router with ordinary links and no page-transition animation;
board URLs remain directly addressable and work with browser history. The sidebar
and top bar hydrate on page load.
Add further components when needed using `npx shadcn add COMPONENT` from this
directory, replacing `COMPONENT` with the component name.

Interactive React components used in Astro need a hydration directive. The
sidebar and top bar use `client:load`, and Documents uses its own `client:load`
island for the action dropdown. Keep board event handlers
inside their board island rather than passing functions through Astro.

## Check and build

```sh
npm run check
npm run build
npm run preview
```

The build writes static output to `dist`. No deployment is configured.
