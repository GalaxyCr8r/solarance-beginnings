// Site configuration. Every value can be overridden at build time with a
// Vite env var (`VITE_…`) so deployments don't require code edits.

export const CONFIG = {
  /** SpacetimeDB host the site connects to. */
  spacetimeHost:
    import.meta.env.VITE_SPACETIME_HOST ?? 'https://maincloud.spacetimedb.com',

  /** Database name chosen at `spacetime publish`. */
  spacetimeDb: import.meta.env.VITE_SPACETIME_DB ?? 'solarance-beginnings',

  /**
   * Auth0 tenant. These defaults are the same tenant the native game client
   * uses (client/.env.template) so a login here resolves to the SAME
   * SpacetimeDB identity as in-game. The Auth0 application must allow this
   * site's origin as a callback/logout/web origin — see README.md.
   */
  auth0Domain:
    import.meta.env.VITE_AUTH0_DOMAIN ?? 'dev-k6zdm2f3z3kst6r7.us.auth0.com',
  auth0ClientId:
    import.meta.env.VITE_AUTH0_CLIENT_ID ?? 'BnJiVrOXavZ1mbvsiwvBcZ96dTFH9k4L',

  /** Where players actually get the game. */
  itchUrl: 'https://galaxycr8r.itch.io/solarance-beginnings',

  discordUrl: 'https://discord.solarance-beginnings.com',
  githubUrl: 'https://github.com/GalaxyCr8r/solarance-beginnings',
} as const;
