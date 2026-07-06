import React from 'react';
import ReactDOM from 'react-dom/client';
import { SpacetimeDBProvider } from 'spacetimedb/react';
import { DbConnection } from './module_bindings';
import { CONFIG } from './config';
import { initAuth, type AuthState } from './auth';
import { App, AuthContext } from './App';
import './styles.css';

// Auth must resolve before the SpacetimeDB connection is built: the token is
// baked into the connection builder, and login/logout are full-page redirects,
// so one bootstrap per page load is all we ever need.
async function bootstrap() {
  let auth: AuthState | undefined;
  try {
    auth = await initAuth();
  } catch (e) {
    // Auth0 unreachable (offline dev, misconfigured tenant…) — the site still
    // works read-only over an anonymous SpacetimeDB connection.
    console.error('Auth0 init failed — continuing anonymously:', e);
  }

  const builder = DbConnection.builder()
    .withUri(CONFIG.spacetimeHost)
    .withDatabaseName(CONFIG.spacetimeDb)
    .withToken(auth?.idToken)
    .onConnect((_conn, identity) => {
      console.log('SpacetimeDB link established:', identity.toHexString());
    })
    .onConnectError((_ctx, err) => {
      console.warn('SpacetimeDB connection error:', err.message);
    });

  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      <AuthContext.Provider value={auth ?? null}>
        <SpacetimeDBProvider connectionBuilder={builder}>
          <App />
        </SpacetimeDBProvider>
      </AuthContext.Provider>
    </React.StrictMode>
  );
}

void bootstrap();
