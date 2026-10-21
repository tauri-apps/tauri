// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import globals from 'globals'
import eslint from '@eslint/js'
import prettierConfig from 'eslint-config-prettier'
import securityPlugin from 'eslint-plugin-security'
import tseslint from 'typescript-eslint'

/** @type {import('eslint').Linter.Config} */
export default [
  eslint.configs.recommended,
  prettierConfig,
  securityPlugin.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  {
    languageOptions: {
      globals: {
        ...globals.node,
        ...globals.jest,
        __statics: true,
        process: true
      },
      parserOptions: {
        project: true,
        tsconfigRootDir: import.meta.dirname
      }
    },
    rules: {
      'no-console': 'error',
      'no-debugger': 'error',
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
      '@typescript-eslint/strict-boolean-expressions': 0,
      'no-return-await': 'warn',
      '@typescript-eslint/return-await': 'off',
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          caughtErrorsIgnorePattern: '^_'
        }
      ]
    }
  }
]
