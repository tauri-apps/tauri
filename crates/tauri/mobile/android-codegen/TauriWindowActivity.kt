// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/* THIS FILE IS AUTO-GENERATED. DO NOT MODIFY!! */

package {{package}}

import android.os.Bundle
import androidx.activity.enableEdgeToEdge

// Hosts the windows that do not set an activity name, each one in a task of its own.
// Tauri declares it in AndroidManifest.xml.
class TauriWindowActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
  }
}
