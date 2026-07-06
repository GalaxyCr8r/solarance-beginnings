# tmp-docs — exploratory simplified website ("relay station")

An **exploratory** replacement candidate for `docs/`: a much smaller
TypeScript + React + Vite site that talks to the live SpacetimeDB module.
It is *not* wired into any deployment; the real site is still `docs/`.

## What it does

- **Splash / Manifesto / FAQ** — static pages explaining the game.
- **Auth0 login** — same tenant as the game client, so a web login resolves
  to the *same SpacetimeDB identity* as in-game.
- **Pilot registration** — claim a username (calls `register_playername`,
  with the same faction rules the game enforces).
- **Galaxy comms** — live global chat (`my_galaxy_chat` view +
  `send_galaxy_chat` reducer) merged with official server bulletins.
- **Galaxy map & system maps** — live `star_system`, `sector`, and
  `jump_gate` tables rendered as SVG charts.
- **"Download the client" dialog** after every login, plus an itch.io link
  and a greyed-out Steam link on every page:
  <https://galaxycr8r.itch.io/solarance-beginnings>

## Run it

```bash
cd tmp-docs
npm install
npm run dev        # http://localhost:5173
npm run build      # type-check + production build into dist/
```

## Configuration

Everything has working defaults (maincloud + the dev Auth0 tenant from
`client/.env.template`). Override with Vite env vars, e.g. a `.env.local`:

```bash
VITE_SPACETIME_HOST=http://localhost:3000   # local `spacetime start`
VITE_SPACETIME_DB=solarance-beginnings
VITE_AUTH0_DOMAIN=dev-k6zdm2f3z3kst6r7.us.auth0.com
VITE_AUTH0_CLIENT_ID=BnJiVrOXavZ1mbvsiwvBcZ96dTFH9k4L
```

### Auth0 setup (one-time, in the Auth0 dashboard)

The game client's Auth0 application is a *Native* app with a
`http://localhost:13613` callback. Browser logins need a **Single Page
Application** entry (either a new application in the same tenant, or the
existing app if you switch its type):

1. Applications → *your app* → Settings
2. **Allowed Callback URLs**: add the site origin(s), e.g.
   `http://localhost:5173, https://solarance-beginnings.com`
3. **Allowed Logout URLs**: same values
4. **Allowed Web Origins**: same values
5. If you created a new SPA application, put its client id in
   `VITE_AUTH0_CLIENT_ID` (or `src/config.ts`).

Because SpacetimeDB derives identity from the token's *issuer + subject*,
any application in the same tenant yields the same identity for the same
user — website and game client stay one account.

## About `src/module_bindings/`

⚠ **Hand-written, not CLI-generated.** This container couldn't reach the
SpacetimeDB installer or GitHub releases, and crates.io only carries the CLI
up to 1.3 (the module is on 2.6), so the bindings for the handful of tables
the site needs were written by hand following the exact structure the 2.x
CLI emits. Field names/order mirror `server/src/tables/*.rs` — BSATN is
positional, so **if a server struct changes, this file must change too**.

To replace them with generated bindings once you have a 2.6-compatible CLI:

```bash
spacetime generate --lang typescript \
  --out-dir tmp-docs/src/module_bindings --project-path server
```

The app imports only `DbConnection`, `tables`, `reducers`, and row types, so
generated output should drop in with few or no app-side edits.

### Things to verify against a live module (untestable from this container)

- The `my_galaxy_chat` **view** is declared client-side as a table with
  `name: 'my_galaxy_chat'`. Confirm the 2.6 codegen does the same and that
  subscription/eviction behaves for views.
- `Player.faction_id` is declared as the one-field `FactionId` wrapper
  product (`{ value: u32 }`), matching the DSL macro's emitted struct.

## Deploying

`npm run build` produces a fully static `dist/` (relative asset paths), so
it can be served from GitHub Pages, a subfolder, or any static host. If it
ever replaces `docs/`, either commit the built output or add a Pages build
action — decision deferred until this stops being exploratory.
