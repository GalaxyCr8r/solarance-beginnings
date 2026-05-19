/* global React */
const { useState: useStateM, useEffect: useEffectM, useRef: useRefM, useMemo: useMemoM } = React;

/* ============================================================
   SECTOR MAP — honest single-system view (the MVP)

   This page is *designed* to swap to a live SpacetimeDB connection.
   The mock data below has the exact shape your generated bindings emit
   (snake_case → camelCase keys: see ts_client_bindings/sector_table.ts).

   To go live, replace the mock store with:

   ----------------------------------------------------------------
   import { DbConnection } from "../ts_client_bindings";

   const conn = DbConnection.builder()
     .withUri("wss://maincloud.spacetimedb.com")
     .withModuleName("solarance-beginnings")
     // No .withToken(...) → server issues an anonymous identity.
     .onConnect((ctx, identity, token) => {
       // For a public read-only galaxy map: subscribe to PUBLIC tables only.
       ctx.subscriptionBuilder().subscribe([
         "SELECT * FROM star_system",
         "SELECT * FROM sector",
         "SELECT * FROM faction",
         "SELECT * FROM jump_gate",
         "SELECT * FROM station",
         "SELECT * FROM station_under_construction",
       ]);
     })
     .build();

   // Read live state from the local cache:
   const sectors = [...conn.db.sector.iter()];
   conn.db.sector.onInsert((ctx, row) => setSectors(s => [...s, row]));
   ----------------------------------------------------------------

   Until then, this page renders MOCK rows that match the bindings shape.
   ============================================================ */

const FACTION_LRAK_ID = 1;
const FACTION_REDIAR_ID = 2;
const FACTION_NEUTRAL = 0;
const FACTION_CONTESTED = 99;

