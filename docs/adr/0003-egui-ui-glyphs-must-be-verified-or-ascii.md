---
status: accepted
date: 2026-07-21
prompted-by: "#195"
---

# UI glyphs must be ASCII, or verified against egui's bundled fonts

## Context

The system map's sector-details panel has a close button, `ui.small_button("✕")` (`client/src/gameplay/gui/map_window.rs:382`). In the running client it renders as a tofu box (☐), not an X.

The client never calls `Context::set_fonts` — it relies entirely on the four fonts egui bundles via the `default_fonts` feature (`Ubuntu-Light`, `NotoEmoji-Regular`, `emoji-icon-font`, `Hack-Regular`, from the `epaint_default_fonts` crate). Parsing each font's `cmap` table directly shows the failure is a glyph-coverage gap, not a missing feature: `✕` (U+2715) isn't in any of the four, and `●` (U+25CF), used the same way in `faction_window.rs:231` and `chat_widget.rs:342`, is only in `Hack-Regular` — which is wired into egui's `Monospace` font family, not the `Proportional` family plain `ui.label`/`ui.button` text uses. Meanwhile `⚠` (U+26A0, `map_window.rs:402`) *is* covered by `NotoEmoji-Regular` and renders fine. There's no pattern a developer can eyeball — "is this a common symbol" doesn't predict "is this glyph in the four specific fonts egui shipped."

## Decision

UI code may only use a Unicode symbol/emoji glyph (anything outside plain ASCII) if it has been checked against the four bundled fonts' `cmap` tables first. Default to ASCII text (`"X"`, `"*"`, `"!"`) or an `egui::Painter`/`Shape` primitive (a drawn circle, a drawn X from two line segments) for anything icon-like. This is the same convention `minimap_widget.rs:90` already uses (`"X"` / `"O"` as plain letters, not symbol glyphs) — it just wasn't written down, so it wasn't followed consistently.

## Why this shape

- **Bundle a full icon font** (Font Awesome, Material Icons, `egui-phosphor`) — rejected for now. It fixes coverage for any icon, but it's a new dependency and asset-loading setup to cover what is currently three glyphs. Revisit if the UI grows enough icon usage that hand-picking ASCII/vector stand-ins becomes its own maintenance burden.
- **Vet each glyph against the four cmaps before using it** — viable, but easy to skip under normal development (nothing forces the check), and it produces UI that renders differently depending on which of four obscure fonts happened to contain the glyph. Rejected as the default; still fine for the rare case where an already-verified glyph is genuinely clearer than ASCII.
- **ASCII / drawn primitives** (chosen) — zero new dependencies, works with fonts already loaded, and is exactly what `minimap_widget.rs` already does. The ceiling is lower visual polish than a real icon font, which is an acceptable trade for an MVP-stage UI.

## Consequences

- The three broken call sites (`map_window.rs:382`, `faction_window.rs:231`, `chat_widget.rs:342`) are tracked in #195, not fixed by this ADR.
- `map_window.rs:402`'s `⚠` already renders correctly and does not need to change, but it's the one glyph in the codebase currently relying on unverified font luck rather than this rule — flag it if `NotoEmoji-Regular`/`emoji-icon-font` ever get swapped out.
- If icon needs grow past what ASCII/drawn primitives comfortably express, revisit the "bundle a real icon font" alternative above rather than accumulating more one-off verified glyphs.
