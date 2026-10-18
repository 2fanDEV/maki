import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  cacheDir: '.vite-cache',
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: { '@': decodeURIComponent(new URL('./src', import.meta.url).pathname) },
  },
  server: { port: 4321 },
  preview: { port: 4321 },
  build: { outDir: 'build', emptyOutDir: false },
});
