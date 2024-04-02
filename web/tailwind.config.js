import colors from 'tailwindcss/colors';
import { fontFamily } from 'tailwindcss/defaultTheme';
import svelteUx from 'svelte-ux/plugins/tailwind.cjs';

/** @type {import('tailwindcss').Config} */
const config = {
  darkMode: ['class'],
  content: ['./src/**/*.{html,js,svelte,ts}', './node_modules/svelte-ux/**/*.{svelte,js}'],
  plugins: [svelteUx],
  safelist: ['dark'],
  theme: {
    extend: {
      fontFamily: {
        sans: [...fontFamily.sans],
      },
    },
  },
  ux: {
    themes: {
      light: {
        primary: 'hsl(31.1538 83.2% 50.9804%)',
        secondary: 'hsl(135.2727 35.9477% 70%)',
        accent: 'hsl(188.7379 94.4954% 42.7451%)',
        'surface-100': 'hsl(30 3.3333% 88.2353%)',
        'surface-200': 'hsl(20 3.7975% 84.5098%)',
        'surface-300': 'hsl(20 3.0303% 80.5882%)',
      },
      dark: {
        primary: 'hsl(31.0909 88% 50.9804%)',
        secondary: 'hsl(135.2727 35.9477% 70%)',
        accent: 'hsl(188.7379 94.4954% 42.7451%)',
        'surface-100': 'hsl(24 11.9048% 16.4706%)',
        'surface-200': 'hsl(26.6667 13.4328% 13.1373%)',
        'surface-300': 'hsl(25.7143 12.7273% 10.7843%)',
      },
    },
  },
};

export default config;
