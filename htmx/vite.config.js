import { defineConfig } from 'vite';

const production = process.env.NODE_ENV === 'production';
const env = production ? 'production' : 'development';

export default defineConfig({
  build: {
    outDir: 'build',
    assetsDir: 'static',
    copyPublicDir: true,
    manifest: true,
    minify: production,
    rollupOptions: {
      input: {
        index: './src/index.ts',
      },
      output: production
        ? {
            assetFileNames: '_app/immutable/[name]-[hash].[extname]',
            chunkFileNames: '_app/immutable/[name]-[hash].js',
            entryFileNames: '_app/immutable/[name]-[hash].js',
          }
        : undefined,
    },
  },
  define: {
    'process.env.ENV': env,
  },
});
