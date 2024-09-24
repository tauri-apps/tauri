// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { defineConfig, presetIcons, presetUno, presetWebFonts } from 'unocss'
import extractorSvelte from '@unocss/extractor-svelte'

export default defineConfig({
  theme: {
    colors: {
      primary: '#FFFFFF',
      primaryLighter: '#e9ecef',
      darkPrimary: '#1B1B1D',
      darkPrimaryLighter: '#242526',
      primaryText: '#1C1E21',
      darkPrimaryText: '#E3E3E3',
      secondaryText: '#858A91',
      darkSecondaryText: '#C2C5CA',
      accent: '#3578E5',
      accentDark: '#306cce',
      accentDarker: '#2d66c3',
      accentDarkest: '#2554a0',
      accentLight: '#538ce9',
      accentLighter: '#72a1ed',
      accentLightest: '#9abcf2',
      accentText: '#FFFFFF',
      darkAccent: '#67d6ed',
      darkAccentDark: '#49cee9',
      darkAccentDarker: '#39cae8',
      darkAccentDarkest: '#19b5d5',
      darkAccentLight: '#85def1',
      darkAccentLighter: '#95e2f2',
      darkAccentLightest: '#c2eff8',
      darkAccentText: '#1C1E21',
      code: '#d6d8da',
      codeDark: '#282a2e',
      hoverOverlay: 'rgba(0,0,0,.05)',
      hoverOverlayDarker: 'rgba(0,0,0,.1)',
      darkHoverOverlay: 'hsla(0,0%,100%,.05)',
      darkHoverOverlayDarker: 'hsla(0,0%,100%,.1)'
    }
  },
  preflights: [
    {
      getCSS: ({ theme }) => `
    ::-webkit-scrollbar-thumb {
      background-color: ${theme.colors.accent};
    }

    .dark ::-webkit-scrollbar-thumb {
      background-color: ${theme.colors.darkAccent};
    }

    code {
      font-size: ${theme.fontSize.xs[0]};
      font-family: ${theme.fontFamily.mono};
      border-radius: ${theme.borderRadius['DEFAULT']};
      background-color: ${theme.colors.code};
    }

    .code-block {
      font-family: ${theme.fontFamily.mono};
      font-size: ${theme.fontSize.sm[0]};
    }

    .dark code {
      background-color: ${theme.colors.codeDark};
    }
    `
    }
  ],
  shortcuts: {
    btn: `select-none outline-none shadow-md p-2 rd-1 text-primaryText border-none font-400 dark:font-600
            bg-accent hover:bg-accentDarker active:bg-accentDarkest text-accentText
            dark:bg-darkAccent dark:hover:bg-darkAccentDarker dark:active:bg-darkAccentDarkest dark:text-darkAccentText`,
    nv: `decoration-none flex items-center relative p-2 rd-1 transition-all-125 ease
            text-darkSecondaryText
            hover:text-accent dark:hover:text-darkAccent
            hover:bg-darkHoverOverlay hover:border-l-4`,
    nv_selected: `nv bg-darkHoverOverlay text-accent dark:text-darkAccent border-l-4`,
    note: `decoration-none flex-inline items-center relative p-2 rd-1
             border-l-4 border-accent dark:border-darkAccent
             bg-accent/10 dark:bg-darkAccent/10`,
    'note-red':
      'note bg-red-700/10 dark:bg-red-700/10 after:bg-red-700 dark:after:bg-red-700',
    input:
      'flex items-center outline-none border-none py-3 px-2 rd-1 shadow-md bg-primaryLighter dark:bg-darkPrimaryLighter text-primaryText dark:text-darkPrimaryText',
    checkbox: 'accent-accent'
  },
  presets: [
    presetUno(),
    presetIcons(),
    presetWebFonts({
      fonts: {
        sans: 'Rubik',
        mono: ['Fira Code', 'Fira Mono:400,700']
      }
    })
  ],
  extractors: [extractorSvelte]
})
