import tailwindcss from 'tailwindcss';
import autoprefixer from 'autoprefixer';

export default {
  plugins: [
    //Some plugins, like tailwindcss/nesting, need to run before Tailwind,
    tailwindcss('../tailwind.config.js'),
    //But others, like autoprefixer, need to run after,
    autoprefixer,
  ],
};