const MOCK = {
  star_system: [{
    id: 1,
    name: "Sol Veridian",
    map_coordinates: { x: 0, y: 0 },
    spectral: { tag: "G" },
    luminosity: 5,
    controlling_faction_id: FACTION_NEUTRAL,
  }],
  faction: [
    { id: FACTION_REDIAR_ID, name: "Rediar Federation", short_name: "Rediar", description: "The blue banner.", joinable: true,  capital_station_id: 1001 },
    { id: FACTION_LRAK_ID,   name: "Lrak Combine",      short_name: "Lrak",   description: "The red banner.",  joinable: true,  capital_station_id: 1002 },
    { id: FACTION_NEUTRAL,   name: "Unaligned",         short_name: "—",      description: "No flag.",         joinable: false, capital_station_id: null },
  ],
  // 10 hand-placed sectors per MVP doc. Pixel coords in 0..100 (we scale to canvas).
  sector: [
    { id: 1,  system_id: 1, name: "Echo Bay",        description: "Rediar capital sector. New pilots spawn here.",         controlling_faction_id: FACTION_REDIAR_ID, security_level: 9, sunlight: 0.9, anomalous: 0.0, nebula: 0.1, rare_ore: 0.1, x: 28, y: 32, background_gfx_key: "bg_clean" },
    { id: 2,  system_id: 1, name: "Iron Furrow",     description: "Dense, low-quality asteroid field. Tutorial mining.",   controlling_faction_id: FACTION_REDIAR_ID, security_level: 8, sunlight: 0.8, anomalous: 0.0, nebula: 0.0, rare_ore: 0.2, x: 18, y: 48, background_gfx_key: null },
    { id: 3,  system_id: 1, name: "Pale Crossing",   description: "Jumpgate hub between Rediar and the neutral mid-ring.", controlling_faction_id: FACTION_REDIAR_ID, security_level: 7, sunlight: 0.6, anomalous: 0.0, nebula: 0.4, rare_ore: 0.0, x: 38, y: 56, background_gfx_key: null },
    { id: 4,  system_id: 1, name: "Quiet Belt",      description: "Neutral asteroid sector. Both factions mine here.",     controlling_faction_id: FACTION_NEUTRAL,   security_level: 5, sunlight: 0.5, anomalous: 0.1, nebula: 0.2, rare_ore: 0.5, x: 50, y: 50, background_gfx_key: null },
    { id: 5,  system_id: 1, name: "The Hinge",       description: "Central jumpgate hub. Most traffic passes through.",    controlling_faction_id: FACTION_NEUTRAL,   security_level: 5, sunlight: 0.4, anomalous: 0.0, nebula: 0.6, rare_ore: 0.0, x: 50, y: 32, background_gfx_key: null },
    { id: 6,  system_id: 1, name: "The Seam",        description: "Contested. Both factions building here. Slow progress.", controlling_faction_id: FACTION_CONTESTED, security_level: 3, sunlight: 0.3, anomalous: 0.2, nebula: 0.5, rare_ore: 0.4, x: 62, y: 44, background_gfx_key: null },
    { id: 7,  system_id: 1, name: "Karren's Reach",  description: "Lrak frontier. Refinery sector.",                       controlling_faction_id: FACTION_LRAK_ID,   security_level: 7, sunlight: 0.7, anomalous: 0.0, nebula: 0.1, rare_ore: 0.1, x: 72, y: 36, background_gfx_key: null },
    { id: 8,  system_id: 1, name: "Lrakhold",        description: "Lrak capital sector. Spawn point for red pilots.",      controlling_faction_id: FACTION_LRAK_ID,   security_level: 9, sunlight: 0.8, anomalous: 0.0, nebula: 0.1, rare_ore: 0.1, x: 82, y: 22, background_gfx_key: "bg_clean" },
    { id: 9,  system_id: 1, name: "Ore Trench",      description: "Lrak-aligned. Highest known rare-ore density.",         controlling_faction_id: FACTION_LRAK_ID,   security_level: 6, sunlight: 0.6, anomalous: 0.0, nebula: 0.2, rare_ore: 0.8, x: 78, y: 58, background_gfx_key: null },
    { id: 10, system_id: 1, name: "Stillwater",      description: "Quiet, slow sector. Research module planned.",          controlling_faction_id: FACTION_NEUTRAL,   security_level: 8, sunlight: 0.7, anomalous: 0.4, nebula: 0.3, rare_ore: 0.0, x: 32, y: 70, background_gfx_key: null },
  ],
  jump_gate: [
    // From → to (we render as edges)
    { id: 1, from_sector_id: 1,  to_sector_id: 2 },
    { id: 2, from_sector_id: 1,  to_sector_id: 3 },
    { id: 3, from_sector_id: 3,  to_sector_id: 4 },
    { id: 4, from_sector_id: 4,  to_sector_id: 5 },
    { id: 5, from_sector_id: 1,  to_sector_id: 5 },
    { id: 6, from_sector_id: 5,  to_sector_id: 6 },
    { id: 7, from_sector_id: 6,  to_sector_id: 7 },
    { id: 8, from_sector_id: 7,  to_sector_id: 8 },
    { id: 9, from_sector_id: 6,  to_sector_id: 9 },
    { id: 10, from_sector_id: 9, to_sector_id: 7 },
    { id: 11, from_sector_id: 4, to_sector_id: 10 },
    { id: 12, from_sector_id: 3, to_sector_id: 10 },
  ],
  // One station per occupied sector + faction
  station: [
    { id: 1001, sector_id: 1,  owner_faction_id: FACTION_REDIAR_ID, name: "Outpost Echo (Capital)",  size: { tag: "Capital" } },
    { id: 1002, sector_id: 8,  owner_faction_id: FACTION_LRAK_ID,   name: "Lrakhold Anchor (Capital)", size: { tag: "Capital" } },
    { id: 1003, sector_id: 2,  owner_faction_id: FACTION_REDIAR_ID, name: "Furrow Refinery",          size: { tag: "Small" } },
    { id: 1004, sector_id: 3,  owner_faction_id: FACTION_REDIAR_ID, name: "Pale Gate Yard",           size: { tag: "Medium" } },
    { id: 1005, sector_id: 7,  owner_faction_id: FACTION_LRAK_ID,   name: "Karren Refinery",          size: { tag: "Medium" } },
    { id: 1006, sector_id: 9,  owner_faction_id: FACTION_LRAK_ID,   name: "Trench Foundry",           size: { tag: "Small" } },
    { id: 1007, sector_id: 6,  owner_faction_id: FACTION_CONTESTED, name: "Seam Frame",               size: { tag: "Small" } },
  ],
  station_under_construction: [
    { id: 1003, is_operational: true,  construction_progress_percentage: 100 },
    { id: 1004, is_operational: false, construction_progress_percentage: 62 },
    { id: 1005, is_operational: false, construction_progress_percentage: 48 },
    { id: 1006, is_operational: false, construction_progress_percentage: 81 },
    { id: 1007, is_operational: false, construction_progress_percentage: 34 },
  ],
  // Live ping-style data we'd subscribe to: rolling contribution counter
  contribution_log_recent: 142, // mock
  online_pilots: 6, // mock
};

