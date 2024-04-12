import './app.postcss';
import { startLiveReload } from './livereload.js';
import htmx from 'htmx.org';
import Alpine from 'alpinejs';

window.Alpine = Alpine;
window.htmx = htmx;

if (process.env.LIVE_RELOAD) {
  startLiveReload();
}

Alpine.start();
