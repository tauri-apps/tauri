module.exports = {
  root: true,

  env: {
    node: true,
    jest: true
  },

  parser: '@typescript-eslint/parser',

  extends: [
    'standard-with-typescript',
    'plugin:@typescript-eslint/recommended-requiring-type-checking',
    'plugin:lodash-template/recommended',
    // TODO: make this work with typescript
    // 'plugin:node/recommended'
    'prettier',
    'prettier/@typescript-eslint'
  ],

  plugins: ['@typescript-eslint', 'node', 'security'],

  parserOptions: {
    tsconfigRootDir: __dirname,
    project: './tsconfig.json'
  },

  globals: {
    __statics: true,
    process: true
  },

  // add your custom rules here
  rules: {
    // allow console.log during development only
    'no-console': process.env.NODE_ENV === 'production' ? 'error' : 'off',
    // allow debugger during development only
    'no-debugger': process.env.NODE_ENV === 'production' ? 'error' : 'off',
    'no-process-exit': 'off',
    'security/detect-non-literal-fs-filename': 'warn',
    'security/detect-unsafe-regex': 'error',
    'security/detect-buffer-noassert': 'error',
    'security/detect-child-process': 'warn',
    'security/detect-disable-mustache-escape': 'error',
    'security/detect-eval-with-expression': 'error',
    'security/detect-no-csrf-before-method-override': 'error',
    'security/detect-non-literal-regexp': 'error',
    'security/detect-non-literal-require': 'warn',
    'security/detect-object-injection': 'warn',
    'security/detect-possible-timing-attacks': 'error',
    'security/detect-pseudoRandomBytes': 'error',
    'space-before-function-paren': 'off',
    '@typescript-eslint/default-param-last': 'off',
    '@typescript-eslint/strict-boolean-expressions': 0
  }
};
