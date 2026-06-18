/* global React, ReactDOM */
const { useState, useEffect, useRef, useMemo } = React;

/* ============================================================
   TWEAKS — exposed defaults persisted via __edit_mode_set_keys
   ============================================================ */
const TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "crt": true,
  "scanIntensity": 10,
  "bloom": 55,
  "accent": "violet",
  "showStarfield": true
}/*EDITMODE-END*/;

const ACCENT_PRESETS = {
  violet: "oklch(0.74 0.16 305)",  // signature — cosmic violet
  cyan:   "oklch(0.80 0.10 220)",  // cool console blue
  amber:  "oklch(0.82 0.14 75)",   // warm terminal amber
  green:  "oklch(0.80 0.13 145)",  // phosphor green
  rose:   "oklch(0.78 0.12 25)",   // dim red — danger console
};

/* ============================================================
   Hash router
   ============================================================ */
function useHashRoute() {
  const [route, setRoute] = useState(() => (window.location.hash.replace(/^#\/?/, "") || "home"));
  useEffect(() => {
    const onHash = () => setRoute(window.location.hash.replace(/^#\/?/, "") || "home");
    window.addEventListener("hashchange", onHash);
    return () => window.removeEventListener("hashchange", onHash);
  }, []);
  return route;
}

/* ============================================================
   Shell: status bar + nav + footer
   ============================================================ */
function StatusBar() {
  const [time, setTime] = useState(() => fmtSpaceTime(new Date()));
  const [version, setVersion] = useState(() => loadCachedVersion());

  useEffect(() => {
    const id = setInterval(() => setTime(fmtSpaceTime(new Date())), 1000);
    return () => clearInterval(id);
  }, []);

  useEffect(() => {
    let cancelled = false;
    fetchLatestVersion().then(v => {
      if (!cancelled && v) {
        setVersion(v);
        cacheVersion(v);
      }
    });
    return () => { cancelled = true; };
  }, []);

  return (
    <div className="statusbar">
      <div className="row">
        <span className="dot" />
        <span>SOLARANCE-BEGINNINGS.COM // PUBLIC TERMINAL</span>
        <span style={{ color: "var(--accent)" }} title={version.source ? `Source: ${version.source}` : ""}>
          BUILD: {version.label}
        </span>
        <span className="spacer" />
        <span>GST {time}</span>
        <span className="blink" style={{ color: "var(--accent)" }}>●</span>
      </div>
    </div>
  );
}

/* ============================================================
   Version detection — GitHub releases → Cargo.toml → hardcoded
   ============================================================ */
const GITHUB_OWNER = "GalaxyCr8r";
const GITHUB_REPO  = "solarance-beginnings";
const FALLBACK_VERSION = { label: "0.4.0-pre-alpha", source: "fallback" };
const VERSION_CACHE_KEY = "solarance.version";
const VERSION_CACHE_TTL_MS = 60 * 60 * 1000; // 1 hour

function loadCachedVersion() {
  try {
    const raw = localStorage.getItem(VERSION_CACHE_KEY);
    if (!raw) return FALLBACK_VERSION;
    const parsed = JSON.parse(raw);
    if (Date.now() - parsed.ts < VERSION_CACHE_TTL_MS && parsed.label) {
      return { label: parsed.label, source: parsed.source };
    }
  } catch (_e) { /* ignore */ }
  return FALLBACK_VERSION;
}

function cacheVersion(v) {
  try {
    localStorage.setItem(VERSION_CACHE_KEY, JSON.stringify({ ...v, ts: Date.now() }));
  } catch (_e) { /* ignore (private mode etc) */ }
}

async function fetchLatestVersion() {
  // 1) GitHub releases API — the proper signal once you tag a release.
  try {
    const r = await fetch(
      `https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPO}/releases/latest`,
      { headers: { "Accept": "application/vnd.github+json" } }
    );
    if (r.ok) {
      const data = await r.json();
      const tag = (data.tag_name || "").replace(/^v/, "");
      if (tag) return { label: `${tag}-pre-alpha`, source: "gh-release" };
    }
  } catch (_e) { /* network/cors — fall through */ }

  // 2) Cargo.toml on main — covers between releases.
  try {
    const r = await fetch(
      `https://raw.githubusercontent.com/${GITHUB_OWNER}/${GITHUB_REPO}/main/client/Cargo.toml`
    );
    if (r.ok) {
      const text = await r.text();
      // Find the [package] block's version line (avoid matching [dependencies] lines)
      const pkgMatch = text.match(/\[package\][\s\S]*?(?=^\[|\Z)/m);
      const block = pkgMatch ? pkgMatch[0] : text;
      const m = block.match(/^\s*version\s*=\s*"([^"]+)"/m);
      if (m) return { label: `${m[1]}-pre-alpha`, source: "cargo.toml" };
    }
  } catch (_e) { /* fall through */ }

  return null;
}

function fmtSpaceTime(d) {
  // Galactic Standard Time — just the user's UTC, with cycle counter for flavor
  const h = String(d.getUTCHours()).padStart(2, "0");
  const m = String(d.getUTCMinutes()).padStart(2, "0");
  const s = String(d.getUTCSeconds()).padStart(2, "0");
  return `${h}:${m}:${s}`;
}

function Nav({ route }) {
  const links = [
    { id: "home",      label: "Bridge" },
    { id: "manifesto", label: "Manifesto" },
    { id: "roadmap",   label: "Roadmap" },
    { id: "map",       label: "System Map" },
    { id: "devlog",    label: "Devlog" },
    { id: "community", label: "Pilot's Lounge" },
  ];
  return (
    <div className="nav container">
      <a className="brand" href="#/home">
        <img src="assets/solarance-icon.png" alt="Solarance" className="brand-icon" />
        <span className="sub">// Beginnings</span>
      </a>
      <div className="links">
        {links.map(l => (
          <a key={l.id} href={`#/${l.id}`}
             className={"navlink" + (route === l.id ? " active" : "")}>
            {l.label}
          </a>
        ))}
      </div>
      <div className="right">
        <a className="btn ghost" href="https://discord.solarance-beginnings.com" target="_blank" rel="noopener">DISCORD →</a>
      </div>
    </div>
  );
}

function Footer() {
  return (
    <footer className="foot container">
      <span>© 2026 SOLARANCE — solo dev, pre-alpha</span>
      <span>Built on SpacetimeDB</span>
      <span className="spacer" />
      <span>Honest scope, monthly cadence.</span>
      <a href="#/manifesto">Mission</a>
      <a href="#/community">Discord</a>
    </footer>
  );
}

/* ============================================================
   Pages export — defined in content.jsx
   We expect: HomePage, ManifestoPage, RoadmapPage, MapPage, DevlogPage, CommunityPage
   ============================================================ */
function App() {
  const route = useHashRoute();
  const [tweaks, setTweaks] = (window.useTweaks || useFallbackTweaks)(TWEAK_DEFAULTS);

  // Apply tweaks to :root css vars + body class
  useEffect(() => {
    const root = document.documentElement;
    root.style.setProperty("--accent", ACCENT_PRESETS[tweaks.accent] || ACCENT_PRESETS.violet);
    root.style.setProperty("--scan-opacity", String(tweaks.scanIntensity / 100));
    root.style.setProperty("--crt-bloom", String(tweaks.bloom / 100));
    if (tweaks.crt) root.classList.add("crt"); else root.classList.remove("crt");
    document.body.style.setProperty("--starfield-display", tweaks.showStarfield ? "block" : "none");
    // toggle ::before via class
    if (tweaks.showStarfield) document.body.classList.remove("no-stars");
    else document.body.classList.add("no-stars");
  }, [tweaks]);

  // Scroll to top on route change
  useEffect(() => { window.scrollTo(0, 0); }, [route]);

  const topRoute = route.split("/")[0];
  const page = (() => {
    switch (topRoute) {
      case "manifesto": return <ManifestoPage />;
      case "roadmap":   return <RoadmapPage />;
      case "map":       return <MapPage />;
      case "devlog":    return <DevlogPage />;
      case "community": return <CommunityPage />;
      case "home":
      default:          return <HomePage />;
    }
  })();

  return (
    <div className="app">
      <StatusBar />
      <Nav route={topRoute} />
      {page}
      <Footer />
      <TweaksUI tweaks={tweaks} setTweaks={setTweaks} />
    </div>
  );
}

function useFallbackTweaks(defaults) {
  const [s, setS] = useState(defaults);
  const setter = (a, b) => {
    if (typeof a === "object") setS(p => ({ ...p, ...a }));
    else setS(p => ({ ...p, [a]: b }));
  };
  return [s, setter];
}

function TweaksUI({ tweaks, setTweaks }) {
  // Use the starter TweaksPanel + controls
  const TP = window.TweaksPanel, TSec = window.TweakSection,
        TT = window.TweakToggle, TS = window.TweakSlider, TR = window.TweakRadio;
  if (!TP) return null;
  return (
    <TP title="Tweaks // CRT Panel">
      <TSec label="Display">
        <TT label="CRT mode"           value={tweaks.crt}            onChange={v => setTweaks("crt", v)} />
        <TT label="Starfield"          value={tweaks.showStarfield}  onChange={v => setTweaks("showStarfield", v)} />
        <TS label="Scanline intensity" value={tweaks.scanIntensity}  onChange={v => setTweaks("scanIntensity", v)} min={0} max={30} step={1} unit="%" />
        <TS label="Phosphor bloom"     value={tweaks.bloom}          onChange={v => setTweaks("bloom", v)} min={0} max={100} step={5} unit="%" />
      </TSec>
      <TSec label="Console color">
        <TR label="Accent" value={tweaks.accent} onChange={v => setTweaks("accent", v)}
            options={["violet", "cyan", "amber", "green", "rose"]} />
      </TSec>
    </TP>
  );
}

window.App = App;
ReactDOM.createRoot(document.getElementById("root")).render(<App />);
