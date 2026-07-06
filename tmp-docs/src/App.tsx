import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
} from 'react';
import { useSpacetimeDB, useTable } from 'spacetimedb/react';
import { tables, type Player } from './module_bindings';
import { JUST_LOGGED_IN_KEY, type AuthState } from './auth';
import { Nav } from './components/Nav';
import { Footer } from './components/Footer';
import { DownloadDialog } from './components/DownloadDialog';
import { SplashPage } from './pages/SplashPage';
import { FaqPage } from './pages/FaqPage';
import { ManifestoPage } from './pages/ManifestoPage';
import { GalaxyMapPage } from './pages/GalaxyMapPage';
import { SystemMapPage } from './pages/SystemMapPage';
import { ChatPage } from './pages/ChatPage';

/** Auth0 state resolved at bootstrap; null when Auth0 was unreachable. */
export const AuthContext = createContext<AuthState | null>(null);
export const useAuth = () => useContext(AuthContext);

/**
 * Everything a page needs to know about "who is looking at this screen":
 * Auth0 login, SpacetimeDB link state, and the registered Player row (if any).
 */
export function useAccount(): {
  auth: AuthState | null;
  connected: boolean;
  loggedIn: boolean;
  player: Player | undefined;
  playersReady: boolean;
} {
  const auth = useAuth();
  const conn = useSpacetimeDB();
  const [players, playersReady] = useTable(tables.player);

  const player = useMemo(() => {
    if (!conn.identity) return undefined;
    return players.find(p => p.id.isEqual(conn.identity!));
  }, [players, conn.identity]);

  return {
    auth,
    connected: conn.isActive,
    loggedIn: Boolean(auth?.idToken),
    player,
    playersReady,
  };
}

// --- Tiny hash router -------------------------------------------------------

export type Route =
  | { page: 'splash' }
  | { page: 'faq' }
  | { page: 'manifesto' }
  | { page: 'galaxy' }
  | { page: 'system'; systemId?: number }
  | { page: 'chat' };

function parseRoute(hash: string): Route {
  const parts = hash.replace(/^#\/?/, '').split('/').filter(Boolean);
  switch (parts[0]) {
    case 'faq':
      return { page: 'faq' };
    case 'manifesto':
      return { page: 'manifesto' };
    case 'galaxy':
      return { page: 'galaxy' };
    case 'system': {
      const id = Number(parts[1]);
      return { page: 'system', systemId: Number.isFinite(id) ? id : undefined };
    }
    case 'chat':
      return { page: 'chat' };
    default:
      return { page: 'splash' };
  }
}

export function useRoute(): Route {
  const [route, setRoute] = useState<Route>(() =>
    parseRoute(window.location.hash)
  );
  useEffect(() => {
    const onHash = () => setRoute(parseRoute(window.location.hash));
    window.addEventListener('hashchange', onHash);
    return () => window.removeEventListener('hashchange', onHash);
  }, []);
  return route;
}

// --- App shell ---------------------------------------------------------------

export function App() {
  const route = useRoute();

  // The post-login "go download the client" dialog. The flag is set by the
  // auth redirect handler, so it shows exactly once per login.
  const [showDownloadDialog, setShowDownloadDialog] = useState(
    () => sessionStorage.getItem(JUST_LOGGED_IN_KEY) === '1'
  );
  const dismissDownloadDialog = () => {
    sessionStorage.removeItem(JUST_LOGGED_IN_KEY);
    setShowDownloadDialog(false);
  };

  // Scroll to top on page change (hash routers don't do this for you).
  useEffect(() => {
    window.scrollTo(0, 0);
  }, [route.page]);

  let page: React.ReactNode;
  switch (route.page) {
    case 'faq':
      page = <FaqPage />;
      break;
    case 'manifesto':
      page = <ManifestoPage />;
      break;
    case 'galaxy':
      page = <GalaxyMapPage />;
      break;
    case 'system':
      page = <SystemMapPage systemId={route.systemId} />;
      break;
    case 'chat':
      page = <ChatPage />;
      break;
    default:
      page = <SplashPage />;
  }

  return (
    <div className="shell">
      <div className="starfield" aria-hidden="true" />
      <Nav route={route} />
      <div className="page">{page}</div>
      <Footer />
      {showDownloadDialog && <DownloadDialog onOk={dismissDownloadDialog} />}
    </div>
  );
}
