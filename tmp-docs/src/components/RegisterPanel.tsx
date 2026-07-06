import { useMemo, useState } from 'react';
import { useSpacetimeDB, useTable } from 'spacetimedb/react';
import { tables, type DbConnection } from '../module_bindings';

/**
 * Username + faction registration for a logged-in identity with no Player row.
 * Mirrors the server-side rules in `register_playername`: the chosen faction
 * must be joinable and hold a Capital station (new pilots spawn there).
 */
export function RegisterPanel() {
  const conn = useSpacetimeDB();
  const [factions, factionsReady] = useTable(tables.faction);
  const [username, setUsername] = useState('');
  const [factionId, setFactionId] = useState<number | undefined>(undefined);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | undefined>(undefined);

  const joinable = useMemo(
    () =>
      factions.filter(f => f.joinable && f.capitalStationId !== undefined),
    [factions]
  );

  const selected = joinable.find(f => f.id === factionId);

  async function submit() {
    const connection = conn.getConnection() as DbConnection | null;
    if (!connection || !conn.identity) {
      setError('No live link to the galaxy — try again in a moment.');
      return;
    }
    if (!username.trim()) {
      setError('Enter a callsign first.');
      return;
    }
    if (username.trim().length > 32) {
      setError('Callsigns are 32 characters max.');
      return;
    }
    if (factionId === undefined) {
      setError('Pick a faction — new pilots spawn at their Capital station.');
      return;
    }
    setBusy(true);
    setError(undefined);
    try {
      await connection.reducers.registerPlayername({
        identity: conn.identity,
        username: username.trim(),
        factionId,
      });
      // Success shows up as our Player row arriving via subscription;
      // useAccount() flips from "unregistered" automatically.
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="panel register">
      <div className="panel-head">▸ PILOT REGISTRATION</div>
      <p className="dim">
        This identity has no callsign yet. Register once, here or in the game
        client — it's the same account either way.
      </p>

      <label className="field">
        <span>CALLSIGN</span>
        <input
          type="text"
          maxLength={32}
          value={username}
          placeholder="e.g. cmdr_helga"
          onChange={e => setUsername(e.target.value)}
          onKeyDown={e => {
            if (e.key === 'Enter') void submit();
          }}
        />
      </label>

      <label className="field">
        <span>FACTION</span>
        {factionsReady && joinable.length === 0 ? (
          <span className="dim">
            No joinable factions found — the galaxy may still be initializing.
          </span>
        ) : (
          <div className="faction-picker">
            {joinable.map(f => (
              <button
                key={f.id}
                className={`faction-option ${factionId === f.id ? 'selected' : ''}`}
                onClick={() => setFactionId(f.id)}
                type="button"
              >
                <b>{f.name}</b>
                <span>[{f.shortName}]</span>
              </button>
            ))}
          </div>
        )}
      </label>

      {selected && <p className="dim faction-blurb">{selected.description}</p>}

      {error && <p className="error">⚠ {error}</p>}

      <button
        className="btn primary"
        onClick={() => void submit()}
        disabled={busy}
      >
        {busy ? 'TRANSMITTING…' : 'REGISTER'}
      </button>
    </div>
  );
}
