// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import autoAdapter from '@sveltejs/adapter-auto'
import staticAdapter from '@sveltejs/adapter-static'
import preprocess from 'svelte-preprocess'

const TARGET = process.env.TARGET

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Consult https://github.com/sveltejs/svelte-preprocess
  // for more information about preprocessors
  preprocess: preprocess(),

  kit: {
    adapter:
      TARGET === 'tauri'
        ? staticAdapter({
            fallback: 'index.html'
          })
        : autoAdapter()
  }
}

export default config
