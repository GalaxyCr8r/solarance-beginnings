import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// `base: './'` keeps the built site relocatable (GitHub Pages subpath, custom
// domain, or local file preview) — the same property the old no-build site had.
export default defineConfig({
  base: './',
  plugins: [react()],
});
