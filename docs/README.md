# Solarance: Beginnings — site

The contents of this folder are a complete static site. No build step.

## Run locally

`fetch()` is blocked on `file://`, so the devlog needs an http server. Any of:

```bash
# python
python -m http.server 5173

# node
npx serve .

# vite (if you want hot reload)
npx vite
```

Then open http://localhost:5173/.

## Deploy to GitHub Pages

1. Push this folder to a repo (either as the repo root, or under `/docs`).
2. Settings → Pages → Source → Deploy from a branch → `main` / `/` (or `/docs`).
3. Settings → Pages → Custom domain → `solarance-beginnings.com`.
4. At your DNS provider, point `solarance-beginnings.com` to GitHub Pages:
   - A records: `185.199.108.153`, `185.199.109.153`, `185.199.110.153`, `185.199.111.153`
   - (or CNAME `www` → `<you>.github.io`)
5. Tick "Enforce HTTPS" once the cert provisions.

The files that make this work without surprises:
- `.nojekyll` — disables Jekyll so the `.md` files in `devlog/` aren't mangled.
- `CNAME` — tells GitHub which custom domain this site uses.

## Adding a devlog post

1. Drop `devlog/NNN-slug.md` with frontmatter (see existing posts).
2. Add `"NNN-slug"` to the top of `devlog/index.json`.
3. Commit. GH Pages republishes in ~1 minute.

## Going live with SpacetimeDB

The Sector Map currently renders mock data shaped like your bindings.
To wire it to your real public module, follow the code snippet shown at the
bottom of the Sector Map page in `map.jsx`. You'll need to:

1. `npm i spacetimedb` (or vendor the runtime + your `ts_client_bindings/`)
2. Replace the `MOCK` object in `map.jsx` with the live `DbConnection` setup
3. Subscribe to the public read-only tables only — anonymous identity, no token

The renderer doesn't change.
