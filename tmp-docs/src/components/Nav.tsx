import { useSpacetimeDB } from 'spacetimedb/react';
import { useAccount, type Route } from '../App';
import { login, logout } from '../auth';

const LINKS: { hash: string; label: string; page: Route['page'] }[] = [
  { hash: '#/', label: 'HOME', page: 'splash' },
  { hash: '#/manifesto', label: 'MANIFESTO', page: 'manifesto' },
  { hash: '#/faq', label: 'FAQ', page: 'faq' },
  { hash: '#/galaxy', label: 'GALAXY', page: 'galaxy' },
  { hash: '#/system', label: 'SYSTEM', page: 'system' },
  { hash: '#/chat', label: 'COMMS', page: 'chat' },
];

export function Nav({ route }: { route: Route }) {
  const { auth, loggedIn, player } = useAccount();
  const conn = useSpacetimeDB();

  return (
    <header className="nav">
      <a className="brand" href="#/">
        <img src="./assets/solarance-icon.png" alt="" />
        <span>
          SOLARANCE<i>::</i>BEGINNINGS
        </span>
      </a>

      <nav className="links">
        {LINKS.map(l => (
          <a
            key={l.hash}
            href={l.hash}
            className={route.page === l.page ? 'active' : ''}
          >
            {l.label}
          </a>
        ))}
      </nav>

      <div className="nav-right">
        <span
          className={`linkdot ${conn.isActive ? 'on' : 'off'}`}
          title={
            conn.isActive
              ? 'Live link to the galaxy established'
              : 'No live link — showing static pages only'
          }
        >
          {conn.isActive ? 'LINK' : 'NO LINK'}
        </span>

        {loggedIn ? (
          <>
            <span className="pilot-chip" title={auth?.profileLabel}>
              {player ? player.username : 'UNREGISTERED'}
            </span>
            <button
              className="btn ghost small"
              onClick={() => auth && logout(auth)}
            >
              LOG OUT
            </button>
          </>
        ) : (
          <button
            className="btn primary small"
            onClick={() => auth && login(auth)}
            disabled={!auth}
            title={auth ? 'Log in with Auth0' : 'Auth0 unavailable'}
          >
            LOG IN
          </button>
        )}
      </div>
    </header>
  );
}
