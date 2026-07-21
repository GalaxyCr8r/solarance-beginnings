# Releasing

Version lives in `server/`, `client/`, and `client-admin/` Cargo.toml
(`solarance-shared` is independent at 0.1.0). It is compiled into each binary via
`env!("CARGO_PKG_VERSION")`, shown on the login splash (`client/src/login.rs`) and
stored in `global_config` at server init (`server/src/lifecycle/init.rs`) — so
**the tagged commit's version MUST equal the release version.**

**Invariant:** `main` always carries the *next* version with a `-dev` suffix (e.g. `0.8.0-dev`).

To release `X.Y.Z`:

1. PR: drop the suffix in all three Cargo.toml (`X.Y.Z-dev` → `X.Y.Z`). Merge.
2. `gh release create vX.Y.Z --target main --generate-notes --latest`
   — the tag lands on the finalized commit; the splash now correctly shows `vX.Y.Z`.
3. PR: bump all three to the next planned `-dev` (e.g. `X.Y.Z` → `0.(Y+1).0-dev`). Merge.
   `main` resumes development.

Never tag before step 1: tagging a `-dev` (or otherwise stale) commit bakes the wrong
version into the binary, so the release displays a version that lags its tag.
