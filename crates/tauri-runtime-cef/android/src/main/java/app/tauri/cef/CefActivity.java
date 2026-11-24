// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
package app.tauri.cef;

import android.os.Bundle;
import android.view.ViewGroup;
import android.widget.FrameLayout;
import com.google.androidgamesdk.GameActivity;
import app.tauri.plugin.PluginManager;

/** A GameActivity for winit with an Android view container for CEF. */
public class CefActivity extends GameActivity {
    private FrameLayout cefContainer;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        PluginManager.INSTANCE.onCreate(this);
        cefContainer = new FrameLayout(this);
        addContentView(cefContainer, new ViewGroup.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
    }

    public PluginManager getPluginManager() { return PluginManager.INSTANCE; }

    @Override
    protected void onDestroy() {
        PluginManager.INSTANCE.onDestroy(this);
        super.onDestroy();
    }

    public FrameLayout getCefContainer() { return cefContainer; }

    public void postCefTask(long task) {
        runOnUiThread(() -> nativeRunTask(task));
    }

    private static native void nativeRunTask(long task);
}
