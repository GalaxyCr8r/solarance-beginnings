/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_SPACETIME_HOST?: string;
  readonly VITE_SPACETIME_DB?: string;
  readonly VITE_AUTH0_DOMAIN?: string;
  readonly VITE_AUTH0_CLIENT_ID?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
