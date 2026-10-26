import tailwindcss from '@tailwindcss/vite'

export default defineNuxtConfig({
  compatibilityDate: '2026-10-27',
  ssr: false,
  experimental: { entryImportMap: false },
  devtools: { enabled: false },
  devServer: { port: 3001, https: false },
  modules: ['shadcn-nuxt', '@pinia/nuxt', '@pinia/colada-nuxt', '@nuxt/eslint'],
  shadcn: { prefix: '', componentDir: '~/components/ui' },
  vite: { plugins: [tailwindcss()], build: { assetsInlineLimit: 0 } },
  css: ['~/assets/css/main.css'],
  app: {
    head: {
      title: 'Saturn - Release console',
      htmlAttrs: { lang: 'en', class: 'dark' },
      meta: [{ name: 'description', content: 'Your self-hosted release and patch console.' }],
      link: [{ rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }],
    },
  },
  nitro: {
    prerender: { routes: ['/'], crawlLinks: false },
    devProxy: Object.fromEntries(
      ['/auth', '/admin', '/api'].map((prefix) => [
        prefix,
        {
          target: `${process.env.SATURN_API_ORIGIN || 'http://host.docker.internal:3000'}${prefix}`,
          changeOrigin: false,
        },
      ]),
    ),
  },
  typescript: { strict: true, typeCheck: false },
})
