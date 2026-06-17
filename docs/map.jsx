/* global React */
const { useState: useStateM, useEffect: useEffectM, useRef: useRefM, useMemo: useMemoM } = React;

/* ============================================================
   SYSTEM MAP — the current star system: its sectors + orbital objects.

   Renders the MOCK snapshot below until you connect. "Connect" then reads
   the live galaxy from SpacetimeDB's anonymous, read-only HTTP `/sql`
   endpoint (no SDK, no build step, no bindings — see `fetchGalaxy`), and
   reshapes the rows into the same structure MOCK uses so the renderer is
   unchanged. Public tables only:
     star_system · sector · star_system_object · jump_gate · station ·
     station_under_construction · faction

   Host defaults to maincloud; override via the input or ?host=… .
   If a host ever gates `/sql`, swap `sqlQuery` for a websocket subscription.
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

// ---- Live data (issue #162) ----------------------------------------------
// No-build site, so we read the public tables straight off SpacetimeDB's
// anonymous read-only HTTP `/sql` endpoint and reshape them like MOCK above —
// one fetch per connect, no SDK/bundler/bindings. If a host ever gates `/sql`,
// `sqlQuery` is the single function to swap for a websocket subscription.
const LIVE_DB = "solarance-beginnings";
const DEFAULT_HOST =
  new URLSearchParams(location.search).get("host") || "https://maincloud.spacetimedb.com";

// star_system_object.kind is a sum type; SQL JSON encodes it as [index, payload].
const ORBIT_KINDS = ["Star", "Planet", "Moon", "AsteroidBelt", "NebulaBelt"];
function kindName(v) {
  const idx = Array.isArray(v) ? v[0] : v;
  return ORBIT_KINDS[idx] ?? String(idx);
}

// One SQL statement -> [{ col: value }]. Rows come back positional with a schema.
function sqlRows(result) {
  const t = Array.isArray(result) ? result[0] : result;
  if (!t || !t.schema) return [];
  const cols = t.schema.elements.map(e => (e.name && e.name.some) || e.name);
  return t.rows.map(r => Object.fromEntries(cols.map((c, i) => [c, r[i]])));
}
async function sqlQuery(host, sql) {
  const res = await fetch(`${host}/v1/database/${LIVE_DB}/sql`, {
    method: "POST",
    headers: { "Content-Type": "text/plain" },
    body: sql,
  });
  if (!res.ok) throw new Error(`SQL ${res.status}: ${(await res.text()).slice(0, 120)}`);
  return sqlRows(await res.json());
}

// Pull the whole galaxy and reshape into MOCK's structure so the renderer is
// unchanged. Sector x/y are system-space; normalise to the 0..100 percent grid
// the layout uses, and project the orbital objects with the same transform.
async function fetchGalaxy(host) {
  const [star_system, sectorsRaw, gatesRaw, station, station_under_construction, faction, orbitsRaw] =
    await Promise.all([
      sqlQuery(host, "SELECT id, name FROM star_system"),
      sqlQuery(host, "SELECT id, name, system_id, controlling_faction_id, security_level, sunlight, anomalous, nebula, rare_ore, x, y FROM sector"),
      sqlQuery(host, "SELECT current_sector_id, target_sector_id FROM jump_gate"),
      sqlQuery(host, "SELECT id, name, sector_id, owner_faction_id FROM station"),
      sqlQuery(host, "SELECT id, is_operational, construction_progress_percentage FROM station_under_construction"),
      sqlQuery(host, "SELECT id, name, short_name FROM faction"),
      sqlQuery(host, "SELECT system_id, kind, orbit_au, rotation_or_width_km FROM star_system_object"),
    ]);

  if (!sectorsRaw.length) throw new Error("No sectors returned from this host/database.");

  // Normalise sector coords -> 0..100 percent (uniform scale, padded).
  const PAD = 12, SPAN_PCT = 100 - 2 * PAD;
  const xs = sectorsRaw.map(s => s.x), ys = sectorsRaw.map(s => s.y);
  const minX = Math.min(...xs), maxX = Math.max(...xs);
  const minY = Math.min(...ys), maxY = Math.max(...ys);
  const span = Math.max(maxX - minX, maxY - minY, 1);
  const cx = (minX + maxX) / 2, cy = (minY + maxY) / 2;
  const nx = x => 50 + ((x - cx) / span) * SPAN_PCT;
  const ny = y => 50 + ((y - cy) / span) * SPAN_PCT;

  const sector = sectorsRaw.map(s => ({ ...s, x: nx(s.x), y: ny(s.y) }));

  // Dedup bidirectional gates into single from/to pairs.
  const seen = new Set();
  const jump_gate = [];
  let gid = 1;
  for (const g of gatesRaw) {
    const a = Number(g.current_sector_id), b = Number(g.target_sector_id);
    const key = a < b ? `${a}:${b}` : `${b}:${a}`;
    if (seen.has(key)) continue;
    seen.add(key);
    jump_gate.push({ id: gid++, from_sector_id: a, to_sector_id: b });
  }

  // Orbital backdrop: belts are rings on the system origin, the rest are points.
  const originX = nx(0), originY = ny(0);
  const orbital = orbitsRaw.map((o, i) => {
    const kind = kindName(o.kind);
    const belt = kind === "AsteroidBelt" || kind === "NebulaBelt";
    return {
      id: i, kind, belt,
      px: belt ? originX : nx(Math.cos(o.rotation_or_width_km) * o.orbit_au),
      py: belt ? originY : ny(Math.sin(o.rotation_or_width_km) * o.orbit_au),
      r: (Math.abs(o.orbit_au) / span) * SPAN_PCT,
    };
  });

  return {
    star_system, sector, jump_gate, station, station_under_construction,
    faction, orbital,
    online_pilots: 0, contribution_log_recent: 0,
  };
}

function factionClassFor(id, factions) {
  const name = (factions.find(f => f.id === id) || {}).name || "";
  if (/lrak/i.test(name)) return "lrak";
  if (/rediar/i.test(name)) return "rediar";
  return "neutral";
}
function factionNameFor(id, factions) {
  return (factions.find(f => f.id === id) || {}).name || "Unaligned";
}

function MapPage() {
  const [data, setData] = useStateM(MOCK);
  const [selectedId, setSelectedId] = useStateM(7); // default focus
  const [conn, setConn] = useStateM("snapshot"); // 'snapshot' | 'live' | 'connecting'
  const [hoverEdge, setHoverEdge] = useStateM(null);
  const [host, setHost] = useStateM(DEFAULT_HOST);
  const [err, setErr] = useStateM(null);

  // Connect: read the live galaxy off the public SQL endpoint and swap it in.
  // The renderer below is unchanged — live data is reshaped to match MOCK.
  const tryLiveConnect = async () => {
    setConn("connecting");
    setErr(null);
    try {
      const live = await fetchGalaxy(host.trim().replace(/\/+$/, ""));
      setData(live);
      if (live.sector[0]) setSelectedId(live.sector[0].id);
      setConn("live");
    } catch (e) {
      setErr(String(e && e.message ? e.message : e));
      setConn("snapshot");
    }
  };

  const factions = data.faction || [];
  const orbital = data.orbital || [];
  const systemName =
    (data.star_system && data.star_system[0] && data.star_system[0].name) || "Sol Veridian";

  const selected = data.sector.find(s => s.id === selectedId);
  const selectedStations = data.station.filter(st => st.sector_id === selectedId);
  const selectedConstruction = selectedStations.map(st => ({
    station: st,
    prog: data.station_under_construction.find(c => c.id === st.id),
  }));

  // Compute edge positions
  const edges = data.jump_gate.map(g => {
    const a = data.sector.find(s => s.id === g.from_sector_id);
    const b = data.sector.find(s => s.id === g.to_sector_id);
    if (!a || !b) return null;
    return { id: g.id, ax: a.x, ay: a.y, bx: b.x, by: b.y, a, b };
  }).filter(Boolean);

  return (
    <main className="container" style={{ padding: "44px 18px 60px" }}>
      <div style={{ display: "flex", alignItems: "end", gap: 18, flexWrap: "wrap", marginBottom: 18 }}>
        <div style={{ flex: 1, minWidth: 260 }}>
          <div className="kicker accent-bloom">▸ System Map · sectors + orbital objects</div>
          <h1 style={{ marginTop: 6 }}>{systemName}</h1>
          <p style={{ color: "var(--fg-dim)", marginTop: 8, maxWidth: "62ch" }}>
            One system: every sector and its jumpgate network, with the orbital
            objects as a faded backdrop. Connect to a live SpacetimeDB host to
            replace this snapshot with the real galaxy.
          </p>
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: 10, alignItems: "flex-end" }}>
          <ConnPill state={conn} onConnect={tryLiveConnect} />
          <input
            value={host}
            onChange={e => setHost(e.target.value)}
            spellCheck={false}
            style={{ width: 240, fontFamily: "inherit", fontSize: 11, padding: "4px 6px",
                     background: "#111", color: "var(--fg)", border: "1px solid var(--line)" }}
            title="SpacetimeDB host (HTTP). Append ?host=… to the URL to preset it."
          />
          {err && <span className="tag" style={{ color: "#f66" }}>connect failed: {err}</span>}
          <div style={{ display: "flex", gap: 8 }}>
            <span className="tag dim">{data.sector.length} sectors</span>
            <span className="tag dim">{edges.length} jumpgates</span>
            <span className="tag accent">{data.online_pilots} pilots online</span>
          </div>
        </div>
      </div>

      <div className="map-shell">
        {/* Orbital backdrop (faded): star/planets/moons as dots, belts as rings */}
        <svg viewBox="0 0 100 100" preserveAspectRatio="none"
             style={{ position: "absolute", inset: 0, width: "100%", height: "100%", zIndex: 0, opacity: 0.5 }}>
          {orbital.map(o => o.belt ? (
            <circle key={o.id} cx={o.px} cy={o.py} r={o.r} fill="none"
                    stroke={o.kind === "NebulaBelt" ? "#b545ff" : "#7a4628"}
                    strokeWidth="0.15" opacity="0.6" vectorEffect="non-scaling-stroke" />
          ) : (
            <circle key={o.id} cx={o.px} cy={o.py}
                    r={o.kind === "Star" ? 1.6 : o.kind === "Planet" ? 0.9 : 0.5}
                    fill="none"
                    stroke={o.kind === "Star" ? "#ffd34d" : o.kind === "Moon" ? "#999" : "#9cc7ff"}
                    strokeWidth="0.2" vectorEffect="non-scaling-stroke" />
          ))}
        </svg>

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
        {data.sector.map(s => {
          const cls = factionClassFor(s.controlling_faction_id, factions);
          const isSel = s.id === selectedId;
          return (
            <div key={s.id}
                 className={"sector-node " + cls + (isSel ? " active" : "")}
                 style={{ top: `${s.y}%`, left: `${s.x}%`, zIndex: isSel ? 8 : 3 }}
                 onClick={() => setSelectedId(s.id)}>
              <div className="ring" />
              <div className="pip" />
              <div className="lab" style={{ color: isSel ? "var(--accent)" : "var(--fg)" }}>{s.name}</div>
              <div className="sub">SEC {s.security_level} · {factionNameFor(s.controlling_faction_id, factions).split(" ")[0]}</div>
            </div>
          );
        })}

        {/* Overlays */}
        <div className="map-overlay tl">
          <h4>// {systemName}</h4>
          <div className="detail-row"><span className="k">Sectors</span><span className="v">{data.sector.length}</span></div>
          <div className="detail-row"><span className="k">Jumpgates</span><span className="v">{edges.length}</span></div>
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
              <span className={"tag " + factionClassFor(selected.controlling_faction_id, factions).replace("neutral","dim")}>
                {factionNameFor(selected.controlling_faction_id, factions)}
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
          <div className="detail-row"><span className="k">Pilots online</span><span className="v" style={{color:"var(--accent)"}}>{data.online_pilots}</span></div>
          <div className="detail-row"><span className="k">Contributions (24h)</span><span className="v" style={{color:"var(--accent)"}}>{data.contribution_log_recent}</span></div>
          <div className="detail-row" style={{ fontSize: 10.5 }}><span className="k" style={{whiteSpace:"nowrap"}}>Galactic time</span><span className="v"><GalTime /></span></div>
        </div>
      </div>

      {/* Below the map: the integration story */}
      <div className="grid-2" style={{ marginTop: 28 }}>
        <div className="bracket"><span className="br-tr" /><span className="br-bl" />
          <div className="kicker">// data source</div>
          <h3 style={{ marginTop: 8 }}>Pulled from SpacetimeDB. Eventually.</h3>
          <p style={{ color: "var(--fg-dim)", marginTop: 8 }}>
            Until you connect, the map is a <b>snapshot</b>. Hit connect and it
            reads the live galaxy straight from the public tables —
            <code style={{ color: "var(--accent)" }}> star_system</code>,&nbsp;
            <code style={{ color: "var(--accent)" }}>sector</code>,&nbsp;
            <code style={{ color: "var(--accent)" }}>faction</code>,&nbsp;
            <code style={{ color: "var(--accent)" }}>jump_gate</code>,&nbsp;
            <code style={{ color: "var(--accent)" }}>station</code>,&nbsp;
            <code style={{ color: "var(--accent)" }}>star_system_object</code>.
          </p>
          <p style={{ color: "var(--fg-dim)" }}>
            No SDK, no build step: a single anonymous, read-only POST to
            SpacetimeDB's <code style={{ color: "var(--accent)" }}>/sql</code> HTTP
            endpoint, reshaped into the same structure the renderer already draws.
          </p>
        </div>
        <div className="terminal">
          <div style={{ color: "var(--fg-muted)", marginBottom: 6 }}>// how the map goes live (no build step)</div>
          <div><span style={{color:"var(--accent)"}}>POST</span> {"{host}"}/v1/database/solarance-beginnings/sql</div>
          <div>{"  "}<span style={{color:"var(--fg-muted)"}}>body:</span> <span style={{color:"var(--green)"}}>"SELECT * FROM sector"</span></div>
          <div style={{ marginTop: 8, color: "var(--fg-muted)" }}>// anonymous · read-only · public tables only:</div>
          <div>{"  "}<span style={{color:"var(--green)"}}>star_system</span> · <span style={{color:"var(--green)"}}>sector</span> · <span style={{color:"var(--green)"}}>star_system_object</span></div>
          <div>{"  "}<span style={{color:"var(--green)"}}>jump_gate</span> · <span style={{color:"var(--green)"}}>station</span> · <span style={{color:"var(--green)"}}>faction</span></div>
          <div style={{ marginTop: 8, color: "var(--fg-muted)" }}>// rows come back positional + a schema →</div>
          <div style={{ color: "var(--fg-muted)" }}>// reshape to objects, normalise x/y, draw.</div>
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
