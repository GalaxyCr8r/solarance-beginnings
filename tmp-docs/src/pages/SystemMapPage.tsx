import { useMemo, useState } from 'react';
import { useSpacetimeDB, useTable } from 'spacetimedb/react';
import { tables, type Sector } from '../module_bindings';
import { factionColor, factionName, fitViewBox } from '../mapTheme';

/**
 * Sector chart for one star system: sectors positioned by their in-system
 * coordinates, jumpgate lanes drawn between them, colored by controlling
 * faction. Click a sector for its survey data.
 */
export function SystemMapPage({ systemId }: { systemId?: number }) {
  const conn = useSpacetimeDB();
  const [systems] = useTable(tables.starSystem);
  const [sectors, sectorsReady] = useTable(tables.sector);
  const [gates] = useTable(tables.jumpGate);
  const [factions] = useTable(tables.faction);

  // Default to the first charted system when none is picked.
  const activeSystem =
    systems.find(s => s.id === systemId) ?? systems[0] ?? undefined;

  const systemSectors = useMemo(
    () =>
      activeSystem
        ? sectors.filter(sec => sec.systemId === activeSystem.id)
        : [],
    [sectors, activeSystem]
  );

  const [selectedId, setSelectedId] = useState<bigint | undefined>(undefined);
  const selected = systemSectors.find(s => s.id === selectedId);

  // One lane per unordered sector pair (gates come in mirrored pairs).
  const lanes = useMemo(() => {
    const byId = new Map(systemSectors.map(s => [s.id, s]));
    const seen = new Set<string>();
    const out: { a: Sector; b: Sector; active: boolean }[] = [];
    for (const g of gates) {
      const a = byId.get(g.currentSectorId);
      const b = byId.get(g.targetSectorId);
      if (!a || !b) continue; // cross-system or unknown endpoint
      const key =
        g.currentSectorId < g.targetSectorId
          ? `${g.currentSectorId}-${g.targetSectorId}`
          : `${g.targetSectorId}-${g.currentSectorId}`;
      if (seen.has(key)) continue;
      seen.add(key);
      out.push({ a, b, active: g.isActive });
    }
    return out;
  }, [gates, systemSectors]);

  const vb = fitViewBox(systemSectors.map(s => ({ x: s.x, y: s.y })));

  return (
    <main className="container map-page">
      <div className="map-head">
        <div>
          <div className="kicker">
            ▸ SECTOR CHART{activeSystem ? ` · ${activeSystem.name.toUpperCase()}` : ''}
          </div>
          <h1>System Map</h1>
        </div>
        <div className="map-status">
          {systems.length > 1 && (
            <span className="system-picker">
              {systems.map(s => (
                <a
                  key={s.id}
                  href={`#/system/${s.id}`}
                  className={`tag ${activeSystem?.id === s.id ? 'accent' : 'dim'}`}
                >
                  {s.name}
                </a>
              ))}
            </span>
          )}
          {conn.isActive ? (
            <span className="tag accent">
              ◉ LIVE — {systemSectors.length} sector
              {systemSectors.length === 1 ? '' : 's'}
            </span>
          ) : (
            <span className="tag warn">◌ NO LINK — awaiting telemetry</span>
          )}
        </div>
      </div>

      <div className="map-layout">
        <div className="map-frame">
          <svg
            viewBox={`${vb.minX} ${vb.minY} ${vb.size} ${vb.size}`}
            className="map-svg"
            role="img"
            aria-label="Sector map with jumpgate lanes"
          >
            {/* jump lanes under the nodes */}
            {lanes.map((lane, i) => (
              <line
                key={i}
                x1={lane.a.x}
                y1={lane.a.y}
                x2={lane.b.x}
                y2={lane.b.y}
                className={lane.active ? 'lane' : 'lane inactive'}
                strokeWidth={vb.size * 0.0035}
              />
            ))}

            {systemSectors.map(sec => {
              const r = vb.size * 0.022;
              const color = factionColor(sec.controllingFactionId);
              const isSel = sec.id === selectedId;
              return (
                <g
                  key={String(sec.id)}
                  className="sector-node"
                  onClick={() =>
                    setSelectedId(isSel ? undefined : sec.id)
                  }
                >
                  <rect
                    x={sec.x - r}
                    y={sec.y - r}
                    width={r * 2}
                    height={r * 2}
                    fill="rgba(10,14,22,0.85)"
                    stroke={color}
                    strokeWidth={vb.size * (isSel ? 0.005 : 0.0028)}
                    transform={`rotate(45 ${sec.x} ${sec.y})`}
                  />
                  <circle cx={sec.x} cy={sec.y} r={r * 0.35} fill={color} />
                  <text
                    x={sec.x}
                    y={sec.y - r * 1.9}
                    textAnchor="middle"
                    className="map-label"
                    fontSize={vb.size * 0.03}
                  >
                    {sec.name.toUpperCase()}
                  </text>
                  <text
                    x={sec.x}
                    y={sec.y + r * 2.6}
                    textAnchor="middle"
                    className="map-sublabel"
                    fontSize={vb.size * 0.02}
                  >
                    SEC {sec.securityLevel}/10
                  </text>
                </g>
              );
            })}

            {sectorsReady && systemSectors.length === 0 && (
              <text
                x={vb.minX + vb.size / 2}
                y={vb.minY + vb.size / 2}
                textAnchor="middle"
                className="map-sublabel"
                fontSize={vb.size * 0.04}
              >
                NO SECTOR DATA
              </text>
            )}
          </svg>

          {!conn.isActive && (
            <div className="map-overlay">
              <p>
                ◌ No live link to the galaxy. The chart populates when the
                SpacetimeDB module is reachable.
              </p>
            </div>
          )}
        </div>

        <aside className="sector-detail panel">
          {selected ? (
            <>
              <div className="panel-head">
                ▸ SURVEY · {selected.name.toUpperCase()}
              </div>
              {selected.description && <p>{selected.description}</p>}
              <dl className="spec-list">
                <dt>Controlled by</dt>
                <dd style={{ color: factionColor(selected.controllingFactionId) }}>
                  {factionName(factions, selected.controllingFactionId)}
                </dd>
                <dt>Security</dt>
                <dd>{selected.securityLevel} / 10</dd>
                <dt>Sunlight</dt>
                <dd>
                  <Meter value={selected.sunlight} />
                </dd>
                <dt>Nebula</dt>
                <dd>
                  <Meter value={selected.nebula} />
                </dd>
                <dt>Anomalous</dt>
                <dd>
                  <Meter value={selected.anomalous} />
                </dd>
                <dt>Rare ore</dt>
                <dd>
                  <Meter value={selected.rareOre} />
                </dd>
              </dl>
            </>
          ) : (
            <>
              <div className="panel-head">▸ SURVEY</div>
              <p className="dim">
                Select a sector to read its survey data — controlling faction,
                security level, and resource potentials.
              </p>
            </>
          )}
        </aside>
      </div>

      <p className="dim map-footnote">
        ▸ Lanes are jumpgate connections. Diamond color = controlling faction.
        The cockpit view of these sectors lives in the game client.
      </p>
    </main>
  );
}

function Meter({ value }: { value: number }) {
  const pct = Math.round(Math.max(0, Math.min(1, value)) * 100);
  return (
    <span className="meter" title={`${pct}%`}>
      <span className="meter-fill" style={{ width: `${pct}%` }} />
    </span>
  );
}
