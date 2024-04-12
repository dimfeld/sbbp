import * as fs from 'fs';
import { defineConfig } from 'vite';

const production = process.env.NODE_ENV === 'production';
const env = production ? 'production' : 'development';

const enableLiveReload = !production || process.env.LIVE_RELOAD === 'true';

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
    'process.env.LIVE_RELOAD': enableLiveReload,
  },
  plugins: [
    {
      name: 'modify-manifest-keys',
      apply: 'build',
      configResolved(config) {
        if (config.build.manifest) {
          config.plugins.push({
            name: 'modify-manifest-keys-plugin',
            apply: 'build',
            writeBundle() {
              const manifestPath = `${config.build.outDir}/.vite/manifest.json`;
              const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
              const modifiedManifest = {};

              for (const key in manifest) {
                // Remove "src/" prefix and change ts to js
                const newKey = key.replace(/^src\//, '').replace(/\.ts$/, '.js');
                modifiedManifest[newKey] = manifest[key];
              }

              fs.writeFileSync(manifestPath, JSON.stringify(modifiedManifest, null, 2));
            },
          });
        }
      },
    },
  ],
});
