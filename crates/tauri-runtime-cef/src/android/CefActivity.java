// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
package app.tauri.cef;

import android.Manifest;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.content.res.Configuration;
import android.os.Bundle;
import android.view.ViewGroup;
import android.widget.FrameLayout;
import androidx.activity.result.ActivityResultLauncher;
import androidx.activity.result.contract.ActivityResultContracts;
import androidx.core.content.ContextCompat;
import com.google.androidgamesdk.GameActivity;
import app.tauri.plugin.PluginManager;
import java.util.ArrayDeque;
import java.util.ArrayList;

/** A GameActivity for winit with an Android view container for CEF. */
public class CefActivity extends GameActivity {
    private FrameLayout cefContainer;
    private long onDestroyTask;
    private boolean cefDestroyed;
    private ActivityResultLauncher<String[]> mediaPermissionLauncher;
    private final ArrayDeque<MediaPermissionRequest> mediaPermissionRequests = new ArrayDeque<>();

    private static class MediaPermissionRequest {
        final String[] permissions;
        final long callback;

        MediaPermissionRequest(String[] permissions, long callback) {
            this.permissions = permissions;
            this.callback = callback;
        }
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        PluginManager.INSTANCE.onCreate(this);

        mediaPermissionLauncher = registerForActivityResult(
                new ActivityResultContracts.RequestMultiplePermissions(), result -> {
                    MediaPermissionRequest request = mediaPermissionRequests.poll();
                    if (request == null) return;

                    boolean granted = true;
                    for (String permission : request.permissions) {
                        granted &= Boolean.TRUE.equals(result.get(permission));
                    }

                    launchNextMediaPermissionRequest();
                    nativeMediaPermissionResult(request.callback, granted);
                });

        cefContainer = new FrameLayout(this);
        addContentView(cefContainer, new ViewGroup.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
    }

    public PluginManager getPluginManager() { return PluginManager.INSTANCE; }

    @Override
    protected void onNewIntent(Intent intent) {
        super.onNewIntent(intent);
        PluginManager.INSTANCE.onNewIntent(intent);
    }

    @Override
    protected void onRestart() {
        super.onRestart();
        PluginManager.INSTANCE.onRestart(this);
    }

    @Override
    protected void onResume() {
        super.onResume();
        PluginManager.INSTANCE.onResume(this);
    }

    @Override
    protected void onPause() {
        super.onPause();
        PluginManager.INSTANCE.onPause(this);
    }

    @Override
    protected void onStop() {
        super.onStop();
        PluginManager.INSTANCE.onStop(this);
    }

    @Override
    protected void onDestroy() {
        while (!mediaPermissionRequests.isEmpty()) {
            nativeMediaPermissionResult(mediaPermissionRequests.remove().callback, false);
        }

        PluginManager.INSTANCE.onDestroy(this);

        // GameActivity joins the native loop in super.onDestroy(). Notify it
        // first, since winit currently does not forward Android's destroy event.
        long task;
        synchronized (this) {
            cefDestroyed = true;
            task = onDestroyTask;
            onDestroyTask = 0;
        }
        if (task != 0) nativeRunTask(task);

        super.onDestroy();
    }

    @Override
    public void onConfigurationChanged(Configuration configuration) {
        super.onConfigurationChanged(configuration);
        PluginManager.INSTANCE.onConfigurationChanged(configuration);
    }

    public FrameLayout getCefContainer() { return cefContainer; }

    public void postCefTask(long task) {
        runOnUiThread(() -> nativeRunTask(task));
    }

    public synchronized boolean setOnDestroyTask(long task) {
        if (onDestroyTask != 0) return false;
        if (cefDestroyed) nativeRunTask(task);
        else onDestroyTask = task;
        return true;
    }

    public void requestMediaPermissions(boolean audio, boolean video, long callback) {
        if (isFinishing() || isDestroyed()) {
            nativeMediaPermissionResult(callback, false);
            return;
        }

        ArrayList<String> permissions = new ArrayList<>();
        if (audio) permissions.add(Manifest.permission.RECORD_AUDIO);
        if (video) permissions.add(Manifest.permission.CAMERA);

        permissions.removeIf(permission -> ContextCompat.checkSelfPermission(this, permission)
                == PackageManager.PERMISSION_GRANTED);

        if (permissions.isEmpty()) {
            nativeMediaPermissionResult(callback, true);
            return;
        }

        mediaPermissionRequests.add(new MediaPermissionRequest(
                permissions.toArray(new String[0]), callback));
        if (mediaPermissionRequests.size() == 1) launchNextMediaPermissionRequest();
    }

    private void launchNextMediaPermissionRequest() {
        MediaPermissionRequest next = mediaPermissionRequests.peek();
        if (next != null) mediaPermissionLauncher.launch(next.permissions);
    }

    private static native void nativeRunTask(long task);
    private static native void nativeMediaPermissionResult(long callback, boolean granted);
}
