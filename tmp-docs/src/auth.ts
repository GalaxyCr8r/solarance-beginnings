// Auth0 SPA login, kept deliberately tiny.
//
// SpacetimeDB derives a client's Identity from the OIDC issuer + subject of
// the JWT it connects with. The native game client logs in against the same
// Auth0 tenant and passes the ID token to SpacetimeDB — we do exactly the
// same here, so web login == in-game identity, and a username registered on
// the website is the same pilot in the game.
//
// The whole flow is redirect-based: login and logout both reload the page,
// which lets us build the SpacetimeDB connection exactly once at bootstrap
// with whatever token we have (or none, for anonymous browsing).

import { createAuth0Client, type Auth0Client } from '@auth0/auth0-spa-js';
import { CONFIG } from './config';

/** sessionStorage key set right after a login redirect completes. */
export const JUST_LOGGED_IN_KEY = 'sb.just-logged-in';

export type AuthState = {
  client: Auth0Client;
  /** Raw OIDC ID token to hand to SpacetimeDB, or undefined when anonymous. */
  idToken: string | undefined;
  /** Display info from the OIDC profile (email/name), if logged in. */
  profileLabel: string | undefined;
};

export async function initAuth(): Promise<AuthState> {
  const client = await createAuth0Client({
    domain: CONFIG.auth0Domain,
    clientId: CONFIG.auth0ClientId,
    cacheLocation: 'localstorage',
    authorizationParams: {
      redirect_uri: window.location.origin + window.location.pathname,
    },
  });

  // Returning from the Auth0 universal login page?
  const params = new URLSearchParams(window.location.search);
  if (params.has('code') && params.has('state')) {
    try {
      await client.handleRedirectCallback();
      sessionStorage.setItem(JUST_LOGGED_IN_KEY, '1');
    } catch (e) {
      console.error('Auth0 redirect callback failed:', e);
    }
    // Strip ?code=&state= but keep the hash route.
    window.history.replaceState(
      {},
      '',
      window.location.pathname + window.location.hash
    );
  }

  let idToken: string | undefined;
  let profileLabel: string | undefined;
  if (await client.isAuthenticated()) {
    const claims = await client.getIdTokenClaims();
    idToken = claims?.__raw;
    profileLabel = claims?.email ?? claims?.name ?? claims?.sub;
  }

  return { client, idToken, profileLabel };
}

export function login(auth: AuthState): void {
  void auth.client.loginWithRedirect();
}

export function logout(auth: AuthState): void {
  void auth.client.logout({
    logoutParams: {
      returnTo: window.location.origin + window.location.pathname,
    },
  });
}
