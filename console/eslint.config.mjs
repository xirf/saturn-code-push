import withNuxt from './.nuxt/eslint.config.mjs'

export default withNuxt(
  {
    ignores: ['.output/**', 'dist/**', 'src/**'],
    rules: {
      'vue/multi-word-component-names': 'off',
      'vue/html-self-closing': [
        'error',
        { html: { void: 'always', normal: 'always', component: 'always' } },
      ],
    },
  },
  {
    files: ['app/components/ui/**/*.vue'],
    rules: { 'vue/require-default-prop': 'off' },
  },
)