function factionClassFor(id) {
  if (id === FACTION_LRAK_ID)      return "lrak";
  if (id === FACTION_REDIAR_ID)    return "rediar";
  if (id === FACTION_CONTESTED)    return "contested";
  return "neutral";
}
function factionNameFor(id) {
  return MOCK.faction.find(f => f.id === id)?.name || "Unaligned";
}

function MapPage() {
  const [selectedId, setSelectedId] = useStateM(7); // default focus: Karren's Reach
  const [conn, setConn] = useStateM("snapshot"); // 'snapshot' | 'live' | 'connecting'
  const [hoverEdge, setHoverEdge] = useStateM(null);

  // Simulate a "connect to live galaxy" pretend-action (UI only).
  const tryLiveConnect = () => {
    setConn("connecting");
    setTimeout(() => {
      // In reality: DbConnection.builder()...build() would resolve here.
      // For now we stay in snapshot mode but flip the pill to "live (mock)" for the demo.
      setConn("live");
    }, 1400);
  };

  const selected = MOCK.sector.find(s => s.id === selectedId);
  const selectedStations = MOCK.station.filter(st => st.sector_id === selectedId);
  const selectedConstruction = selectedStations.map(st => ({
    station: st,
    prog: MOCK.station_under_construction.find(c => c.id === st.id),
  }));

  // Compute edge positions
  const edges = MOCK.jump_gate.map(g => {
    const a = MOCK.sector.find(s => s.id === g.from_sector_id);
    const b = MOCK.sector.find(s => s.id === g.to_sector_id);
    if (!a || !b) return null;
    return { id: g.id, ax: a.x, ay: a.y, bx: b.x, by: b.y, a, b };
  }).filter(Boolean);

  return (
    <main className="container" style={{ padding: "44px 18px 60px" }}>
      <div style={{ display: "flex", alignItems: "end", gap: 18, flexWrap: "wrap", marginBottom: 18 }}>
        <div style={{ flex: 1, minWidth: 260 }}>
          <div className="kicker accent-bloom">▸ One system. Ten sectors. Hand-placed.</div>
          <h1 style={{ marginTop: 6 }}>Sol Veridian</h1>
          <p style={{ color: "var(--fg-dim)", marginTop: 8, maxWidth: "62ch" }}>
            The MVP galaxy is one system. That's a feature, not a limitation —
            the social gravity is what makes shared building meaningful.
          </p>
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: 10, alignItems: "flex-end" }}>
          <ConnPill state={conn} onConnect={tryLiveConnect} />
          <div style={{ display: "flex", gap: 8 }}>
            <span className="tag dim">10 sectors</span>
            <span className="tag dim">12 jumpgates</span>
            <span className="tag accent">{MOCK.online_pilots} pilots online</span>
          </div>
        </div>
      </div>

      <div className="map-shell">
        {/* SVG edge layer */}
        <svg viewBox="0 0 100 100" preserveAspectRatio="none"
             style={{ position: "absolute", inset: 0, width: "100%", height: "100%", zIndex: 1 }}>
          <defs>
            <linearGradient id="edgeGrad" x1="0" x2="1" y1="0" y2="0">
              <stop offset="0%"  stopColor="var(--line)" />
              <stop offset="50%" stopColor="var(--fg-muted)" stopOpacity="0.5" />
              <stop offset="100%" stopColor="var(--line)" />
            </linearGradient>
          </defs>
          {edges.map(e => {
            const isSel = (e.a.id === selectedId || e.b.id === selectedId);
            return (
              <line key={e.id}
                x1={e.ax} y1={e.ay} x2={e.bx} y2={e.by}
                stroke={isSel ? "var(--accent)" : "url(#edgeGrad)"}
                strokeWidth={isSel ? 0.18 : 0.10}
                strokeDasharray={isSel ? "0" : "0.4 0.4"}
                opacity={isSel ? 0.85 : 0.55}
                vectorEffect="non-scaling-stroke"
                onMouseEnter={() => setHoverEdge(e.id)}
                onMouseLeave={() => setHoverEdge(null)}
              />
            );
          })}
        </svg>

        {/* Sector nodes */}
        {MOCK.sector.map(s => {
          const cls = factionClassFor(s.controlling_faction_id);
          const isSel = s.id === selectedId;
          return (
            <div key={s.id}
                 className={"sector-node " + cls + (isSel ? " active" : "")}
                 style={{ top: `${s.y}%`, left: `${s.x}%`, zIndex: isSel ? 8 : 3 }}
                 onClick={() => setSelectedId(s.id)}>
              <div className="ring" />
              <div className="pip" />
              <div className="lab" style={{ color: isSel ? "var(--accent)" : "var(--fg)" }}>{s.name}</div>
              <div className="sub">SEC {s.security_level} · {factionNameFor(s.controlling_faction_id).split(" ")[0]}</div>
            </div>
          );
        })}

        {/* Overlays */}
        <div className="map-overlay tl">
          <h4>// Sol Veridian</h4>
          <div className="detail-row"><span className="k">Spectral</span><span className="v">G-class</span></div>
          <div className="detail-row"><span className="k">Sectors</span><span className="v">10 / 10</span></div>
          <div className="detail-row"><span className="k">Status</span><span className="v" style={{color:"var(--green)"}}>Active</span></div>
          <hr style={{ margin: "10px 0" }} />
          <div className="detail-row" style={{ fontSize: 11 }}>
            <span className="k">Live data</span>
            <span className="v" style={{ color: conn === "live" ? "var(--green)" : "var(--fg-muted)"}}>
              {conn === "live" ? "SUBSCRIBED" : "SNAPSHOT"}
            </span>
          </div>
        </div>

        <div className="map-overlay tr">
          <h4>// Legend</h4>
          <div className="legend-row"><span className="legend-dot rediar" />Rediar Federation</div>
          <div className="legend-row"><span className="legend-dot lrak" />Lrak Combine</div>
          <div className="legend-row"><span className="legend-dot neutral" />Unaligned</div>
          <div className="legend-row"><span className="legend-dot contested" />Contested</div>
          <hr style={{ margin: "10px 0" }} />
          <div className="legend-row" style={{ fontSize: 11, color:"var(--fg-muted)" }}>
            <span style={{ width: 18, height: 1, background: "var(--fg-muted)" }} />
            Jumpgate link
          </div>
        </div>

        {/* Selected sector detail */}
        {selected && (
          <div className="map-overlay br">
            <h4 style={{ display: "flex", justifyContent: "space-between", alignItems: "center", gap: 12 }}>
              <span>// {selected.name}</span>
              <span className={"tag " + factionClassFor(selected.controlling_faction_id).replace("contested","warn").replace("neutral","dim")}>
                {factionNameFor(selected.controlling_faction_id)}
              </span>
            </h4>
            <p style={{ color: "var(--fg-dim)", fontSize: 12, marginBottom: 10, marginTop: 4 }}>
              {selected.description}
            </p>
            <div className="detail-row"><span className="k">Security</span><span className="v">Level {selected.security_level} / 10</span></div>
            <div className="detail-row"><span className="k">Sunlight</span><span className="v">{(selected.sunlight*100).toFixed(0)}%</span></div>
            <div className="detail-row"><span className="k">Nebula</span><span className="v">{(selected.nebula*100).toFixed(0)}%</span></div>
            <div className="detail-row"><span className="k">Rare ore</span><span className="v">{(selected.rare_ore*100).toFixed(0)}%</span></div>
            <div className="detail-row"><span className="k">Anomalous</span><span className="v">{(selected.anomalous*100).toFixed(0)}%</span></div>

            {selectedConstruction.length > 0 && (
              <>
                <hr style={{ margin: "10px 0" }} />
                <div style={{ fontSize: 11, color: "var(--accent)", letterSpacing: ".18em", textTransform: "uppercase", marginBottom: 6 }}>
                  Stations ({selectedConstruction.length})
                </div>
                {selectedConstruction.map(({ station, prog }) => (
                  <div key={station.id} style={{ marginBottom: 8 }}>
                    <div className="detail-row" style={{ paddingBottom: 2, fontSize: 12 }}>
                      <span style={{ color: "var(--fg)" }}>{station.name}</span>
                      <span style={{ color: prog?.is_operational ? "var(--green)" : "var(--accent)" }}>
                        {prog ? (prog.is_operational ? "OPERATIONAL" : `${prog.construction_progress_percentage.toFixed(0)}%`) : "—"}
                      </span>
                    </div>
                    {prog && !prog.is_operational && (
                      <div className="progress"><div className="fill" style={{ width: prog.construction_progress_percentage + "%" }} /></div>
                    )}
                  </div>
                ))}
              </>
            )}
          </div>
        )}

        <div className="map-overlay bl">
          <h4>// Live counters</h4>
          <div className="detail-row"><span className="k">Pilots online</span><span className="v" style={{color:"var(--accent)"}}>{MOCK.online_pilots}</span></div>
          <div className="detail-row"><span className="k">Contributions (24h)</span><span className="v" style={{color:"var(--accent)"}}>{MOCK.contribution_log_recent}</span></div>
          <div className="detail-row" style={{ fontSize: 10.5 }}><span className="k" style={{whiteSpace:"nowrap"}}>Galactic time</span><span className="v"><GalTime /></span></div>
        </div>
      </div>

      {/* Below the map: the integration story */}
      <div className="grid-2" style={{ marginTop: 28 }}>
        <div className="bracket"><span className="br-tr" /><span className="br-bl" />
          <div className="kicker">// data source</div>
          <h3 style={{ marginTop: 8 }}>Pulled from SpacetimeDB. Eventually.</h3>
          <p style={{ color: "var(--fg-dim)", marginTop: 8 }}>
            The map above is a <b>snapshot</b> shaped exactly like the public tables in your dev module:
            <code style={{ color: "var(--accent)" }}> star_system</code>,&nbsp;
            <code style={{ color: "var(--accent)" }}>sector</code>,&nbsp;
            <code style={{ color: "var(--accent)" }}>faction</code>,&nbsp;
            <code style={{ color: "var(--accent)" }}>jump_gate</code>,&nbsp;
            <code style={{ color: "var(--accent)" }}>station</code>,&nbsp;
            <code style={{ color: "var(--accent)" }}>station_under_construction</code>.
          </p>
          <p style={{ color: "var(--fg-dim)" }}>
            When the public testnet is up, this page swaps in an anonymous
            <code style={{ color: "var(--accent)" }}> DbConnection</code> with read-only subscriptions
            over those tables. The renderer doesn't change.
          </p>
        </div>
        <div className="terminal">
          <div style={{ color: "var(--fg-muted)", marginBottom: 6 }}>// solarance/map-conn.ts (drop-in)</div>
          <div><span style={{color:"var(--accent)"}}>import</span> {"{ DbConnection }"} <span style={{color:"var(--accent)"}}>from</span> "../ts_client_bindings";</div>
          <div style={{ marginTop: 8 }}><span style={{color:"var(--accent)"}}>const</span> conn = DbConnection.builder()</div>
          <div>{"  "}.withUri(<span style={{color:"var(--green)"}}>"wss://maincloud.spacetimedb.com"</span>)</div>
          <div>{"  "}.withModuleName(<span style={{color:"var(--green)"}}>"solarance-beginnings"</span>)</div>
          <div>{"  "}<span style={{color:"var(--fg-muted)"}}>// no .withToken → anonymous identity</span></div>
          <div>{"  "}.onConnect((ctx) =&gt; {"{"}</div>
          <div>{"    "}ctx.subscriptionBuilder().subscribe([</div>
          <div>{"      "}<span style={{color:"var(--green)"}}>"SELECT * FROM sector"</span>,</div>
          <div>{"      "}<span style={{color:"var(--green)"}}>"SELECT * FROM jump_gate"</span>,</div>
          <div>{"      "}<span style={{color:"var(--green)"}}>"SELECT * FROM station"</span>,</div>
          <div>{"      "}<span style={{color:"var(--green)"}}>"SELECT * FROM station_under_construction"</span>,</div>
          <div>{"      "}<span style={{color:"var(--green)"}}>"SELECT * FROM faction"</span>,</div>
          <div>{"    "}]);</div>
          <div>{"  "}{"}"})</div>
          <div>{"  "}.build();</div>
          <div style={{ marginTop: 6, color: "var(--fg-muted)" }}>// then: [...conn.db.sector.iter()] gives you the live state.</div>
        </div>
      </div>
    </main>
  );
}

function ConnPill({ state, onConnect }) {
  if (state === "live") {
    return (
      <span className="conn-pill live"><span className="dot" /> LIVE · SUBSCRIBED · ANON</span>
    );
  }
  if (state === "connecting") {
    return <span className="conn-pill"><span className="dot" /> CONNECTING…</span>;
  }
  return (
    <button className="conn-pill snapshot" onClick={onConnect}
            style={{ cursor: "pointer" }}
            title="Demo: pretends to connect. The real wire-up is shown below the map.">
      <span className="dot" /> SNAPSHOT · click to demo live →
    </button>
  );
}

function GalTime() {
  const [t, setT] = useStateM(() => galStr());
  useEffectM(() => {
    const id = setInterval(() => setT(galStr()), 1000);
    return () => clearInterval(id);
  }, []);
  return <span>{t}</span>;
}
function galStr() {
  const d = new Date();
  const cycle = Math.floor(d.getTime() / 86400000) - 20100; // arbitrary epoch
  return `C${cycle}·${String(d.getUTCHours()).padStart(2,"0")}${String(d.getUTCMinutes()).padStart(2,"0")}`;
}

window.MapPage = MapPage;
