import { defineConfig } from 'vite';

const production = process.env.NODE_ENV === 'production';
const env = production ? 'production' : 'development';

export default defineConfig({
  build: {
    outDir: 'build',
    copyPublicDir: true,
    manifest: true,
    minify: production,
    rollupOptions: {
      output: production
        ? {
            assetFileNames: '[name]-[hash].[extname]',
            chunkFileNames: '[name]-[hash].js',
            entryFileNames: '[name]-[hash].js',
          }
        : undefined,
    },
    lib: {
      formats: ['es'],
      entry: ['src/index.ts'],
    },
  },
  define: {
    'process.env.ENV': env,
  },
});
