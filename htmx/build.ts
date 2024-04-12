#!/usr/bin/env bun
import { $ } from 'bun';
import { parseArgs } from 'util';

const args = parseArgs({
  options: {
    dev: {
      type: 'boolean',
    },
  },
});

await $`rm -rf build/*`.nothrow().quiet();

await $`vite build`;

// Alter the manifest to better reflect what things feel like at runtime.
// This way you can reference "index.js" instead of "src/index.ts".
// let manifest = await Bun.file('build/.vite/manifest.json').json();
// manifest = Object.entries(manifest).reduce((acc: Record<string, unknown>, [key, value]) => {
//   const newKey = key.replace('src/', '').replace(/\.ts$/, '.js');
//   acc[newKey] = value;
//   return acc;
// }, {});
// await Bun.write('build/.vite/manifest.json', JSON.stringify(manifest, null, 2));

if (args.values.dev) {
  process.exit(0);
}

let glob = new Bun.Glob('build/**/*.{js,css}');
let files = Array.from(glob.scanSync());

console.log('Compressing assets...');
await Promise.all(
  files.map(async (path) => {
    let file = await Bun.file(path).text();
    let zipped = Bun.gzipSync(file, { level: 9 });
    await Bun.write(path + '.gz', zipped);
    await $`brotli -s ${path}`;
    console.log(`Compressed ${path}`);
  })
);
