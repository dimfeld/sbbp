import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  envDir: '..',
  test: {
    testTimeout: 5000,
  },
  optimizeDeps: {
    exclude: ['filigree-web'],
  },
  ssr: {
    noExternal: ['filigree-web'],
  },
  resolve: {
    // Useful for development
    dedupe: ['svelte', '@sveltejs/kit'],
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:7823',
      },
    },
  },
});
