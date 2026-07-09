#!/usr/bin/env node
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Headless CI smoke test for a built tauri_ffi library (point TAURI_FFI_LIB
// at the artifact). Exercises loading, version/ABI, error reporting and the
// handle registry — everything that works without a display or event loop.

import assert from 'node:assert/strict'
import { open, CODES } from '../node/src/ffi.js'
import { ABI_VERSION } from '../node/src/ffi-decls.js'

const { api, check, libPath } = open()

const version = api.ffiVersion()
const abi = api.ffiAbiVersion()
assert.match(version, /^\d+\.\d+\.\d+/, 'ffi version is semver')
assert.equal(abi, ABI_VERSION, 'library ABI matches generated bindings')

// Error path: malformed config must fail with a message.
const outBuilder = [0]
const bad = api.appBuilderNew('{not json', outBuilder)
assert.equal(bad, CODES.INVALID_ARG, 'malformed config is rejected')
assert.ok(api.lastErrorMessage()?.includes('invalid config'), 'error message is set')

// Happy path: builder creation and handle lifecycle (no event loop needed).
check(
  api.appBuilderNew(
    JSON.stringify({ productName: 'smoke', version: '0.0.0', identifier: 'com.tauri.ffi.smoke' }),
    outBuilder
  ),
  'builder_new'
)
assert.ok(outBuilder[0] > 0, 'builder handle is valid')
check(api.appBuilderRegisterCommand(outBuilder[0], 'ping'), 'register_command')
check(api.handleClose(outBuilder[0]), 'handle_close')
assert.equal(api.handleClose(0), CODES.OK, 'closing an unknown handle is a no-op')

console.log(`smoke ok: tauri-ffi ${version} (abi ${abi}) at ${libPath}`)
