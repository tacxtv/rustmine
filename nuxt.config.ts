// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
    telemetry: false,
    devtools: { enabled: true },
    ssr: false,
    pages: true,
    components: true,
    srcDir: 'src-nuxt/',
    css: ['~/assets/sass/global.sass'],
    modules: ['nuxt-quasar-ui'],
    quasar: {
        iconSet: 'mdi-v5',
        plugins: ['Notify', 'Dialog'],
        config: {
            dark: 'auto',
            notify: {
                timeout: 2500,
                position: 'top-right',
                actions: [{ icon: 'mdi-close', color: 'white' }],
            },
        },
    },
})
