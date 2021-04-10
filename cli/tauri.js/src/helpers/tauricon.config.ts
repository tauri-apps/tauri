// Copyright 2019-2021 Tauri Programme within The Commons Conservancy and Contributors
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

export const options = {
  // folder determines in which path to drop the generated file
  // prefix is the first part of the generated file's name
  // infix adds e.g. '44x44' based on the size in sizes to the generated file's name
  // suffix adds a file-ending to the generated file's name
  // sizes determines the pixel width and height to use
  background_color: '#000074',
  theme_color: '#02aa9b',
  sharp: 'kernel: sharp.kernel.lanczos3', // one of [nearest|cubic|lanczos2|lanczos3]
  minify: {
    batch: false,
    overwrite: true,
    available: ['pngquant', 'optipng', 'zopfli'],
    type: 'pngquant',
    pngcrushOptions: {
      reduce: true
    },
    pngquantOptions: {
      quality: [0.6, 0.8],
      floyd: 0.1, // 0.1 - 1
      speed: 10 // 1 - 10
    },
    optipngOptions: {
      optimizationLevel: 4,
      bitDepthReduction: true,
      colorTypeReduction: true,
      paletteReduction: true
    },
    zopfliOptions: {
      transparent: true,
      more: true
    }
  },
  splash_type: 'generate',
  tauri: {
    linux: {
      folder: '.',
      prefix: '',
      infix: true,
      suffix: '.png',
      sizes: [32, 128]
    },
    linux_2x: {
      folder: '.',
      prefix: '128x128@2x',
      infix: false,
      suffix: '.png',
      sizes: [256]
    },
    defaults: {
      folder: '.',
      prefix: 'icon',
      infix: false,
      suffix: '.png',
      sizes: [512]
    },
    appx_logo: {
      folder: '.',
      prefix: 'StoreLogo',
      infix: false,
      suffix: '.png',
      sizes: [50]
    },
    appx_square: {
      folder: '.',
      prefix: 'Square',
      infix: true,
      suffix: 'Logo.png',
      sizes: [30, 44, 71, 89, 107, 142, 150, 284, 310]
    }
    // todo: look at capacitor and cordova for insight into what icons
    // we need for those distribution targets
  }
}
