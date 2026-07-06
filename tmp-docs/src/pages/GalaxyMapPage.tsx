import { useSpacetimeDB, useTable } from 'spacetimedb/react';
import { tables } from '../module_bindings';
import {
  factionColor,
  factionName,
  fitViewBox,
  spectralColor,
} from '../mapTheme';

/**
 * Galactic chart — every known star system, positioned by its map
 * coordinates, colored by spectral class, ringed by controlling faction.
 */
export function GalaxyMapPage() {
  const conn = useSpacetimeDB();
  const [systems, systemsReady] = useTable(tables.starSystem);
  const [factions] = useTable(tables.faction);

  const vb = fitViewBox(
    systems.map(s => ({ x: s.mapCoordinates.x, y: s.mapCoordinates.y }))
  );

  return (
    <main className="container map-page">
      <div className="map-head">
        <div>
          <div className="kicker">▸ GALACTIC CHART · LIVE FEED</div>
          <h1>Galaxy Map</h1>
        </div>
        <div className="map-status">
          {conn.isActive ? (
            <span className="tag accent">◉ LIVE — {systems.length} charted
            system{systems.length === 1 ? '' : 's'}</span>
          ) : (
            <span className="tag warn">◌ NO LINK — awaiting telemetry</span>
          )}
        </div>
      </div>

      <div className="map-frame">
        <svg
          viewBox={`${vb.minX} ${vb.minY} ${vb.size} ${vb.size}`}
          className="map-svg"
          role="img"
          aria-label="Galaxy map of charted star systems"
        >
          <defs>
            <radialGradient id="starGlow" r="50%">
              <stop offset="0%" stopColor="white" stopOpacity="0.9" />
              <stop offset="100%" stopColor="white" stopOpacity="0" />
            </radialGradient>
          </defs>

          {/* subtle grid */}
          <GridLines vb={vb} />

          {systems.map(s => {
            const r = vb.size * 0.02;
            const color = spectralColor(s.spectral.tag);
            const ring = factionColor(s.controllingFactionId);
            return (
              <a key={s.id} href={`#/system/${s.id}`}>
                <g className="system-node">
                  <circle
                    cx={s.mapCoordinates.x}
                    cy={s.mapCoordinates.y}
                    r={r * 2.6}
                    fill="url(#starGlow)"
                    opacity={0.35}
                  />
                  <circle
                    cx={s.mapCoordinates.x}
                    cy={s.mapCoordinates.y}
                    r={r * 1.8}
                    fill="none"
                    stroke={ring}
                    strokeWidth={vb.size * 0.003}
                    strokeDasharray={`${vb.size * 0.012} ${vb.size * 0.008}`}
                  />
                  <circle
                    cx={s.mapCoordinates.x}
                    cy={s.mapCoordinates.y}
                    r={r}
                    fill={color}
                  />
                  <text
                    x={s.mapCoordinates.x}
                    y={s.mapCoordinates.y + r * 3.4}
                    textAnchor="middle"
                    className="map-label"
                    fontSize={vb.size * 0.035}
                  >
                    {s.name.toUpperCase()}
                  </text>
                  <text
                    x={s.mapCoordinates.x}
                    y={s.mapCoordinates.y + r * 3.4 + vb.size * 0.032}
                    textAnchor="middle"
                    className="map-sublabel"
                    fontSize={vb.size * 0.022}
                  >
                    {s.spectral.tag}-class ·{' '}
                    {factionName(factions, s.controllingFactionId)}
                  </text>
                </g>
              </a>
            );
          })}

          {systemsReady && systems.length === 0 && (
            <text
              x={vb.minX + vb.size / 2}
              y={vb.minY + vb.size / 2}
              textAnchor="middle"
              className="map-sublabel"
              fontSize={vb.size * 0.04}
            >
              NO CHARTED SYSTEMS
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

      <p className="dim map-footnote">
        ▸ Click a system to open its sector chart. Ring color = controlling
        faction · star color = spectral class. This is the live database — the
        same one the game clients fly in.
      </p>
    </main>
  );
}

function GridLines({
  vb,
}: {
  vb: { minX: number; minY: number; size: number };
}) {
  const lines = [];
  const step = vb.size / 8;
  for (let i = 0; i <= 8; i++) {
    lines.push(
      <line
        key={`v${i}`}
        x1={vb.minX + i * step}
        y1={vb.minY}
        x2={vb.minX + i * step}
        y2={vb.minY + vb.size}
        className="map-grid"
        strokeWidth={vb.size * 0.0008}
      />,
      <line
        key={`h${i}`}
        x1={vb.minX}
        y1={vb.minY + i * step}
        x2={vb.minX + vb.size}
        y2={vb.minY + i * step}
        className="map-grid"
        strokeWidth={vb.size * 0.0008}
      />
    );
  }
  return <g>{lines}</g>;
}
