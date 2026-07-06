import { useTable } from 'spacetimedb/react';
import { tables } from '../module_bindings';
import { useAccount } from '../App';
import { login } from '../auth';
import { RegisterPanel } from '../components/RegisterPanel';
import { CONFIG } from '../config';

export function SplashPage() {
  const { auth, connected, loggedIn, player } = useAccount();
  const [players] = useTable(tables.player);
  const pilotsOnline = players.filter(p => p.loggedIn).length;

  return (
    <main className="container">
      <section className="hero">
        <div className="hero-copy">
          <div className="kicker">▸ PRE-ALPHA · PERSISTENT · CO-OP</div>
          <img
            className="hero-logo"
            src="./assets/solarance-logo.png"
            alt="Solarance: Beginnings"
          />
          <p className="pitch">
            A cozy, persistent, co-op space sandbox.
            <br />
            For pilots who loved EVE's economy but quit after the gank.
            <br />
            For people who play Stardew on the couch and Factorio at midnight.
            <br />A place to{' '}
            <em>contribute to something bigger than yourself in the time you
            have</em>.
          </p>

          <div className="cta-row">
            <a
              className="btn primary"
              href={CONFIG.itchUrl}
              target="_blank"
              rel="noopener"
            >
              ▸ DOWNLOAD ON ITCH.IO
            </a>
            <a className="btn" href="#/manifesto">
              READ THE MANIFESTO
            </a>
            <a className="btn ghost" href="#/galaxy">
              GALAXY MAP
            </a>
          </div>

          <div className="tag-row">
            {connected ? (
              <span className="tag accent">
                ◉ LIVE — {pilotsOnline} pilot{pilotsOnline === 1 ? '' : 's'} in
                the black right now
              </span>
            ) : (
              <span className="tag warn">◌ live link offline</span>
            )}
            <span className="tag dim">1 system · 10 sectors · 2 factions</span>
            <span className="tag dim">solo dev · evenings · no hype</span>
          </div>
        </div>

        <figure className="hero-shot">
          <img
            src="./assets/screen-01-corvette.png"
            alt="Corvette beside an asteroid, jumpgate above — in-engine screenshot"
          />
          <figcaption>
            in-engine · pre-alpha — corvette, low-grade asteroid, jumpgate
          </figcaption>
        </figure>
      </section>

      {loggedIn && !player && (
        <section className="splash-register">
          <RegisterPanel />
        </section>
      )}

      <section className="three-up">
        <div className="card">
          <div className="kicker">// what this site is</div>
          <h3>A relay station.</h3>
          <p>
            Log in, claim a callsign, talk on galaxy comms, and watch the maps
            tick — all live from the same SpacetimeDB module the game runs on.
            The cockpit itself is the{' '}
            <a href={CONFIG.itchUrl} target="_blank" rel="noopener">
              downloadable client
            </a>
            .
          </p>
        </div>
        <div className="card">
          <div className="kicker">// the loop</div>
          <h3>Find → extract → haul → contribute.</h3>
          <p>
            Mine hand-placed asteroid fields, haul ore to a half-built station,
            deposit it, and watch the station grow — with other pilots' names
            in the contribution log next to yours.
          </p>
        </div>
        <div className="card">
          <div className="kicker">// the promise</div>
          <h3>Your progress waits for you.</h3>
          <p>
            Sessions are twenty minutes, sometimes a Saturday, sometimes
            nothing for a week. No login streaks, no daily quests, no FOMO. The
            galaxy ticks while you sleep — it doesn't punish you for sleeping.
          </p>
        </div>
      </section>

      {!loggedIn && (
        <section className="login-strip">
          <p>
            Already flying? <b>Log in with the same account you use in-game</b>{' '}
            to chat from the website — or register your callsign here before
            your first flight.
          </p>
          <button
            className="btn primary"
            onClick={() => auth && login(auth)}
            disabled={!auth}
          >
            LOG IN / SIGN UP
          </button>
        </section>
      )}
    </main>
  );
}
