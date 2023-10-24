#!/usr/bin/env node

// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const json = process.argv[2]
const field = process.argv[3]

const output = JSON.parse(json)
console.log(output[field])
