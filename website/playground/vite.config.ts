import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  base: '/rust-igraph/playground/',
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    rollupOptions: {
      output: {
        manualChunks: {
          'react-vendor': ['react', 'react-dom'],
          'codemirror': [
            '@codemirror/view',
            '@codemirror/state',
            '@codemirror/lang-rust',
            '@codemirror/theme-one-dark',
          ],
        },
      },
    },
  },
  worker: {
    format: 'es',
  },
});
