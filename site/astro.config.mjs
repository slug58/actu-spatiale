// @ts-check
import { defineConfig, passthroughImageService } from 'astro/config';

// SITE_URL et BASE_PATH sont fournis par GitHub Actions (actions/configure-pages).
// En local : http://localhost:4321/
export default defineConfig({
  site: process.env.SITE_URL || 'http://localhost:4321',
  base: process.env.BASE_PATH || '/',
  // Toutes les images viennent des API (URLs externes) : pas besoin de sharp.
  image: { service: passthroughImageService() },
});
