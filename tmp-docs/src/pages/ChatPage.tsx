import { useEffect, useMemo, useRef, useState } from 'react';
import { useSpacetimeDB, useTable } from 'spacetimedb/react';
import { tables, type DbConnection } from '../module_bindings';
import { useAccount } from '../App';
import { login } from '../auth';
import { RegisterPanel } from '../components/RegisterPanel';
import { CONFIG } from '../config';

type Line = {
  key: string;
  at: Date;
  from: string;
  fromColor: 'server' | 'pilot' | 'self';
  body: string;
};

/**
 * Galaxy comms: the global player chat channel plus official server bulletins,
 * merged chronologically. Reading/writing chat requires a registered pilot —
 * the server's `my_galaxy_chat` view is empty for anonymous identities.
 */
export function ChatPage() {
  const { auth, loggedIn, player, connected } = useAccount();
  const conn = useSpacetimeDB();

  const registered = Boolean(player);
  const [galaxyMessages] = useTable(tables.myGalaxyChat, {
    enabled: registered,
  });
  const [serverMessages] = useTable(tables.serverChannelMessage);
  const [players] = useTable(tables.player);

  const lines = useMemo<Line[]>(() => {
    const nameOf = (idHex: string) =>
      players.find(p => p.id.toHexString() === idHex)?.username ??
      idHex.slice(0, 8);

    const chat: Line[] = galaxyMessages.map(m => {
      const sender = m.sender.tag === 'Player' ? m.sender.value : undefined;
      const self =
        sender !== undefined && conn.identity !== undefined
          ? sender.isEqual(conn.identity)
          : false;
      return {
        key: `g${m.id}`,
        at: m.createdAt.toDate(),
        from: sender ? nameOf(sender.toHexString()) : 'SYSTEM',
        fromColor: self ? 'self' : sender ? 'pilot' : 'server',
        body: m.body,
      };
    });

    const bulletins: Line[] = serverMessages.map(m => ({
      key: `s${m.id}`,
      at: m.createdAt.toDate(),
      from: 'RELAY',
      fromColor: 'server',
      body: m.body,
    }));

    return [...chat, ...bulletins].sort(
      (a, b) => a.at.getTime() - b.at.getTime()
    );
  }, [galaxyMessages, serverMessages, players, conn.identity]);

  // Pin scroll to the newest line.
  const logRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    logRef.current?.scrollTo(0, logRef.current.scrollHeight);
  }, [lines.length]);

  const [draft, setDraft] = useState('');
  const [sendError, setSendError] = useState<string | undefined>();

  async function send() {
    const message = draft.trim();
    if (!message) return;
    const connection = conn.getConnection() as DbConnection | null;
    if (!connection) return;
    setDraft('');
    setSendError(undefined);
    try {
      await connection.reducers.sendGalaxyChat({ message });
    } catch (e) {
      setSendError(e instanceof Error ? e.message : String(e));
      setDraft(message); // give the text back
    }
  }

  return (
    <main className="container chat-page">
      <div className="map-head">
        <div>
          <div className="kicker">▸ GALAXY CHANNEL · ALL REGISTERED PILOTS</div>
          <h1>Comms</h1>
        </div>
        <div className="map-status">
          {connected ? (
            <span className="tag accent">◉ LIVE</span>
          ) : (
            <span className="tag warn">◌ NO LINK</span>
          )}
        </div>
      </div>

      <div className="chat-frame panel">
        <div className="chat-log" ref={logRef}>
          {lines.length === 0 && (
            <p className="dim chat-empty">
              {registered
                ? 'Channel is quiet. Say something.'
                : 'Bulletins appear here; pilot chatter unlocks once you are registered.'}
            </p>
          )}
          {lines.map(l => (
            <div key={l.key} className={`chat-line ${l.fromColor}`}>
              <span className="chat-time">
                {l.at.toLocaleTimeString([], {
                  hour: '2-digit',
                  minute: '2-digit',
                })}
              </span>
              <span className="chat-from">{l.from}</span>
              <span className="chat-body">{l.body}</span>
            </div>
          ))}
        </div>

        {registered ? (
          <div className="chat-input-row">
            <input
              type="text"
              value={draft}
              maxLength={500}
              placeholder={`Transmit as ${player!.username}…`}
              onChange={e => setDraft(e.target.value)}
              onKeyDown={e => {
                if (e.key === 'Enter') void send();
              }}
            />
            <button className="btn primary" onClick={() => void send()}>
              SEND
            </button>
          </div>
        ) : loggedIn ? (
          <div className="chat-gate">
            <RegisterPanel />
          </div>
        ) : (
          <div className="chat-gate">
            <p>
              Galaxy comms are for registered pilots. Log in with the same
              account you use in the{' '}
              <a href={CONFIG.itchUrl} target="_blank" rel="noopener">
                game client
              </a>{' '}
              — or sign up and claim your callsign right here.
            </p>
            <button
              className="btn primary"
              onClick={() => auth && login(auth)}
              disabled={!auth}
            >
              LOG IN / SIGN UP
            </button>
          </div>
        )}
        {sendError && <p className="error chat-error">⚠ {sendError}</p>}
      </div>
    </main>
  );
}
