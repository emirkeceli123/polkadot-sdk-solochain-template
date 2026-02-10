import { defineConfig } from 'vite';
import { viteSingleFile } from 'vite-plugin-singlefile';

export default defineConfig({
  plugins: [viteSingleFile()],
  build: {
    outDir: '..',
    emptyOutDir: false,
    rollupOptions: {
      input: 'explorer.html',
    },
    target: 'es2020',
  },
});
