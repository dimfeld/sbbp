import daisyui from 'daisyui';
import { fontFamily } from 'tailwindcss/defaultTheme';

/** @type {import('tailwindcss').Config} */
const config = {
  darkMode: ['class'],
  content: ['./src/**/*.{html,js,svelte,ts}', '../api/src/pages/**/*.{rs,js,html,ts}'],
  plugins: [daisyui],
  safelist: ['dark'],
  theme: {
    extend: {
      fontFamily: {
        sans: [...fontFamily.sans],
      },
    },
  },
};

export default config;
