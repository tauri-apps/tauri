// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package com.plugin.sample

import android.util.Log

class Example {
    fun pong(value: String): String {
        Log.i("Pong", value)
        return value
    }
}
