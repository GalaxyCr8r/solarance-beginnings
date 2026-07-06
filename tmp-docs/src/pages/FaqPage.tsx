import { CONFIG } from '../config';

const FAQS: { q: string; a: React.ReactNode }[] = [
  {
    q: 'What is Solarance: Beginnings?',
    a: (
      <>
        A cozy, persistent, co-op space sandbox MMO. One shared galaxy that
        keeps ticking while you're away. The MVP loop is: find asteroid fields,
        mine, haul the ore to a half-built station, contribute it, and watch
        the station grow — together with other pilots.
      </>
    ),
  },
  {
    q: 'Can I play it right now?',
    a: (
      <>
        It's pre-alpha, but yes — the latest client is on{' '}
        <a href={CONFIG.itchUrl} target="_blank" rel="noopener">
          itch.io
        </a>
        . Expect rough edges, honest changelogs, and a small galaxy. A Steam
        page will exist <em>someday</em>; it doesn't yet.
      </>
    ),
  },
  {
    q: 'Can I play in the browser?',
    a: (
      <>
        No — this website is a <em>relay station</em>, not a cockpit. You can
        log in, register your callsign, chat on galaxy comms, and watch the
        live maps here. Flying happens in the downloadable native client
        (Rust, Windows/Linux first).
      </>
    ),
  },
  {
    q: 'Is there combat?',
    a: (
      <>
        Not in the MVP — by design, not by accident. If combat ever ships it
        will be opt-in and live in designated lawless space. Core sectors stay
        safe forever. This game is for the kind of player EVE traumatized and
        lost.
      </>
    ),
  },
  {
    q: 'What are the factions?',
    a: (
      <>
        Two are playable at MVP: the <b>Lrak Combine</b> (the pretenders to a
        throne — they rule from the world they call humanity's cradle) and the{' '}
        <b>Rediar Federation</b> (the doubters with the evidence that it
        isn't). Five more exist in the lore and arrive when the systems that
        make a faction <em>matter</em> are built.
      </>
    ),
  },
  {
    q: 'How do accounts work?',
    a: (
      <>
        Login is via Auth0 — the same account works on this website and in the
        game client, because both resolve to the same SpacetimeDB identity.
        Register your callsign once (here or in-game) and it's yours in both
        places.
      </>
    ),
  },
  {
    q: 'What is the game built on?',
    a: (
      <>
        The server is a Rust module running on{' '}
        <a href="https://spacetimedb.com" target="_blank" rel="noopener">
          SpacetimeDB
        </a>
        ; the client is native Rust. This website subscribes to the same
        public tables the game uses — the maps and chat you see here are the
        live database, not a mirror.
      </>
    ),
  },
  {
    q: 'How much does it cost?',
    a: (
      <>
        Nothing right now. It's a pre-alpha passion project built solo, in
        evenings, after the kids go down. If that ever changes, the manifesto
        gets updated first.
      </>
    ),
  },
  {
    q: 'Where do I follow development?',
    a: (
      <>
        One devlog post a month, a small{' '}
        <a href={CONFIG.discordUrl} target="_blank" rel="noopener">
          Discord
        </a>
        , and the{' '}
        <a href={CONFIG.githubUrl} target="_blank" rel="noopener">
          GitHub repo
        </a>
        . No hype cycle. The bar is whether we're still here in 18 months.
      </>
    ),
  },
];

export function FaqPage() {
  return (
    <main className="container prose">
      <div className="kicker">▸ TRANSMISSION LOG · FREQUENTLY ASKED</div>
      <h1>FAQ</h1>
      <div className="faq-list">
        {FAQS.map((f, i) => (
          <details key={i} className="faq-item" open={i === 0}>
            <summary>
              <span className="faq-num">{String(i + 1).padStart(2, '0')}</span>
              {f.q}
            </summary>
            <div className="faq-answer">{f.a}</div>
          </details>
        ))}
      </div>
    </main>
  );
}
