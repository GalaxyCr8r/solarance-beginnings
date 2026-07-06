// Shared visual vocabulary for the galaxy + system maps.

/** Faction id → display color. Ids from server/src/definitions/factions.rs. */
export function factionColor(factionId: number): string {
  switch (factionId) {
    case 1: // Lrak Combine
      return '#e0524d';
    case 2: // Independent Worlds Alliance
      return '#4dbf9e';
    case 3: // Free Trade Union
      return '#d9983f';
    case 4: // Rediar Federation
      return '#5a8ff0';
    case 5: // Vancellan
      return '#9b6fe0';
    case 10: // Alliance of Procyon
      return '#e8c66a';
    default: // Factionless / unknown
      return '#8a93a6';
  }
}

export function factionName(
  factions: readonly { id: number; name: string }[],
  factionId: number
): string {
  return factions.find(f => f.id === factionId)?.name ?? 'Unclaimed';
}

/** Spectral class → star color (O hottest/bluest … M coolest/reddest). */
export function spectralColor(tag: string): string {
  switch (tag) {
    case 'O':
      return '#9db4ff';
    case 'B':
      return '#b8c8ff';
    case 'A':
      return '#e6ecff';
    case 'F':
      return '#fff4e0';
    case 'G':
      return '#ffd98a';
    case 'K':
      return '#ffb46b';
    case 'M':
      return '#ff7a5c';
    default:
      return '#ffd98a';
  }
}

/** Fit a set of points into a square viewBox with padding. */
export function fitViewBox(
  points: { x: number; y: number }[],
  pad = 0.18
): { minX: number; minY: number; size: number } {
  if (points.length === 0) return { minX: -50, minY: -50, size: 100 };
  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;
  for (const p of points) {
    minX = Math.min(minX, p.x);
    minY = Math.min(minY, p.y);
    maxX = Math.max(maxX, p.x);
    maxY = Math.max(maxY, p.y);
  }
  const w = maxX - minX || 1;
  const h = maxY - minY || 1;
  const size = Math.max(w, h) * (1 + pad * 2);
  const cx = (minX + maxX) / 2;
  const cy = (minY + maxY) / 2;
  return { minX: cx - size / 2, minY: cy - size / 2, size };
}
