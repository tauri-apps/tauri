# Changelog

## \[0.35.3]

- [`1bcd5165`](https://github.com/tauri-apps/tao/commit/1bcd51652763fa6d9512370af6adaea140053891) ([#1224](https://github.com/tauri-apps/tao/pull/1224) by [@brtinney](https://github.com/tauri-apps/tao/../../brtinney)) fix(android): don't panic on `onNewIntent` when `intent.getType()` returns null

## \[0.35.2]

- [`98aa0536`](https://github.com/tauri-apps/tao/commit/98aa05369aeeeaecf67ea791e43a5fecc11535af) ([#1211](https://github.com/tauri-apps/tao/pull/1211) by [@SAY-5](https://github.com/tauri-apps/tao/../../SAY-5)) fix(android): don't panic on getCurrentWindowMetrics on API<30

## \[0.35.1]

- [`f3363af7`](https://github.com/tauri-apps/tao/commit/f3363af78a5fc5ef9ce4829b8c72c0232742fd31) ([#1212](https://github.com/tauri-apps/tao/pull/1212) by [@lucasfernog](https://github.com/tauri-apps/tao/../../lucasfernog)) Describe exceptions when they are raised by a JNI call.

## \[0.35.0]

- [`4e7c2f4a`](https://github.com/tauri-apps/tao/commit/4e7c2f4aa06e368403c6f170997fafd0c9a88418) ([#1154](https://github.com/tauri-apps/tao/pull/1154) by [@lucasfernog](https://github.com/tauri-apps/tao/../../lucasfernog)) **Breaking change:** The Android activity should now reference and call the following external functions:

        private external fun onActivityCreate(activity)
        private external fun start()
        private external fun resume()
        private external fun pause()
        private external fun stop()
        private external fun onActivitySaveInstanceState()
        private external fun onActivityDestroy(activity)
        private external fun onActivityLowMemory()
- [`18040018`](https://github.com/tauri-apps/tao/commit/18040018831de762473189999768a4b44ad03c63) ([#1146](https://github.com/tauri-apps/tao/pull/1146) by [@JeffTsang](https://github.com/tauri-apps/tao/../../JeffTsang)) `Event::Resumed` is now only emitted when the app is actually resumed (going back to foreground) so it won't be called on app startup.
- [`56e9840b`](https://github.com/tauri-apps/tao/commit/56e9840b356d4bb97fd9db8477f5aac8640f6b6f) ([#1141](https://github.com/tauri-apps/tao/pull/1141) by [@haecker-felix](https://github.com/tauri-apps/tao/../../haecker-felix)) Use the Linux XDG Desktop Portal to add support for the system color scheme. Needs `dbus` feature flag.
- [`4e7c2f4a`](https://github.com/tauri-apps/tao/commit/4e7c2f4aa06e368403c6f170997fafd0c9a88418) ([#1154](https://github.com/tauri-apps/tao/pull/1154) by [@lucasfernog](https://github.com/tauri-apps/tao/../../lucasfernog)) Added multi-window support for iOS and Android.

  Leverages [scenes](https://developer.apple.com/documentation/uikit/scenes) on iOS and [Activity embedding](https://developer.android.com/develop/ui/views/layout/activity-embedding) on Android.

  iOS:

  - Added Event::SceneRequested (on iPad the user can request a new window to be open - e.g. by long pressing the app icon and selecting "New window")
  - Request new scene to be created on Window::new (if needed, main scene is detected automatically) and assign the window instance later when it gets connected

  Android:

  - Create new activity on Window::new (if needed, main activity is detected automatically)
  - Added builder methods to determine the activity to be created
  - System determines what to do with the activity (new stack, next to another one.. based on the embedding rules)
- [`9cea0358`](https://github.com/tauri-apps/tao/commit/9cea03582a3272c8fe60f20b519ff0cd562448b7) ([#1155](https://github.com/tauri-apps/tao/pull/1155) by [@lucasfernog](https://github.com/tauri-apps/tao/../../lucasfernog)) Fire `Event::Opened` on Android, which now requires the activity to call the onNewIntent(intent) external function.
- [`18040018`](https://github.com/tauri-apps/tao/commit/18040018831de762473189999768a4b44ad03c63) ([#1146](https://github.com/tauri-apps/tao/pull/1146) by [@JeffTsang](https://github.com/tauri-apps/tao/../../JeffTsang)) Use WillEnterForeground instead of DidBecomeActive for Event::Resumed in iOS.

## \[0.34.8]

- [`2fb51231`](https://github.com/tauri-apps/tao/commit/2fb512315899973e2c64d00c0e34ed382d93c893) ([#1182](https://github.com/tauri-apps/tao/pull/1182) by [@Tunglies](https://github.com/tauri-apps/tao/../../Tunglies)) Remove synchronous styleMask mutations in is_zoomed on macOS

## \[0.34.7]

- [`541cf20e`](https://github.com/tauri-apps/tao/commit/541cf20ed1f7bf15576db2faaac35d8beb4ab193) ([#1190](https://github.com/tauri-apps/tao/pull/1190) by [@damadczar](https://github.com/tauri-apps/tao/../../damadczar)) fix(linux): exit cleanly when GtkApplication is a remote GIO instance

## \[0.34.6]

- [`a4837862`](https://github.com/tauri-apps/tao/commit/a48378626f728665526e0c65fce0142282ca1f34) ([#1189](https://github.com/tauri-apps/tao/pull/1189) by [@thomaseizinger](https://github.com/tauri-apps/tao/../../thomaseizinger)) Downgrades several logs in the Windows backend from WARN to DEBUG.
- [`e1c9cb69`](https://github.com/tauri-apps/tao/commit/e1c9cb699b6e0943f45f27fbf558efce8d617f5e) ([#1185](https://github.com/tauri-apps/tao/pull/1185) by [@kanatapple](https://github.com/tauri-apps/tao/../../kanatapple)) Prevent panics on Windows when a monitor handle becomes invalid during info retrieval.
- [`e196538f`](https://github.com/tauri-apps/tao/commit/e196538f989894fcb92401fced54d7d3a65fc91a) ([#1165](https://github.com/tauri-apps/tao/pull/1165) by [@Slinetrac](https://github.com/tauri-apps/tao/../../Slinetrac)) fix(windows): respect default app mode via registry in dark mode detection
- [`a133504b`](https://github.com/tauri-apps/tao/commit/a133504b6dc963a5ad7786e01e746dba72236b65) ([#1184](https://github.com/tauri-apps/tao/pull/1184) by [@Tunglies](https://github.com/tauri-apps/tao/../../Tunglies)) Replace NSString::from_str with ns_string macro in macOS. No user facing changes.

## \[0.34.5]

- [`92e22209`](https://github.com/tauri-apps/tao/commit/92e22209fb58248449175b5b86697243d6a93db2) ([#1152](https://github.com/tauri-apps/tao/pull/1152) by [@lucasfernog](https://github.com/tauri-apps/tao/../../lucasfernog)) Emit `Event::LoopDestroyed` on activity destroy on Android.
- [`da1514be`](https://github.com/tauri-apps/tao/commit/da1514be09993ea4aace8c64cfdf0af14f22b818) ([#1150](https://github.com/tauri-apps/tao/pull/1150) by [@Legend-Master](https://github.com/tauri-apps/tao/../../Legend-Master)) Fix maximized windows have empty edges when using auto hide task bar on Windows

## \[0.34.4]

- [`25f2c58a`](https://github.com/tauri-apps/tao/commit/25f2c58aca96215113d1a4feae5c854e3349cf06) ([#1148](https://github.com/tauri-apps/tao/pull/1148) by [@lucasfernog](https://github.com/tauri-apps/tao/../../lucasfernog)) Trigger `WindowEvent::Destroyed` when the Android activity is destroyed. In this case, the app should either exit by setting the control flow to `ControlFlow::ExitWithCode` or NOT call the create() external function when the activity is recreated and `onCreate` is called, handling how to recreate the app window via a separate hook that can leverage the existing tao event loop.

## \[0.34.3]

- [`0defcd01`](https://github.com/tauri-apps/tao/commit/0defcd015c615b95f42ce3437b06174816f76602) ([#1132](https://github.com/tauri-apps/tao/pull/1132) by [@Legend-Master](https://github.com/tauri-apps/tao/../../Legend-Master)) Exclude audit PDF file from the crate published to crates.io
- [`3969a1a4`](https://github.com/tauri-apps/tao/commit/3969a1a45cfe2e173431eb1134b49e03c30bdc32) ([#1138](https://github.com/tauri-apps/tao/pull/1138) by [@aspcartman](https://github.com/tauri-apps/tao/../../aspcartman)) On macOS, fixed an issue that caused the window background color to be applied incorrectly (typically black).
- [`28f5a96a`](https://github.com/tauri-apps/tao/commit/28f5a96a7d80859d97a84586b6c68fd4daac05d4) ([#1119](https://github.com/tauri-apps/tao/pull/1119) by [@Legend-Master](https://github.com/tauri-apps/tao/../../Legend-Master)) Fix `WindowEvent::MouseWheel` doesn't account for mouse wheel speed settings
- [`a1edbeb4`](https://github.com/tauri-apps/tao/commit/a1edbeb448d1104b8e9dd3181a04d45ff846a397) ([#1126](https://github.com/tauri-apps/tao/pull/1126) by [@Legend-Master](https://github.com/tauri-apps/tao/../../Legend-Master)) Emit `Event::LoopDestroyed` on receiving `WM_ENDSESSION` message on Windows

## \[0.34.2]

- [`60a47340`](https://github.com/tauri-apps/tao/commit/60a47340c967eaaf5b4a41f0be6708d6daf950e3) ([#1108](https://github.com/tauri-apps/tao/pull/1108) by [@Simon-Laux](https://github.com/tauri-apps/tao/../../Simon-Laux)) feat: MacOS: add universal applink support
  by implementing `application:willContinueUserActivityWithType:` and `application:continueUserActivity:restorationHandler:`,
  reusing the existing `Event::Opened { urls }` event for the user facing api.

## \[0.34.1]

- [`f73c70fd`](https://github.com/tauri-apps/tao/commit/f73c70fd870016dc1298599cdbc9bcaa8db5d7bf) ([#1120](https://github.com/tauri-apps/tao/pull/1120) by [@robertrpf](https://github.com/tauri-apps/tao/../../robertrpf)) Added `WindowBuilder::with_focusable` to allow creating unfocusable windows.
- [`f73c70fd`](https://github.com/tauri-apps/tao/commit/f73c70fd870016dc1298599cdbc9bcaa8db5d7bf) ([#1120](https://github.com/tauri-apps/tao/pull/1120) by [@robertrpf](https://github.com/tauri-apps/tao/../../robertrpf)) Added `Window::set_focusable`.

## \[0.34.0]

- [`773d324b`](https://github.com/tauri-apps/tao/commit/773d324be5462fbe472b9bec7505bb04bdc1cf9a) ([#1121](https://github.com/tauri-apps/tao/pull/1121) by [@FabianLars](https://github.com/tauri-apps/tao/../../FabianLars)) Fixed an issue that caused a panic on macOS when tao received `nil` from the OS when trying to get the current NSScreen.
- [`ba65486a`](https://github.com/tauri-apps/tao/commit/ba65486abe14f2963e016731520e0cdadce3e199) ([#1107](https://github.com/tauri-apps/tao/pull/1107) by [@Legend-Master](https://github.com/tauri-apps/tao/../../Legend-Master)) Fix initial position gets reset to 0 on Windows if it's accounted for the shadow
- [`4a085054`](https://github.com/tauri-apps/tao/commit/4a08505412ed365ba160832305c5cfec5b53e622) ([#1103](https://github.com/tauri-apps/tao/pull/1103) by [@aurelj](https://github.com/tauri-apps/tao/../../aurelj)) Added `x11` feature flag (enabled by default).

## \[0.33.0]

- [`dae6d887`](https://github.com/tauri-apps/tao/commit/dae6d8875dbc10ce0ab45d9a00072436b3688b0e) ([#1100](https://github.com/tauri-apps/tao/pull/1100) by [@Legend-Master](https://github.com/tauri-apps/tao/../../Legend-Master)) Updated `windows` to `0.61`.

## \[0.32.8]

- [`b863d49c`](https://github.com/tauri-apps/tao/commit/b863d49c9735755ef6a51127b393a25b1a35d296) ([#1058](https://github.com/tauri-apps/tao/pull/1058) by [@1111mp](https://github.com/tauri-apps/tao/../../1111mp)) macOS: Add `set_dock_visibility` method to support setting the visibility of the application in the dock.
- [`996e28df`](https://github.com/tauri-apps/tao/commit/996e28dfe1ae9c7d41bebf1f9b2b9cfecf9e81b2) ([#1090](https://github.com/tauri-apps/tao/pull/1090) by [@lucasfernog](https://github.com/tauri-apps/tao/../../lucasfernog)) Fix crash when sending key event when macOS app has no window opened.
- [`dd9251a8`](https://github.com/tauri-apps/tao/commit/dd9251a8f0a954ed4fa251db8bec29b336545379) ([#1091](https://github.com/tauri-apps/tao/pull/1091) by [@1111mp](https://github.com/tauri-apps/tao/../../1111mp)) Fix `Window::theme()` always returning `Theme::Light` and `WindowEvent::ThemeChanged()` not delivered on macOS.

## \[0.32.7]

- [`1951b9ab`](https://github.com/tauri-apps/tao/commit/1951b9abbb4bf424e573c75161db40e68c4c603d) ([#1083](https://github.com/tauri-apps/tao/pull/1083) by [@gezihuzi](https://github.com/tauri-apps/tao/../../gezihuzi)) Fixed application crash during startup when certain window buttons are disabled on macOS.

## \[0.32.6]

- [`9d1da74b`](https://github.com/tauri-apps/tao/commit/9d1da74bb70f7bb17a4d41532c2b94c709b65ec8) ([#1080](https://github.com/tauri-apps/tao/pull/1080) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) Fix crash on Windows because of missing functions on older Windows systems, regression in 0.32

## \[0.32.5]

- [`08c9c4c6`](https://github.com/tauri-apps/tao/commit/08c9c4c6e4ffa7a96ed6888d11f0b49f5819705d) ([#1078](https://github.com/tauri-apps/tao/pull/1078) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) Fix `Window::set_focus` regression on macOS

## \[0.32.4]

- [`4679d683`](https://github.com/tauri-apps/tao/commit/4679d6835ab3ea5ea125dcf812d4c504330ec12e) ([#1056](https://github.com/tauri-apps/tao/pull/1056) by [@dgerhardt](https://github.com/tauri-apps/tao/../../dgerhardt)) On Windows 11, fix incorrect window positioning and sizing on `WM_DPICHANGED`.

## \[0.32.3]

- [`c91dcde7`](https://github.com/tauri-apps/tao/commit/c91dcde76f9c451aff8a295ccfe9741fc1b9dbba) ([#1075](https://github.com/tauri-apps/tao/pull/1075) by [@Legend-Master](https://github.com/tauri-apps/tao/../../Legend-Master)) On Windows, fix `Window::inner_size` always returns the restore size instead of the current size for maximized undecorated window

## \[0.32.2]

- [`b296cf53`](https://github.com/tauri-apps/tao/commit/b296cf53f43c99a5cabdb443c489f2acb5d1e853) ([#1074](https://github.com/tauri-apps/tao/pull/1074) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) On Windows, add `WindowExtWindows::has_undecorated_shadow` to check if window has shadows for undecorated window or not.
- [`f4ec11d7`](https://github.com/tauri-apps/tao/commit/f4ec11d795ea3ffbc66514e28fd973a2e73c83da) ([#1070](https://github.com/tauri-apps/tao/pull/1070) by [@Legend-Master](https://github.com/tauri-apps/tao/../../Legend-Master)) Fix undecorated top left and right border resizing direction on Windows

## \[0.32.1]

- [`1be722db`](https://github.com/tauri-apps/tao/commit/1be722dbc2a4341f576d9aa0c0bfc01825da8200) ([#1071](https://github.com/tauri-apps/tao/pull/1071) by [@lucasfernog](https://github.com/tauri-apps/tao/../../lucasfernog)) Fix content protection on macOS crashing the app.

## \[0.32.0]

- [`94afde98`](https://github.com/tauri-apps/tao/commit/94afde98436bf743719205f5b9aed0d2ee78169d) ([#1040](https://github.com/tauri-apps/tao/pull/1040) by [@Teddytrombone](https://github.com/tauri-apps/tao/../../Teddytrombone)) Add missing function keys F13-F24 to linux implementation
- [`36645136`](https://github.com/tauri-apps/tao/commit/3664513621459cbb3909d82f9abf8fb402525150) ([#1050](https://github.com/tauri-apps/tao/pull/1050) by [@FabianLars](https://github.com/tauri-apps/tao/../../FabianLars)) Raised MSRV to 1.74
- [`8c8f0e8b`](https://github.com/tauri-apps/tao/commit/8c8f0e8b79dfb09c3775cf87be212eb382fa7589) ([#1049](https://github.com/tauri-apps/tao/pull/1049) by [@madsmtm](https://github.com/tauri-apps/tao/../../madsmtm)) Use `objc2`.
- [`6fda4984`](https://github.com/tauri-apps/tao/commit/6fda49843e61c6adb650e5af8311feb55cf019cf) ([#1048](https://github.com/tauri-apps/tao/pull/1048) by [@madsmtm](https://github.com/tauri-apps/tao/../../madsmtm)) macOS: Remove `From<ActivationPolicy>` implementation for `cocoa::appkit::NSApplicationActivationPolicy`.
- [`28f728c7`](https://github.com/tauri-apps/tao/commit/28f728c7bf6b6fe9c66098515ac780b3bee320b1) ([#1068](https://github.com/tauri-apps/tao/pull/1068) by [@Legend-Master](https://github.com/tauri-apps/tao/../../Legend-Master)) Update `windows` to 0.60
- [`5cc92980`](https://github.com/tauri-apps/tao/commit/5cc92980c4b112125668e83c436524d53402e983) ([#1052](https://github.com/tauri-apps/tao/pull/1052) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) On Windows, fix regression `Window::inner_size` reporting larger size than what's visible for undecorated window.
- [`5cc92980`](https://github.com/tauri-apps/tao/commit/5cc92980c4b112125668e83c436524d53402e983) ([#1052](https://github.com/tauri-apps/tao/pull/1052) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) On Windows, undecorated window with shadows, now have native resize handles outside of the window client area.

## \[0.31.1]

- [`83e35e96`](https://github.com/tauri-apps/tao/commit/83e35e961f4893790b913ee2efc15ae33fd16fb2) ([#1036](https://github.com/tauri-apps/tao/pull/1036) by [@FabioGNR](https://github.com/tauri-apps/tao/../../FabioGNR)) Call `gtk::init` when creating the eventloop to fix crashes with some gtk APIs.
- [`bb537fe9`](https://github.com/tauri-apps/tao/commit/bb537fe9969e490b74548738f16173d5b9d84c63) ([#1039](https://github.com/tauri-apps/tao/pull/1039) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) On Windows, fix regression that caused `Window::set_size` to have no effect at all for undecorated window with shadows.

## \[0.31.0]

- [`5d6d7da0`](https://github.com/tauri-apps/tao/commit/5d6d7da0ade44e08b33496bf445afb69a09037f0) ([#1017](https://github.com/tauri-apps/tao/pull/1017) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) On Windows, fix regression caused undecorated window with shadows to be slightly larger on creation.
- [`2e6cf1a4`](https://github.com/tauri-apps/tao/commit/2e6cf1a4332093806a5d277098b6573299256622) ([#1022](https://github.com/tauri-apps/tao/pull/1022) by [@Jnschrber](https://github.com/tauri-apps/tao/../../Jnschrber)) On Windows, fix crash on older windows versions that doesn't support dark mode.
- [`6b49f55a`](https://github.com/tauri-apps/tao/commit/6b49f55a96ac47d0e645a04c7e4fabe4dd063196) ([#1016](https://github.com/tauri-apps/tao/pull/1016) by [@Legend-Master](https://github.com/tauri-apps/tao/../../Legend-Master)) Expose raw gdk monitor through `MonitorHandleExtUnix::gdk_monitor`
- [`720bd93f`](https://github.com/tauri-apps/tao/commit/720bd93f9798e5fa2a8adc2c93dde2ce3c2c9700) ([#1018](https://github.com/tauri-apps/tao/pull/1018) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) On Windows, fix regression in initial window position when using logical positions.
- [`73741a75`](https://github.com/tauri-apps/tao/commit/73741a75409a93267947d92697616437ae4fccb8) ([#1008](https://github.com/tauri-apps/tao/pull/1008) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) Remove `instant` dependency, changed `StartCause::ResumeTimeReached`, `StartCause::WaitCancelled` and `ControlFlow::WaitUntil` to use `std::time::Instant` instead.
- [`fa9aaa60`](https://github.com/tauri-apps/tao/commit/fa9aaa6066dcc0316d57038fc1b1e3353dc5c3e7) ([#1019](https://github.com/tauri-apps/tao/pull/1019) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) On Windows, fix fullscreen for undecorated window have white borders.

## \[0.30.7]

- [`97382238`](https://github.com/tauri-apps/tao/commit/97382238b218d66baf5693b6087b7bef2e66ec70) ([#1007](https://github.com/tauri-apps/tao/pull/1007) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) On Windows, fix `Window::set_inner_size` regression not handling borders correctly for undecorated window with shadows.
- [`97382238`](https://github.com/tauri-apps/tao/commit/97382238b218d66baf5693b6087b7bef2e66ec70) ([#1007](https://github.com/tauri-apps/tao/pull/1007) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) On Windows, fix `Window::inner_size` regression returning incorrect size for window with decorations.

## \[0.30.6]

- [`1f72c246`](https://github.com/tauri-apps/tao/commit/1f72c2465edcc9bee9170bcfd74d4e917e48febc) ([#1002](https://github.com/tauri-apps/tao/pull/1002) by [@ahqsoftwares](https://github.com/tauri-apps/tao/../../ahqsoftwares)) Add `WindowExtUnix::set_badge_count` for Linux, `WindowExtIos::set_badge_count` for iOS, `WindowExtMacos::set_badge_label` for Macos, `MacdowExtWindows::set_overlay_icon` for Windows
- [`946f8049`](https://github.com/tauri-apps/tao/commit/946f804995208fd9796d6baee828392c4ce2056b) ([#1005](https://github.com/tauri-apps/tao/pull/1005) by [@lucasfernog](https://github.com/tauri-apps/tao/../../lucasfernog)) Fix memory leak on Android.
- [`aff33fbb`](https://github.com/tauri-apps/tao/commit/aff33fbb2dab66cca2481fb75a3a6d612689269c) ([#1001](https://github.com/tauri-apps/tao/pull/1001) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) On Linux, `Window::outer_position`, `Window::outer_size` and `WindowEvent::Moved` to include/account for borders and titlebar.
- [`06d109fe`](https://github.com/tauri-apps/tao/commit/06d109feb3fd0f77004c8c576fc1e54e7e82f867) ([#993](https://github.com/tauri-apps/tao/pull/993) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) On Windows, fix `Window::inner_size` returns slightly larger than what's visible for undecorated windows but have shadows.
- [`edfbd364`](https://github.com/tauri-apps/tao/commit/edfbd364a6abf8704de53a31ab10d0ce595cbde7) ([#992](https://github.com/tauri-apps/tao/pull/992) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) On Windows, fix `WindowBuilder::with_position` with a position on a non-primary monitor resulting in an incorrectly positioned window.

## \[0.30.5]

- [`532b5ab0`](https://github.com/tauri-apps/tao/commit/532b5ab0bffb1e7b39281951848947feb0716f0a) ([#1000](https://github.com/tauri-apps/tao/pull/1000) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) Fix `android_binding` macro incorrect expansion.
- [`67e44e4c`](https://github.com/tauri-apps/tao/commit/67e44e4cb2d278333abe367f23f0e5ef0d2463db) ([#991](https://github.com/tauri-apps/tao/pull/991) by [@Legend-Master](https://github.com/tauri-apps/tao/../../Legend-Master)) Fix setting theme to `None` crashes the app on macOS

## \[0.30.4]

- [`b404cde1`](https://github.com/tauri-apps/tao/commit/b404cde150eb767026122691aed2d21f9d6fd051) ([#995](https://github.com/tauri-apps/tao/pull/995) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) Add `WindowAttributes::background_color`, `WindowBuilder::with_background_color`, and `Window::set_background_color` APIs to set and change window background color.

## \[0.30.3]

- [`4dcd2312`](https://github.com/tauri-apps/tao/commit/4dcd231209db80a49b3403c01966026cee975b00) ([#979](https://github.com/tauri-apps/tao/pull/979) by [@Zamoca42](https://github.com/tauri-apps/tao/../../Zamoca42)) On Linux Wayland, changed the event handling for maximizing to process events sequentially to avoid "Error 71(Protocol error): dispatching to Wayland display".
- [`2ee007a5`](https://github.com/tauri-apps/tao/commit/2ee007a56a86980d329745eaf7b748fbf30edeac) ([#981](https://github.com/tauri-apps/tao/pull/981) by [@thep0y](https://github.com/tauri-apps/tao/../../thep0y)) Add `Window::is_always_on_top` method to check if a window is always on top on macOS, Linux and Windows.
- [`4dcd2312`](https://github.com/tauri-apps/tao/commit/4dcd231209db80a49b3403c01966026cee975b00) ([#979](https://github.com/tauri-apps/tao/pull/979) by [@Zamoca42](https://github.com/tauri-apps/tao/../../Zamoca42)) On Linux Wayland, fixed an issue where the window was not moving when dragging the header bar area.
- [`4dcd2312`](https://github.com/tauri-apps/tao/commit/4dcd231209db80a49b3403c01966026cee975b00) ([#979](https://github.com/tauri-apps/tao/pull/979) by [@Zamoca42](https://github.com/tauri-apps/tao/../../Zamoca42)) On Linux Wayland, fixed an issue where the window was not resizing when dragging the window borders.
- [`4dcd2312`](https://github.com/tauri-apps/tao/commit/4dcd231209db80a49b3403c01966026cee975b00) ([#979](https://github.com/tauri-apps/tao/pull/979) by [@Zamoca42](https://github.com/tauri-apps/tao/../../Zamoca42)) On Linux Wayland, added buttons for maximize and minimize in the title bar.
- [`2fffdc9d`](https://github.com/tauri-apps/tao/commit/2fffdc9db73edbd529f4c3cf889b33029cefd955) ([#983](https://github.com/tauri-apps/tao/pull/983) by [@Legend-Master](https://github.com/tauri-apps/tao/../../Legend-Master)) Fix blinking title bar when changing system settings on Windows

## \[0.30.2]

- [`016e122c`](https://github.com/tauri-apps/tao/commit/016e122c5f10eb61f8abe052a888950a460e0804) ([#978](https://github.com/tauri-apps/tao/pull/978) by [@Legend-Master](https://github.com/tauri-apps/tao/../../Legend-Master)) Fix changing the theme activates the window on Windows

## \[0.30.1]

- [`ad652e50`](https://github.com/tauri-apps/tao/commit/ad652e50bfca1195481cd347ccaa486818f9334d) ([#969](https://github.com/tauri-apps/tao/pull/969) by [@CampioneDev](https://github.com/tauri-apps/tao/../../CampioneDev)) On iOS, implement `application:openURL:options:` to handle custom URL schemes.
- [`1a085ade`](https://github.com/tauri-apps/tao/commit/1a085ade59dcebdd3a6da4e8a8433be4702fe997) ([#937](https://github.com/tauri-apps/tao/pull/937) by [@Legend-Master](https://github.com/tauri-apps/tao/../../Legend-Master)) Add a function `Window::set_theme` and `EventLoopWindowTarget::set_them`to set theme after window or event loop creation.

## \[0.30.0]

- [`222d5786`](https://github.com/tauri-apps/tao/commit/222d57862b24511eda733812524df1736cd1f64d) ([#971](https://github.com/tauri-apps/tao/pull/971) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) On Windows, fix `Window::monitor_from_point` and `EventLoopTargetWindow::monitor_from_point` returning invalid monitor handle.
- [`e47d4c4a`](https://github.com/tauri-apps/tao/commit/e47d4c4aa08cb1d0f431c6bdf8f81cc82ecc72d1) ([#967](https://github.com/tauri-apps/tao/pull/967) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) On Linux, removed internal check for current desktop environment before applying `Window::set_progress_bar` API. This should allow `Window::set_progress_bar` to work on KDE Plasma and similar environments that support `libunity` APIs.
- [`9b5aa60b`](https://github.com/tauri-apps/tao/commit/9b5aa60ba6f6e45ac3fc42dc715d7e071d29bb2b) ([#970](https://github.com/tauri-apps/tao/pull/970) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) Changed `WindowExtWindows::set_skip_taskbar` and `WindowExtUnix::set_skip_taskbar` to return a result instead of panicing internally.

## \[0.29.1]

- [`4cd53415`](https://github.com/tauri-apps/tao/commit/4cd534151a2d7a14ade906f960ec02655a91feae) ([#964](https://github.com/tauri-apps/tao/pull/964) by [@lucasfernog](https://github.com/tauri-apps/tao/../../lucasfernog)) Allow Android domain names to include `_1` as escaped `_` characters - required because `_` is the separator for domain parts.

## \[0.29.0]

- [`e67cf1b2`](https://github.com/tauri-apps/tao/commit/e67cf1b2826d32b8eb58f6d111271a1c42b62978) ([#941](https://github.com/tauri-apps/tao/pull/941) by [@Sanae6](https://github.com/tauri-apps/tao/../../Sanae6)) Prevent duplicate mouse press, release, and motion events from firing on Linux (fixes #939)
- [`b7dab732`](https://github.com/tauri-apps/tao/commit/b7dab732a9b8c71e7433c8f1b69e55c9c49f4e50) ([#947](https://github.com/tauri-apps/tao/pull/947) by [@muwoo](https://github.com/tauri-apps/tao/../../muwoo)) Fix `Window::request_user_attention` not taking effect after minimizing the window by clicking the taskbar icon
- [`f54cc11e`](https://github.com/tauri-apps/tao/commit/f54cc11e4441a706a276c05f0e65f48a69f779bd) ([#938](https://github.com/tauri-apps/tao/pull/938) by [@andrewbaxter](https://github.com/tauri-apps/tao/../../andrewbaxter)) Add `EventLoopWindowTargetExtUnix::gtk_app` getter.
- [\`\`](https://github.com/tauri-apps/tao/commit/undefined) Return a new `BadIcon::DimensionsZero` error variant in `Icon::from_rgba` if one of the passed icon dimensions is zero.
- [`80e10084`](https://github.com/tauri-apps/tao/commit/80e1008438d57b72921192ff44ffc252ab676edb) ([#954](https://github.com/tauri-apps/tao/pull/954) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) Return a new `BadIcon::DimensionsZero` error variant in `Icon::from_rgba` if one of the passed icon dimensions is zero.
- [`80e10084`](https://github.com/tauri-apps/tao/commit/80e1008438d57b72921192ff44ffc252ab676edb) ([#954](https://github.com/tauri-apps/tao/pull/954) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) Return a new `BadIcon::DimensionsMultiplyOverflow` error variant in `Icon::from_rgba` if dimensions multiplication overflowed.
- [`f5756196`](https://github.com/tauri-apps/tao/commit/f57561964cde6f3ff77c26ffec54b92116f49921) ([#956](https://github.com/tauri-apps/tao/pull/956) by [@MarijnS95](https://github.com/tauri-apps/tao/../../MarijnS95)) **Breaking change**: Upgrade `ndk` crate to `0.9` and `ndk-sys` crate to `0.6`.  Types from the `ndk` crate are used in public API surface.
  **Breaking change**: Change `NativeKeyCode::Android(u32)` type to use `i32`, which is the native type used by all Android API.
  **Breaking change**: The `setup` function passed to `android_binding!()` must now take a `&ThreadLooper` instead of `&ForeignLooper`, matching the `wry` change in https://github.com/tauri-apps/wry/pull/1296.
- [`f54cc11e`](https://github.com/tauri-apps/tao/commit/f54cc11e4441a706a276c05f0e65f48a69f779bd) ([#938](https://github.com/tauri-apps/tao/pull/938) by [@andrewbaxter](https://github.com/tauri-apps/tao/../../andrewbaxter)) Add `WindowExtUnix::new_from_gtk_window`.

## \[0.28.1]

- [`7e8f75e9`](https://github.com/tauri-apps/tao/commit/7e8f75e916ad502e00b5f6dff6cfb6bfadb92118) ([#926](https://github.com/tauri-apps/tao/pull/926) by [@pewsheen](https://github.com/tauri-apps/tao/../../pewsheen)) On macOS, add `set_fullsize_content_view` and `set_titlebar_transparent` to `Window` to set the title bar style.
- [`3bbddc64`](https://github.com/tauri-apps/tao/commit/3bbddc64d824df5b502f1c165e2b3cdf8a684886) ([#931](https://github.com/tauri-apps/tao/pull/931) by [@muwoo](https://github.com/tauri-apps/tao/../../muwoo)) **Breaking Change**: On Windows, `UserAttentionType::Informational` will flash the taskbar icon 4 times only and not until the app recieves focus.
- [`29bee151`](https://github.com/tauri-apps/tao/commit/29bee15103b6beeec617f103d475d2b707c35d83) ([#935](https://github.com/tauri-apps/tao/pull/935) by [@renovate](https://github.com/tauri-apps/tao/../../renovate)) Update `windows` crate to `0.57`
- [`ab792dbd`](https://github.com/tauri-apps/tao/commit/ab792dbd6c5f0a708c818b20eaff1d9a7534c7c1) ([#928](https://github.com/tauri-apps/tao/pull/928) by [@amrbashir](https://github.com/tauri-apps/tao/../../amrbashir)) On Windows, always allow dark theme for app and window through `AllowDarkModeForApp` and `AllowDarkModeForWindow`, which fixes an issue when calling `IsDarkModeAllowedForWindow()`

## \[0.28.0]

- [`cd38f237`](https://github.com/tauri-apps/tao/commit/cd38f237481b58585cc2da98c3bb9a0ce8e64b9f)([#912](https://github.com/tauri-apps/tao/pull/912)) Add `EventLoopBuilderExtUnix::with_app_id` on Linux to allow setting a unique app id for the underlying gtk application.
- [`2d6ad2de`](https://github.com/tauri-apps/tao/commit/2d6ad2ded633c33d05a35a69ef657600444004b9)([#923](https://github.com/tauri-apps/tao/pull/923)) Fix unwanted deviation when dragging window around monitor boundaries on macOS.
- [`f06843be`](https://github.com/tauri-apps/tao/commit/f06843be442c4ad06e66166d1a7924347739c204)([#517](https://github.com/tauri-apps/tao/pull/517)) Add `event::Reopen` for handle click on dock icon on macOS.

## \[0.27.1]

- [`9ef1379f`](https://github.com/tauri-apps/tao/commit/9ef1379fcc3e1bcd6ce54c11818bed52f0b9eb25)([#901](https://github.com/tauri-apps/tao/pull/901)) On macOS, fix an issue where the set minimize maximize and close buttons would interfere with each other.
- [`5af059b1`](https://github.com/tauri-apps/tao/commit/5af059b1e5d79661cb9338676f132a6f36446f05)([#909](https://github.com/tauri-apps/tao/pull/909)) Update `windows` crate to `0.56`

## \[0.27.0]

- [`c2357732`](https://github.com/tauri-apps/tao/commit/c23577325bd26e7e8eb46e17bbf533717e363e04)([#896](https://github.com/tauri-apps/tao/pull/896)) Replaced `dpi` module with a re-export of the `dpi` crate which has a few breaking changes:

  - Replaced `LogicalPixel` with `LogicalUnit`
  - Replaced `PhysicalPixel` with `PhysicalUnit`
  - Removed `Size::width`, `Size::height`, `Position::x`, `Position::y` and `PixelUnit::value`.

## \[0.26.2]

- [`17f54d40`](https://github.com/tauri-apps/tao/commit/17f54d402838e20c05c05a1cf3db9e0f76e7fa68)([#887](https://github.com/tauri-apps/tao/pull/887)) Update `windows` crate to `0.54`

## \[0.26.1]

### bug

- [`f2ffb501`](https://github.com/tauri-apps/tao/commit/f2ffb501ebd6f456987f80791629893e37a5b31e)([#884](https://github.com/tauri-apps/tao/pull/884)) Fixed compile errors for Linux Arm64 targets.

## \[0.26.0]

- [`2af91313`](https://github.com/tauri-apps/tao/commit/2af91313b2e8c48ef3330568ca9da25150a7bdaa)([#880](https://github.com/tauri-apps/tao/pull/880)) Updated the minimum supported Rust version to 1.70.
- [`90ad07b3`](https://github.com/tauri-apps/tao/commit/90ad07b324636d27cae267ad52751fb886aa92a0)([#878](https://github.com/tauri-apps/tao/pull/878)) **Breaking change**: Removed `window::hit_test` function.
- [`2af91313`](https://github.com/tauri-apps/tao/commit/2af91313b2e8c48ef3330568ca9da25150a7bdaa)([#880](https://github.com/tauri-apps/tao/pull/880)) Progress bar on Linux no longer relies on zbus. Changed  `ProgressBarState`'s field `unity_uri` to `desktop_filename`.
- [`90ad07b3`](https://github.com/tauri-apps/tao/commit/90ad07b324636d27cae267ad52751fb886aa92a0)([#878](https://github.com/tauri-apps/tao/pull/878)) On Windows and Linux, disable resizing undecorated windows when in fullscreen.
- [`90ad07b3`](https://github.com/tauri-apps/tao/commit/90ad07b324636d27cae267ad52751fb886aa92a0)([#878](https://github.com/tauri-apps/tao/pull/878)) On Windows, fix undecorated window resizing.
- [`89ce9d26`](https://github.com/tauri-apps/tao/commit/89ce9d26c3ac2483f93a386451b5a197cbfb684c)([#874](https://github.com/tauri-apps/tao/pull/874)) On Windows, apply `ScaleFactorChanged` if new size is different than what OS reported. This fixes an issue when moving the window to another monitor and immediately maximizing it, resulting in a maximized window (i.e have `WS_MAXIMIZE` window style) but doesn't cover the monitor work area.

## \[0.25.0]

- [`ae4b693d`](https://github.com/tauri-apps/tao/commit/ae4b693dc0a5d4f556bb9e6dcdbbeb1cfbf8f862)([#864](https://github.com/tauri-apps/tao/pull/864)) On Windows, Remove `WS_CAPTION` and `WS_EX_WINDOWEDGE` window styles when creating a child window.
- [`e10f6a68`](https://github.com/tauri-apps/tao/commit/e10f6a68287553c4bb8a62b71ee62dd543918681)([#862](https://github.com/tauri-apps/tao/pull/862)) **Breaking Change**: Changed `WindowBuilderExtUnix::with_transient_for` signature to take `&impl gtk::glib::IsA<gtk::Window>` instead of `gtk::ApplicationWindow` which covers more gtk window types and matches the underlying API call signature.

## \[0.24.1]

- [`25a8836b`](https://github.com/tauri-apps/tao/commit/25a8836b6493d1873d7c263bb5603c2a4e3364a1)([#860](https://github.com/tauri-apps/tao/pull/860)) Fix the app crash on restart due to Android context was not released. Release the Android context when the app is destroyed to avoid assertion failure.
- [`5eb2124e`](https://github.com/tauri-apps/tao/commit/5eb2124e706b57f236a42503d530f9671cd0a2cf)([#852](https://github.com/tauri-apps/tao/pull/852)) Enable macOS secure state restoration on OS versions that support it. This avoids
  'WARNING: Secure coding is not enabled for restorable state!' on macOS Sonoma.
- [`f0bf850f`](https://github.com/tauri-apps/tao/commit/f0bf850fee4dddb045522257aa53719b3136f4ff)([#859](https://github.com/tauri-apps/tao/pull/859)) Derive `Debug, Copy, Clone, PartialEq, Eq, Hash` for `ResizeDirection`.
- [`29b01bff`](https://github.com/tauri-apps/tao/commit/29b01bff2b547ca5b6bba57584c9c7cc2b93c098)([#849](https://github.com/tauri-apps/tao/pull/849)) On Windows, remove `SetWindowTheme` call with `DarkMode_Explorer` theme which fixes a glitch downstream in `muda` crate when manually drawing the menu bar.
- [`60bbcac1`](https://github.com/tauri-apps/tao/commit/60bbcac168887eecbbddeb05d479347f9c9478d5)([#858](https://github.com/tauri-apps/tao/pull/858)) On Windows, fix when the `Show window contents while dragging` setting is turned off in Windows, there is a window size issue when dragging between multi-monitors with different scaling.
- [`68803e67`](https://github.com/tauri-apps/tao/commit/68803e67480cfdc2e57cb13647328cd6539ce6fc)([#854](https://github.com/tauri-apps/tao/pull/854)) On Windows, fix consecutive calls to `window.set_fullscreen(Some(Fullscreen::Borderless(None)))` resulting in losing previous window state when eventually exiting fullscreen using `window.set_fullscreen(None)`.

## \[0.24.0]

- [`43c94f0b`](https://github.com/tauri-apps/tao/commit/43c94f0b2021d3831846c53bfd268cdda2c87b51)([#830](https://github.com/tauri-apps/tao/pull/830)) This patch contains a couple of changes to how the anroid macros:

  - Changed `android_binding` macro 4th argument signature, which is a setup function that is called once when the event loop is first created, from `unsafe fn(JNIEnv, &ForeignLooper, GlobalRef)` to `unsafe fn(&str, JNIEnv, &ForeignLooper, GlobalRef)`.
  - Moved `android_fn!` and `generate_package_name` macro from crate root `platform::android::prelude`
- [`f497b5dc`](https://github.com/tauri-apps/tao/commit/f497b5dc828c202376f37ed835d5fd48b1a93530)([#829](https://github.com/tauri-apps/tao/pull/829)) Add `Window::drag_resize_window` and `ResizeDirection` enum to initialize window resizing. Supported on Windows and Linux only.
- [`28b53f80`](https://github.com/tauri-apps/tao/commit/28b53f80c49bbf2ae8902b98a2e28f6451a5a8f1)([#705](https://github.com/tauri-apps/tao/pull/705)) Fix `Window::primary_monitor` panicking on Linux when there is no primary monitor, e.g. with Wayland.
- [`e33104c2`](https://github.com/tauri-apps/tao/commit/e33104c2cf06fd1fcabba89332a48a06b975929e)([#831](https://github.com/tauri-apps/tao/pull/831)) On macOS, fix `WindowEvent::Destroyed` may fire twice.
- [`853101be`](https://github.com/tauri-apps/tao/commit/853101bea465098e56756aa700c4d50f503ee95a)([#821](https://github.com/tauri-apps/tao/pull/821)) This release includes an update to `raw-window-handle` crate to `0.6` but will also provide a feature flags to select which `raw-window-handle` to use:

  - `rwh_06` (default): `raw-window-handle@0.6`
  - ` rwh_05:  `raw-window-handle@0.5\`
  - ` rwh_04:  `raw-window-handle@0.4\`
- [`fce9d260`](https://github.com/tauri-apps/tao/commit/fce9d2603e55f2b37ff69acfbbd484de6298351e)([#844](https://github.com/tauri-apps/tao/pull/844)) On Windows, fix `WindowBuilder::with_theme` has no effect when forcing light theme on a dark mode system.
- [`c0278d83`](https://github.com/tauri-apps/tao/commit/c0278d83f93d8ee18d1e9eeee88e71d84c12357a)([#839](https://github.com/tauri-apps/tao/pull/839)) On Windows, remove `WS_CLIPCHILDREN` from window style

## \[0.23.0]

- [`cf22c902`](https://github.com/tauri-apps/tao/commit/cf22c902d4c961be0f6cfba6a8c865e11073b027)([#85](https://github.com/tauri-apps/tao/pull/85)) **Breaking change**: Removed clipboard implementation. Use `arboard` crate instead.
- [`081ba16a`](https://github.com/tauri-apps/tao/commit/081ba16a39066039dea9e5ee3f223056629ed28f)([#800](https://github.com/tauri-apps/tao/pull/800)) Fix `Window::theme` may return a theme different from the actual window's theme on Linux.
- [`32ce759e`](https://github.com/tauri-apps/tao/commit/32ce759e4e2eb8c8cfd67a538f9a46c11f4f91dd)([#801](https://github.com/tauri-apps/tao/pull/801)) Updated to gtk 0.18 and Bump MSRV to 1.70.0.
- [`f569bbab`](https://github.com/tauri-apps/tao/commit/f569bbabda0af38595320fc64f8e645cde1bb9ef)([#815](https://github.com/tauri-apps/tao/pull/815)) Fix `Window::current_monitor` sometimes panicking on Linux when the window is invisible.
- [`7e854cb1`](https://github.com/tauri-apps/tao/commit/7e854cb1f5206b63cdd4a08c17bb35be58736e43)([#817](https://github.com/tauri-apps/tao/pull/817)) On Windows, fix incorrect delta reported for `DeviceEvent::MouseWheel` event.
- [`7e854cb1`](https://github.com/tauri-apps/tao/commit/7e854cb1f5206b63cdd4a08c17bb35be58736e43)([#817](https://github.com/tauri-apps/tao/pull/817)) On Windows, fix `Window::set_progress_bar` incorrect states.
- [`7e854cb1`](https://github.com/tauri-apps/tao/commit/7e854cb1f5206b63cdd4a08c17bb35be58736e43)([#817](https://github.com/tauri-apps/tao/pull/817)) Update `windows` and `windows-implement` crate to `0.51`

## \[0.22.3]

- [`dabfed7d`](https://github.com/tauri-apps/tao/commit/dabfed7dc34eb60c796561961103e768f1d53689)([#802](https://github.com/tauri-apps/tao/pull/802)) Fixes set size APIs crashing on Linux.

## \[0.22.2]

- [`65ebab88`](https://github.com/tauri-apps/tao/commit/65ebab888de2d1ae4b7562572060d1e51ef85043)([#787](https://github.com/tauri-apps/tao/pull/787)) Fix compilation error on iOS.

## \[0.22.1]

- [`6df56c2d`](https://github.com/tauri-apps/tao/commit/6df56c2dd1b0f4695dd40b70f75a22d1ba32c384)([#751](https://github.com/tauri-apps/tao/pull/751)) On Windows, apply dark mode app-wide to some controls like context menus.
- [`6df56c2d`](https://github.com/tauri-apps/tao/commit/6df56c2dd1b0f4695dd40b70f75a22d1ba32c384)([#751](https://github.com/tauri-apps/tao/pull/751)) On Windows, add `EventLoopBuilderExtWindows::with_theme` to control the app-wide theme.

## \[0.22.0]

- [`06b617ea`](https://github.com/tauri-apps/tao/commit/06b617eaa2cea7039e9b71b8322b517486e3b1e5)([#776](https://github.com/tauri-apps/tao/pull/776)) Update jni to 0.21.
- [`d0b20c94`](https://github.com/tauri-apps/tao/commit/d0b20c94eaf555ba27f3cfbbf2636e3f3b036a97)([#778](https://github.com/tauri-apps/tao/pull/778)) This release contains a number of **breaking changes** that aimed at removing menus, system-tray and global-shortcuts features which have been moved to different crates, [`muda`](https://github.com/tauri-apps/muda/), [`tray-icon`](https://github.com/tauri-apps/tray-icon/) and [`global-hotkey`](https://github.com/tauri-apps/global-hotkey) and here is a summary of the changes:

  - Removed `tray` crago feature flag.
  - Removed `accelerator`, `menu`, `system_tray` and `global_shortcut` modules and all associated types.
  - Removed `Event::MenuEvent`, `Event::TrayEvent`, `Event::GlobalShortcutEvent`, `TrayEvent` and `Rectangle` types.
  - Added `EventLoopBuilder` type.
  - Removed `EventLoop::with_user_event`, instead use `EventLoopBuilder::<T>::with_user_event().build()`.
  - Removed `EventLoopExtWindows`, `EventLoopExtMacOS` and `EventLoopExtUnix`, instead use `EventLoopBuilderExtWindows`, `EventLoopBuilderExtMacOS` and `EventLoopBuilderExtUnix`.
  - Changed `WindowExtWindows::hinstance`, `WindowExtWindows::hwnd` and `MonitorHandleExtWindow::hmonitor` to return `isize` instead of `*const c_void`

## \[0.21.1]

- [`9a320882`](https://github.com/tauri-apps/tao/commit/9a320882ed824d18f9e20f8a9af7a97f51805c87)([#761](https://github.com/tauri-apps/tao/pull/761)) On Android, use a lockfree queue (crossbeam channel) to prevent deadlocks inside send_event.
- [`b31cb692`](https://github.com/tauri-apps/tao/commit/b31cb692df2b0a03d2fbdf2fbf7ba82591678e24)([#772](https://github.com/tauri-apps/tao/pull/772)) On macOS, fix `WindowExtMacOS::ns_view` returning an invalid pointer if the view was replaced by a call to `setContentView` later on.
- [`4d0e1862`](https://github.com/tauri-apps/tao/commit/4d0e1862b6a2a7580631d637ef937d217f0797bf)([#762](https://github.com/tauri-apps/tao/pull/762)) Add `WindowExtWindows::set_rtl` and `WindowBuilderExtWindows::with_rtl` to set right-to-left layout on Windows.
- [`75eb0c1e`](https://github.com/tauri-apps/tao/commit/75eb0c1e7e83a766af0e083ce09c761d1974cde4)([#769](https://github.com/tauri-apps/tao/pull/769)) Add `WindowBuilderExtWindows::with_window_classname` to set the name of the window class created/used to create windows.
- [`494e4585`](https://github.com/tauri-apps/tao/commit/494e4585d1177b12bdaacbb3aa381d0514f5252f)([#775](https://github.com/tauri-apps/tao/pull/775)) Ensure the macOS app delegate is defined before accessing it.

## \[0.21.0]

- [`81329013`](https://github.com/tauri-apps/tao/commit/813290130ea255b2cb45a66234a422519d13f667)([#743](https://github.com/tauri-apps/tao/pull/743)) On macOS, fix the unexpected shifting of the window when dragging after closing the share dialog.
- [`baa02977`](https://github.com/tauri-apps/tao/commit/baa02977483c9da21451d65bde0a64230778a034)([#418](https://github.com/tauri-apps/tao/pull/418)) Added APIs for setting progress bars for the application icon on Linux (Unity only) and macOS, along with progress indicator for specific window on Windows.
- [`8f361f0c`](https://github.com/tauri-apps/tao/commit/8f361f0c19014e6ef647fe7fa8adc47718796984)([#752](https://github.com/tauri-apps/tao/pull/752)) Handle universal links on iOS and send `Event::Opened { urls }`.
- [`bb3c53d1`](https://github.com/tauri-apps/tao/commit/bb3c53d1d84ebc26b0d66230877ecc7b6a71db27)([#764](https://github.com/tauri-apps/tao/pull/764)) On macOS, fix `SystemTrayEvent` not emitted after calling `set_menu`.
- [`5af3da4a`](https://github.com/tauri-apps/tao/commit/5af3da4a2cfa0e648dfa87c067deb6745b73bcc8)([#746](https://github.com/tauri-apps/tao/pull/746)) On macOS, force `NativeImage` height to be `18` to have consistent size for all icons and match custom icons.
- [`093d8fbc`](https://github.com/tauri-apps/tao/commit/093d8fbc20954d51c96c94618479ff465ca55888)([#422](https://github.com/tauri-apps/tao/pull/422)) Implement `Event::Opened` on macOS for file association and deeplink support.
- [`e9875fe5`](https://github.com/tauri-apps/tao/commit/e9875fe54e1e8ae1db6e28eedeed2e3e51f8226b)([#755](https://github.com/tauri-apps/tao/pull/755)) On Windows, fix leak of `tao::system_tray::Icon` when calling `tao::system_tray::SystemTray::set_icon` and leak of `String` when calling `tao::system_tray::SystemTray::set_tooltip`.
- [`50e69d71`](https://github.com/tauri-apps/tao/commit/50e69d718e9c71d044ebc3535ca58a992db18547)([#749](https://github.com/tauri-apps/tao/pull/749)) On Windows, fix disabling `resizable` also disabling maximize button and messing up `Window::set_maximized`.

## \[0.20.0]

- [`c6082173`](https://github.com/tauri-apps/tao/commit/c6082173a943e23653783fd0872f64c66bf96de9)([#731](https://github.com/tauri-apps/tao/pull/731)) Fix build error on target i686-pc-windows-msvc
- [`90ce80cd`](https://github.com/tauri-apps/tao/commit/90ce80cd4dc8babb5e5fab21fb783c710340b923)([#732](https://github.com/tauri-apps/tao/pull/732)) Enable shadows by default for undecorated window on Windows.

## \[0.19.1]

- On Windows, fix auto-hide taskbar can't be shown when maximizing undecorated window.
  - [c5d606df](https://github.com/tauri-apps/tao/commit/c5d606dffeb1733ab06fd8c43eb3b9e7b2f553fe) fix(windows): leave space for auto-hidden taskbar for undecorated windows ([#726](https://github.com/tauri-apps/tao/pull/726)) on 2023-04-19
- On Linux, fix `ShortcutManager::unregister_all` making `ShortcutManager::register` succeed but no events are triggered.
  - [ee5dc41f](https://github.com/tauri-apps/tao/commit/ee5dc41f0071c9177304b3697d5b4c21c5734fd4) fix(linux): clear shortcuts instead of replacing it ([#724](https://github.com/tauri-apps/tao/pull/724)) on 2023-04-18
- On macOS, fix window frozed when starting with fullscreen.
  - [71594667](https://github.com/tauri-apps/tao/commit/71594667432c554f46dad06bfce87ba8edf18605) fix(macOS): windows frozen when starting in fullscreen ([#727](https://github.com/tauri-apps/tao/pull/727)) on 2023-05-04

## \[0.19.0]

- **Breaking change**: All ow specifying the android activity in `android_binding` macro, instead of hard-coded `TauriActivity`.
  - [b78b9616](https://github.com/tauri-apps/tao/commit/b78b961621e9cde355c8de2cb9fc168efcae4313) feat!: allow specifying android activity in binding macro ([#723](https://github.com/tauri-apps/tao/pull/723)) on 2023-04-14
- Fix set_focus not working on Windows in some situations like interactive notifications.
  - [62db4313](https://github.com/tauri-apps/tao/commit/62db431338636dd0adafca15d636c08f68941984) fix(windows): Use SetForegroundWindow before focus hack ([#719](https://github.com/tauri-apps/tao/pull/719)) on 2023-04-04

## \[0.18.3]

- On macOS, fix wry window will crash if unfocused.
  - [6a03847f](https://github.com/tauri-apps/tao/commit/6a03847f6d1343315174c9cd41b58a4cd2798657) On macOS, fix wry window can crash if unfocused ([#714](https://github.com/tauri-apps/tao/pull/714)) on 2023-03-24

## \[0.18.2]

- fix not get actual ns_view when it's replace by setContentView
  - [76ae625b](https://github.com/tauri-apps/tao/commit/76ae625bae429148f295fc3eaf3e90984cf0c7ad) fix: not get actual ns_view when it's replace by setContentView ([#710](https://github.com/tauri-apps/tao/pull/710)) on 2023-03-07
- Fix `Window::cursor_position` and `EventLoopWindowTarget::cursor_position` scale on Linux and macOS.
  - [dc913cd5](https://github.com/tauri-apps/tao/commit/dc913cd5fe72f098a3545288eeed9b68f5e320ef) fix: scale cursor_position ([#712](https://github.com/tauri-apps/tao/pull/712)) on 2023-03-08
- On macOS, Fix `cursor_position` return incorrect position.
  - [ea2e60d9](https://github.com/tauri-apps/tao/commit/ea2e60d9df719a3abc335dc22078f72bbff1d3ef) fix(macOS): `cursor_position` returns incorrect position ([#711](https://github.com/tauri-apps/tao/pull/711)) on 2023-03-07
- Fix arrow cursor icon on Linux
  - [e9eba855](https://github.com/tauri-apps/tao/commit/e9eba8555b2bff1080d75d3386ce990c04576cde) chore: rename change file on 2023-02-22
- Attempt to get primary monitor on linux will now return None rather than panicking if monitor not found.
  - [28b53f80](https://github.com/tauri-apps/tao/commit/28b53f80c49bbf2ae8902b98a2e28f6451a5a8f1) fix: don't panic if primary monitor not discoverable. ([#705](https://github.com/tauri-apps/tao/pull/705)) on 2023-02-22
- On macOS, Remove linking to `ColorSync`
  - [a1e96d1b](https://github.com/tauri-apps/tao/commit/a1e96d1b1284a76576da0de7cba4730df9776bb5) feat: remove linking to `ColorSync` ([#713](https://github.com/tauri-apps/tao/pull/713)) on 2023-03-15

## \[0.18.1]

- Retain NSMenu reference instead of autoreleasing it.
  - [5c37a54a](https://github.com/tauri-apps/tao/commit/5c37a54ab577e74730052658c9ed2e9b85462be8) fix(macos): retain and release NSMenu manually ([#699](https://github.com/tauri-apps/tao/pull/699)) on 2023-02-20

## \[0.18.0]

- Fix undecorated window shadow enabled by default on Windows.
  - [1011b688](https://github.com/tauri-apps/tao/commit/1011b688ab67ffe898e24a3fa9c4566e91ab7359) fix(windows): default undecorated shadow to false ([#689](https://github.com/tauri-apps/tao/pull/689)) on 2023-02-07
- On Linux, Add wayland raw handle methods. (#685)
  - [3ce71295](https://github.com/tauri-apps/tao/commit/3ce71295c1a55134e77bb461c0fd49347b782403) Add missing change file in [#685](https://github.com/tauri-apps/tao/pull/685) on 2023-02-04
- Update `windows-rs` to `0.44` which bumps the MSRV of this crate on Windows to `1.64`.
  - [8971d731](https://github.com/tauri-apps/tao/commit/8971d731b02ec61a1665351c9bae11f5e4058dc4) chore(deps): update to windows-rs 0.44 ([#687](https://github.com/tauri-apps/tao/pull/687)) on 2023-02-06

## \[0.17.0]

- Bump gtk version: 0.15 -> 0.16
  - [b59f1b49](https://github.com/tauri-apps/tao/commit/b59f1b4922533c7b3bd14516de8da1c6467061ea) Bump gtk version 0.15 -> 0.16 ([#679](https://github.com/tauri-apps/tao/pull/679)) on 2023-01-26
- Add `Window::cursor_position` and `EventLoopWindowTarget::cursor_position` to get the current mouse position.
  - [5d8bf51d](https://github.com/tauri-apps/tao/commit/5d8bf51d7ea2ea89feb4e8b003e583fed69bf300) feat: add `cursor_position` ([#668](https://github.com/tauri-apps/tao/pull/668)) on 2023-01-12
- On Linux, spawn device event thread only once instead of a new thread on each iteration of the event loop.
  - [ca1ed5de](https://github.com/tauri-apps/tao/commit/ca1ed5decf516dc674b5e26815aa98346fca11ff) fix(linux): spawn device thread only once ([#678](https://github.com/tauri-apps/tao/pull/678)) on 2023-01-23
- On Windows, fix `Window::set_minimized(false)` not working when the window was minimized using `Win + D` hotkey.
  - [e1149563](https://github.com/tauri-apps/tao/commit/e1149563b85eb6187f5aa78d53cab9c5d7b87025) fix(Windows): fix `set_minimized` with `Win + D` ([#676](https://github.com/tauri-apps/tao/pull/676)) on 2023-01-21

## \[0.16.0]

- Yanked `0.15.9` and publish a new minor as `0.15.9` included breaking changes by depending on `tao-macros`.
  - [5397b8f6](https://github.com/tauri-apps/tao/commit/5397b8f6177f01418f1e56b60c9777395607278c) chore: bump minor on 2023-01-11

## \[0.15.9]

- On Linux, Fix mnemonics for submenus.
  - [77569c89](https://github.com/tauri-apps/tao/commit/77569c893f9835717a79bf445fa3a7f433e0fb3f) fix(linux): fix mnemonics for submenus ([#650](https://github.com/tauri-apps/tao/pull/650)) on 2022-12-20
  - [e313ef69](https://github.com/tauri-apps/tao/commit/e313ef69d2b3849fa7f5d43effad7c1e76c73748) publish new versions ([#651](https://github.com/tauri-apps/tao/pull/651)) on 2023-01-09
  - [3cd851d1](https://github.com/tauri-apps/tao/commit/3cd851d14126c305964b957eeb4f9ed0011d96cb) Revert "Publish New Versions" ([#663](https://github.com/tauri-apps/tao/pull/663)) on 2023-01-09
- On iOS, add Sync trait to `EventLoopProxy` when `T` has Send trait.
  - [651137ce](https://github.com/tauri-apps/tao/commit/651137ce9ec5bf37593e6641d8f6ab79fc9d6f3c) On iOS, add Sync trait on `EventLoopProxy` when `T` has Send trait ([#658](https://github.com/tauri-apps/tao/pull/658)) on 2023-01-04
  - [e313ef69](https://github.com/tauri-apps/tao/commit/e313ef69d2b3849fa7f5d43effad7c1e76c73748) publish new versions ([#651](https://github.com/tauri-apps/tao/pull/651)) on 2023-01-09
  - [3cd851d1](https://github.com/tauri-apps/tao/commit/3cd851d14126c305964b957eeb4f9ed0011d96cb) Revert "Publish New Versions" ([#663](https://github.com/tauri-apps/tao/pull/663)) on 2023-01-09
- On Linux, fix setting min/max size clears the other.
  - [9927c3a5](https://github.com/tauri-apps/tao/commit/9927c3a5815bbc581d35ca7dc4e7ded834ef5f51) fix(linux): fix setting min/max size, clears the other ([#669](https://github.com/tauri-apps/tao/pull/669)) on 2023-01-11
- Fix resize event emits before fullscreen actually exit.
  - [3867e7b7](https://github.com/tauri-apps/tao/commit/3867e7b783cd0d1bf00ce81214cbfe53354466cd) On macOS, fix resize event emits before fullscreen actually exit ([#662](https://github.com/tauri-apps/tao/pull/662)) on 2023-01-09
  - [e313ef69](https://github.com/tauri-apps/tao/commit/e313ef69d2b3849fa7f5d43effad7c1e76c73748) publish new versions ([#651](https://github.com/tauri-apps/tao/pull/651)) on 2023-01-09
  - [3cd851d1](https://github.com/tauri-apps/tao/commit/3cd851d14126c305964b957eeb4f9ed0011d96cb) Revert "Publish New Versions" ([#663](https://github.com/tauri-apps/tao/pull/663)) on 2023-01-09
- Add `WindowBuilder::with_visible_on_all_workspaces` and `Window::set_visible_on_all_workspaces`.
  - [0aa2176c](https://github.com/tauri-apps/tao/commit/0aa2176c31fbc76cdc7746601c9bc3d0da449a88) feat: add `set_visible_on_all_workspaces`, closes [#185](https://github.com/tauri-apps/tao/pull/185) ([#666](https://github.com/tauri-apps/tao/pull/666)) on 2023-01-11
- Add `WindowExtWindows::set_undecorated_shadow` and `WindowBuilderExtWindows::with_undecorated_shadow` to draw the drop shadow behind a borderless window.
  - [f832ca99](https://github.com/tauri-apps/tao/commit/f832ca99abbaf93e6ce67a87da710b4f4efe6e6e) feat(Windows): undecorated shadows ([#664](https://github.com/tauri-apps/tao/pull/664)) on 2023-01-10

## \[0.15.8]

- Add `with_cursor_moved` Unix extension trait method.
  - [8c6b2d05](https://github.com/tauri-apps/tao/commit/8c6b2d05ae55cefe72bd63d1adfeca6c20058879) Add `with_cursor_moved` unix extension method ([#644](https://github.com/tauri-apps/tao/pull/644)) on 2022-12-14

## \[0.15.7]

- On Linux, fix menu item mnemonics.
  - [86a439ed](https://github.com/tauri-apps/tao/commit/86a439edc5da2bd1baa1067831dde2408fd14fbf) fix: fix menu mnemonics ([#640](https://github.com/tauri-apps/tao/pull/640)) on 2022-12-08
  - [e623efdc](https://github.com/tauri-apps/tao/commit/e623efdc9ab797b3d9e104f34ba5bc1a4648b32c) publish new versions ([#639](https://github.com/tauri-apps/tao/pull/639)) on 2022-12-10
  - [bdce0a4c](https://github.com/tauri-apps/tao/commit/bdce0a4c816bb63b9e52114924ee4a66d353a019) Revert "publish new versions ([#639](https://github.com/tauri-apps/tao/pull/639))" on 2022-12-10
- On Windows, retain `WS_MAXIMIZE` window style when unminimizing a maximized window.
  - [ca844a2e](https://github.com/tauri-apps/tao/commit/ca844a2ebb171f676962bd0bebd65243b9239347) fix(Windows): retain `WS_MAXIMIZE` when unminimizing a maximized window, closes [#622](https://github.com/tauri-apps/tao/pull/622) ([#638](https://github.com/tauri-apps/tao/pull/638)) on 2022-12-04
  - [e623efdc](https://github.com/tauri-apps/tao/commit/e623efdc9ab797b3d9e104f34ba5bc1a4648b32c) publish new versions ([#639](https://github.com/tauri-apps/tao/pull/639)) on 2022-12-10
  - [bdce0a4c](https://github.com/tauri-apps/tao/commit/bdce0a4c816bb63b9e52114924ee4a66d353a019) Revert "publish new versions ([#639](https://github.com/tauri-apps/tao/pull/639))" on 2022-12-10
- On macOS, strip menu mnemonics for consistency with other platforms.
  - [86a439ed](https://github.com/tauri-apps/tao/commit/86a439edc5da2bd1baa1067831dde2408fd14fbf) fix: fix menu mnemonics ([#640](https://github.com/tauri-apps/tao/pull/640)) on 2022-12-08
  - [e623efdc](https://github.com/tauri-apps/tao/commit/e623efdc9ab797b3d9e104f34ba5bc1a4648b32c) publish new versions ([#639](https://github.com/tauri-apps/tao/pull/639)) on 2022-12-10
  - [bdce0a4c](https://github.com/tauri-apps/tao/commit/bdce0a4c816bb63b9e52114924ee4a66d353a019) Revert "publish new versions ([#639](https://github.com/tauri-apps/tao/pull/639))" on 2022-12-10

## \[0.15.6]

- Revert `nextResponder` call because this will bring key beep sound regression. We'll call the key equivalent in wry instead.
  - [a59b69b2](https://github.com/tauri-apps/tao/commit/a59b69b2733b273d86dc200ba90065be3db871a6) On macOS, revert nextResponder calls ([#628](https://github.com/tauri-apps/tao/pull/628)) on 2022-11-21

## \[0.15.5]

- Change `WebviewAttributes::focused` default to `true`.
  - [ece3e8f6](https://github.com/tauri-apps/tao/commit/ece3e8f6a34de21ec4c19944f668edd47ecc8ce0) fix: default `focused` to true on 2022-11-20
- On Linux, wake the main context in `EventLoopProxy::send_event()`.
  - [b7b5f04d](https://github.com/tauri-apps/tao/commit/b7b5f04d4b4c2f58146aca1b7e03223cdae74f7c) Gtk: wake the main context in EventLoopProxy::send_event(), closes [#625](https://github.com/tauri-apps/tao/pull/625) ([#626](https://github.com/tauri-apps/tao/pull/626)) on 2022-11-16

## \[0.15.4]

- On macOS, call next responder in view's keyDown and doCommandbySelector.
  - [516e5fcd](https://github.com/tauri-apps/tao/commit/516e5fcd50de601330f3434ecd00bf5889f1a5cc) On macOS, remove `doCommandBySelector` in view ([#620](https://github.com/tauri-apps/tao/pull/620)) on 2022-11-09
  - [e9d6dadb](https://github.com/tauri-apps/tao/commit/e9d6dadb59fd8d5d32704a5d80d8d587f5d581ca) Publish New Versions ([#621](https://github.com/tauri-apps/tao/pull/621)) on 2022-11-09
  - [045b768e](https://github.com/tauri-apps/tao/commit/045b768e30b4dc261edcaba4b8ed8ec9fee8305e) On macOS, call next responder in view's keyDown and doCommandbySelector ([#623](https://github.com/tauri-apps/tao/pull/623)) on 2022-11-14

## \[0.15.3]

- On macOS, remove `doCommandBySelector` in view since this will block the key event to responder chain.
  - [516e5fcd](https://github.com/tauri-apps/tao/commit/516e5fcd50de601330f3434ecd00bf5889f1a5cc) On macOS, remove `doCommandBySelector` in view ([#620](https://github.com/tauri-apps/tao/pull/620)) on 2022-11-09

## \[0.15.2]

- On Windows, fix compliation regression introduced in 0.15.1 when `tray` feature is active
  - [081664dc](https://github.com/tauri-apps/tao/commit/081664dc6b12c7765b667072dfbfbc089e50c5a3) fix(Windows): fix build regression when tray feature is used ([#618](https://github.com/tauri-apps/tao/pull/618)) on 2022-11-09

## \[0.15.1]

- On Windows, fix window always visible initially.
  - [ae06c3e2](https://github.com/tauri-apps/tao/commit/ae06c3e2806b85a9baa10b84c898cd0c15af7de4) fix(Windows): fix windows always visible initially on 2022-11-08

## \[0.15.0]

- Add support for parsing `ArrowUp`, `ArrowDown`, `ArrowLeft` and `ArrowRight` in a str as valid key. Previously only `Up`, `Down`, `Left` and `Right` worked.
  - [5e85dbef](https://github.com/tauri-apps/tao/commit/5e85dbef325fd8050b5cc26079627a3b645285c7) fix: parse `Arrow*` in a accelerator string ([#609](https://github.com/tauri-apps/tao/pull/609)) on 2022-10-31
- Add `WindowBuilder::with_content_protection`.
  - [8084c800](https://github.com/tauri-apps/tao/commit/8084c8001c2b8aa00abe7305765c19ce4e8ffc66) feat: add `WindowBuilder::with_content_protection` ([#605](https://github.com/tauri-apps/tao/pull/605)) on 2022-10-30
- On macOS, fix default cursor always being arrow cursor
  - [1359fccf](https://github.com/tauri-apps/tao/commit/1359fccfe5b95de9a28afb92e9ac3adfc331fb3c) On macOS, fix default cursor always being arrow cursor ([#614](https://github.com/tauri-apps/tao/pull/614)) on 2022-11-06
- On Windows, fixed focus event emission on minimize.
  - [37bca310](https://github.com/tauri-apps/tao/commit/37bca310be977f8922eb13e35d0e53b925a6039d) fix(windows): fix focus event emission on minimize ([#559](https://github.com/tauri-apps/tao/pull/559)) on 2022-09-21
- Update jni to 0.20.
  - [38fef108](https://github.com/tauri-apps/tao/commit/38fef1087d217874aee016e237b6c15eedbd0250) feat(android): update to jni 0.20 ([#610](https://github.com/tauri-apps/tao/pull/610)) on 2022-10-31
- On Linux, add DeviceEvent::Key.
  - [775974d7](https://github.com/tauri-apps/tao/commit/775974d7084185b80afe76c5acf3727687b0fd02) feat(linux): add DeviceEvent::Key ([#600](https://github.com/tauri-apps/tao/pull/600)) on 2022-10-21
- fix(linux): Improve event loop process on Linux a bit. This changes only a few check and should make dragging windows on egui smoother.
  - [b529eec9](https://github.com/tauri-apps/tao/commit/b529eec9ba32d8d3195e1866bdb26b93beba7e13) fix(linux): improve event loop process on Linux ([#587](https://github.com/tauri-apps/tao/pull/587)) on 2022-10-12
- Fix inverted delta in `WindowEvent::MouseWheel` on Linux
  - [8451f754](https://github.com/tauri-apps/tao/commit/8451f754a97ddfc5f9cb3d81ed3ea4ae38c173f3) fix: Inverse mouse scroll wheel on Linux ([#585](https://github.com/tauri-apps/tao/pull/585)) on 2022-10-11
- Add `EventLoopExtMacOS::set_activate_ignoring_other_apps` on macOS.
  - [d2c6a91c](https://github.com/tauri-apps/tao/commit/d2c6a91c4588bab460ef5924b10cfb224c1336c6) feat: add `EventLoopExtMacOS::set_activate_ignoring_other_apps` ([#612](https://github.com/tauri-apps/tao/pull/612)) on 2022-11-01
- Add `WindowExtMacOS::set_allows_automatic_window_tabbing`, `WindowExtMacOS::allows_automatic_window_tabbing`, and `WindowBuilderExtMacOS::with_automatic_window_tabbing` on macOS.
  - [7c7ce8ab](https://github.com/tauri-apps/tao/commit/7c7ce8ab2d838a79ecdf83df00124c418a6a51f6) feat(macos): add `allows_automatic_window_tabbing` APIs ([#586](https://github.com/tauri-apps/tao/pull/586)) on 2022-10-12
- Support cross compiling for macos from a non macos host.
  - [2edc7418](https://github.com/tauri-apps/tao/commit/2edc74182378c09eddae22de9fb77a301f2cbbf4) Fix cross compilation. ([#601](https://github.com/tauri-apps/tao/pull/601)) on 2022-10-25
- Add `WindowExtMacOS::is_doucmented_edited` and `WindowExtMacOS::set_is_doucmented_edited` on macOS.
  - [33fdeab6](https://github.com/tauri-apps/tao/commit/33fdeab6291d4aef8ea9facb58fe9583f6c1aaf3) feat(macos): add document edited apis, closes [#268](https://github.com/tauri-apps/tao/pull/268) ([#287](https://github.com/tauri-apps/tao/pull/287)) on 2022-10-03
- On macOS, scale menu item icons height to 18.
  - [5e3d344c](https://github.com/tauri-apps/tao/commit/5e3d344c77006fce03459a851e30cb798e568756) fix(macos): scale menu item icon height to 18, closes [#584](https://github.com/tauri-apps/tao/pull/584) ([#590](https://github.com/tauri-apps/tao/pull/590)) on 2022-10-15
- Add support for the "+" key in menu accelerators using `KeyCode::Plus` or the "Plus" keyword.
  See documentation for `KeyCode::Plus` for notes on platform-dependent behaviour.
  - [937aba7b](https://github.com/tauri-apps/tao/commit/937aba7b7faba04e8d154f7681f97985f0b8ca76) feat(menus): add support for Plus key in accelerators, closes [#227](https://github.com/tauri-apps/tao/pull/227) ([#573](https://github.com/tauri-apps/tao/pull/573)) on 2022-09-27
- Add the application name to the "Quit" and "Hide" native menu items on macOS.
  - [65f768e5](https://github.com/tauri-apps/tao/commit/65f768e55fb1eb53642246c63194ef75e84f908a) fix(menus): add app name to native Quit and Hide items on macOS, closes [#536](https://github.com/tauri-apps/tao/pull/536) ([#570](https://github.com/tauri-apps/tao/pull/570)) on 2022-09-25
- Fix the native Services menu on macOS.
  - [d343abf8](https://github.com/tauri-apps/tao/commit/d343abf8ccc67dff8bc6db2b370a652683360f64) fix(menus): fix macOS Services menu not working, closes [#243](https://github.com/tauri-apps/tao/pull/243) ([#569](https://github.com/tauri-apps/tao/pull/569)) on 2022-09-25
- Scale the tray icon according to its aspect ratio on macOS.
  - [dbbfd97c](https://github.com/tauri-apps/tao/commit/dbbfd97c615ba0eec582b484e06608b87f34ef7a) feat(macos): support to change tray icon aspect ratio, close [#564](https://github.com/tauri-apps/tao/pull/564) ([#565](https://github.com/tauri-apps/tao/pull/565)) on 2022-09-25
- Add builder methods on Linux to control the drawing behavior of the window. `WindowBuilderExtUnix::with_double_buffered`, `WindowBuilderExtUnix::with_rgba_visual` and `WindowBuilderExtUnix::with_app_paintable`
  - [0637c605](https://github.com/tauri-apps/tao/commit/0637c605bd74eaf6ac9995a340bcc650a46664e8) feat(linux): add drawing behavior builder methods, closes [#567](https://github.com/tauri-apps/tao/pull/567) ([#572](https://github.com/tauri-apps/tao/pull/572)) on 2022-09-27
- On Windows, show Window menu (also known as the System menu or Control menu) in response to <kbd>Alt+Space</kbd>.
  - [0d76094e](https://github.com/tauri-apps/tao/commit/0d76094e90252285002a63cb4d889c6c8c485bdf) fix(Windows): show window menu on alt+space, closes 547 ([#593](https://github.com/tauri-apps/tao/pull/593)) on 2022-10-19
- On Windows, fix icons specified on `WindowBuilder` not taking effect for windows created after the firt one.
  - [d72b1e1a](https://github.com/tauri-apps/tao/commit/d72b1e1a0cb7757bdc67008d70c8d25596ad393d) fix(Windows): fix icons specified on `WindowBuilder` not taking effect for windows created after the first one ([#604](https://github.com/tauri-apps/tao/pull/604)) on 2022-10-27
- Added tabbing identifier APIs on macOS.
  - [8815291e](https://github.com/tauri-apps/tao/commit/8815291e8cf4ac855ae4b9ad93183a27d6da5bb7) feat(macos): add tabbing identifier APIs ([#592](https://github.com/tauri-apps/tao/pull/592)) on 2022-10-18
- On Linux, reduce channel redirect. Now sending user events and redraw request will send to event loops directly.
  - [dd86a9eb](https://github.com/tauri-apps/tao/commit/dd86a9ebec67bf103e79bdfd1a2377cbe832bc03) refactor(linux): reduce channel redirect ([#588](https://github.com/tauri-apps/tao/pull/588)) on 2022-10-16
- Add `WindowBuilder::with_focused` to specify whether to initially focus the window or not.
  - [e42ff071](https://github.com/tauri-apps/tao/commit/e42ff07190c37b6b8a7808900133838247a730df) feat: add `WindowBuilder::with_focused` ([#576](https://github.com/tauri-apps/tao/pull/576)) on 2022-10-03
- Add APIs for disabling the individual window controls on desktop platforms, `Window::set_closable`, `Window::is_closable`, `WindowBuilder::with_closable`, `Window::set_minimizable`, `Window::is_minimizable`, `WindowBuilder::with_minimizable`, `Window::set_maximizable`, `Window::is_maximizable`, `WindowBuilder::with_maximizable`. See the docs for platform-specific notes, especially regarding Linux.
  - [a50fd867](https://github.com/tauri-apps/tao/commit/a50fd867b3df033dff077f5320b4b7037b04e454) feat: options to disable individual window controls, closes [#116](https://github.com/tauri-apps/tao/pull/116) ([#574](https://github.com/tauri-apps/tao/pull/574)) on 2022-10-11
- Add `Window::title` to get the current window title.
  - [c50529b3](https://github.com/tauri-apps/tao/commit/c50529b3ee453c73acf36bd7e1cd7c14669951f3) feat: add `Window::title` getter, closes [#546](https://github.com/tauri-apps/tao/pull/546) ([#579](https://github.com/tauri-apps/tao/pull/579)) on 2022-10-04
- Default to MOD_NOREPEAT for registering global shortcuts / hotkeys via win32 RegisterHotKey on Windows. This prevents shortcuts from repeatedly activating when the accelerator is pressed and held down, and ensures that we maintain platform-agnostic consistency.
  - [d15a756c](https://github.com/tauri-apps/tao/commit/d15a756cfca5d7ec7b4a526edd68c3576f308d9a) Prevent global shortcut activation from repeating on Windows ([#602](https://github.com/tauri-apps/tao/pull/602)) on 2022-10-23

## \[0.14.0]

- Implement "always on bottom" as contrary to "always on top".
  - [a2a7b726](https://github.com/tauri-apps/tao/commit/a2a7b7262cc55e4c6defb79d5f77efce9d7e386d) Always on bottom ([#522](https://github.com/tauri-apps/tao/pull/522)) on 2022-08-22
- Fix calling android functions when package name contained escaped underscore.
  - [6d8cc7e3](https://github.com/tauri-apps/tao/commit/6d8cc7e3e4091462a741ee748112a3ea4aa4f12f) fix(android): unescape escaped underscore in package name ([#531](https://github.com/tauri-apps/tao/pull/531)) on 2022-08-16
- Add `Window::set_content_protection` for macOS and Windows.
  - [802146fb](https://github.com/tauri-apps/tao/commit/802146fb8692a46185846a64163c174520450c43) feat: implement set_content_protection, closes [#550](https://github.com/tauri-apps/tao/pull/550) ([#551](https://github.com/tauri-apps/tao/pull/551)) on 2022-09-04
- - Add DeviceEventFilter on Windows.
- **Breaking**: On Windows, device events are now ignored for unfocused windows by default, use `EventLoopWindowTarget::set_device_event_filter` to set the filter level.
- [5bbd4f8f](https://github.com/tauri-apps/tao/commit/5bbd4f8f72901425432a35915d79d0bee0c96cce) Add DeviceEventFilter on Windows ([#465](https://github.com/tauri-apps/tao/pull/465)) on 2022-08-17
- Fix system tray creation after event loop starts on macOS.
  - [759b7db3](https://github.com/tauri-apps/tao/commit/759b7db37b8188ea38fa2919f9a0e504d4d2edca) fix(macos): retain tray to prevent segfault when event loop is running ([#539](https://github.com/tauri-apps/tao/pull/539)) on 2022-08-20
- Fix resize doesn't work when calling with resizable. Also add platform specific note to `set_resizable`.
  On Linux, most size methods like maximized are async and do not work well with calling
  sequentailly. For setting inner or outer size, you don't need to set resizable to true before
  it. It can resize no matter what. But if you insist to do so, it has a `100, 100` minimum
  limitation somehow. For maximizing, it requires resizable is true. If you really want to set
  resizable to false after it. You might need a mechanism to check the window is really
  maximized.
  - [4524d5d3](https://github.com/tauri-apps/tao/commit/4524d5d399c8bf01d22b160a9f9a04d5b074b466) fix(Linux): resize doesn't work when calling with resizable, fix [#545](https://github.com/tauri-apps/tao/pull/545) ([#553](https://github.com/tauri-apps/tao/pull/553)) on 2022-09-08
- Add `Window::is_focused`.
  - [7d2eeeeb](https://github.com/tauri-apps/tao/commit/7d2eeeebb4da15e9aeda9bf17e80ebdf23c95cee) feat: Window::is_focused ([#533](https://github.com/tauri-apps/tao/pull/533)) on 2022-08-17
- On Linux, fix global shortcut are never triggered when a Lock key is ON, eg. NumLock, CapsLock.
  - [07e3c1f5](https://github.com/tauri-apps/tao/commit/07e3c1f55d18537dc5c776b2706490676bba7cde) fix(linux/globalShorcut): extract needed mods from event state, closes [#307](https://github.com/tauri-apps/tao/pull/307), closes [#537](https://github.com/tauri-apps/tao/pull/537) ([#538](https://github.com/tauri-apps/tao/pull/538)) on 2022-08-19
  - [871ad037](https://github.com/tauri-apps/tao/commit/871ad037b02b8ab4d650ba390664386e195c0bc7) chore: remove changefile, bug still exists on 2022-08-20
  - [7e5556e0](https://github.com/tauri-apps/tao/commit/7e5556e0f247076e8f547fca313d957eeca46366) fix(linux/globalShortcut): grab the shortcut with extra mods, closes [#307](https://github.com/tauri-apps/tao/pull/307) ([#540](https://github.com/tauri-apps/tao/pull/540)) on 2022-08-20
- Disables the global shortcut manager on wayland as its X11-specific.
  - [27ab6f4d](https://github.com/tauri-apps/tao/commit/27ab6f4dcef19c052b0434873e34b54447b70860) fix(linux/globalShortcut): disable on wayland ([#543](https://github.com/tauri-apps/tao/pull/543)) on 2022-08-26
- Added `SystemTrayExtMacOS::set_title` to `SystemTray` and `SystemTrayBuilderExtMacOS::with_title` to set the tray icon title on MacOS
  - [972307dd](https://github.com/tauri-apps/tao/commit/972307ddf088b0f941be2ea66bded2473222aed5) feat: added text support to system tray for macos, closes [#65](https://github.com/tauri-apps/tao/pull/65) ([#554](https://github.com/tauri-apps/tao/pull/554)) on 2022-09-10
- Update `windows-rs` to the latest 0.39.0 release.

The `alloc` feature has been removed, which means it no longer accepts Rust `String` or `&str` parameters and implicitly converts them to `PWSTR` or `PSTR`.

For string literals, that feature was replaced with `s!()` and `w!()` macros which null terminate the string literal at compile time and convert to UTF-16 if necessary. The `s!()` macro is fine, however the `w!()` macro uses `HSTRING` types from WinRT for maximum compatibility with WinRT types. Since Tao only uses Win32 APIs, this change relies on `util::encode_wide` to convert to a `Vec<u16>` instead.

- [84e1a9f9](https://github.com/tauri-apps/tao/commit/84e1a9f93fa7e9e83f1ed92320ab9d7998673c60) Update windows to 0.39.0 ([#544](https://github.com/tauri-apps/tao/pull/544)) on 2022-08-31

## \[0.13.3]

- Implement custom protocol on Android.
  - [b464b8ae](https://github.com/tauri-apps/tao/commit/b464b8ae296cf2b545cd0a98a1506f83779e94ff) feat(android): implement custom protocol ([#527](https://github.com/tauri-apps/tao/pull/527)) on 2022-08-13
- Changed `WebViewMessage::Eval` to evaluate an specific script.
  - [903c7e7f](https://github.com/tauri-apps/tao/commit/903c7e7f5b8c7984515697865fc7c74b496a64dc) feat(android): change WebViewMessage::Eval to run specific script ([#529](https://github.com/tauri-apps/tao/pull/529)) on 2022-08-13
- Fix webview initialization scripts implementation on Android.
  - [3d66ad0b](https://github.com/tauri-apps/tao/commit/3d66ad0b5548ed40da6e954fb5a911c3fb5a13e8) fix(android): run initialization scripts before page loads ([#528](https://github.com/tauri-apps/tao/pull/528)) on 2022-08-13
- Removed the webview logic from the Android glue.
  - [152aaa44](https://github.com/tauri-apps/tao/commit/152aaa4481ff8e44fc32ea3fe93a74c7fecd5be5) refactor(android): remove WebView logic, allow wry to hook into it ([#530](https://github.com/tauri-apps/tao/pull/530)) on 2022-08-14
- Implement `SystemTray::set_tooltip` and `SystemTrayBuilder::with_tooltip` on Windows.
  - [06949a79](https://github.com/tauri-apps/tao/commit/06949a7948100a51e98008c9e6f4ac73e069433a) feat(windows): implement `with_tooltip`&`set_tooltip`, closes [#205](https://github.com/tauri-apps/tao/pull/205) ([#524](https://github.com/tauri-apps/tao/pull/524)) on 2022-08-10

## \[0.13.2]

- Remove the NSStatusItem from the menu bar when the `SystemTray` instance is dropped.
  - [aca4d3fb](https://github.com/tauri-apps/tao/commit/aca4d3fb2619d8bd38e4514583e921227cba6a04) feat(tray): remove from tray on `Drop` on macOS ([#520](https://github.com/tauri-apps/tao/pull/520)) on 2022-08-04
- Fixes `Window::is_decorated` always returning `true` on macOS.
  - [c3e076e9](https://github.com/tauri-apps/tao/commit/c3e076e9345ad33183426f5ef4bd936305254e15) fix(window): `is_decorated` wrong return value, closes [#518](https://github.com/tauri-apps/tao/pull/518) ([#519](https://github.com/tauri-apps/tao/pull/519)) on 2022-08-04
- Fix theme feature to support Darker theme on Linux.
  - [c6d6c011](https://github.com/tauri-apps/tao/commit/c6d6c0115c2facd488e8fab73c8f8b92e172771c) fix: support Darker theme on Linux ([#511](https://github.com/tauri-apps/tao/pull/511)) on 2022-08-03
- Add `Window::is_minimized()`.
  - [9c348154](https://github.com/tauri-apps/tao/commit/9c3481548b05de1d56c2efe8a1951fd014006b27) feat: add `Window::is_minimized()`, closes [#257](https://github.com/tauri-apps/tao/pull/257) ([#486](https://github.com/tauri-apps/tao/pull/486)) on 2022-08-06
- Implement `SystemTrayBuilder::with_tooltip` and `SystemTray::set_tooltip` on macOS.
  - [14e26568](https://github.com/tauri-apps/tao/commit/14e265682fab87502d59a718c9607aaf146c4d3e) feat(macos): add `SystemTray::set_tooltip`, ref [#409](https://github.com/tauri-apps/tao/pull/409) ([#410](https://github.com/tauri-apps/tao/pull/410)) on 2022-08-03
- On Windows, fix a ghost window appearing occasionally when clicking the tray icon.
  - [ad1f641f](https://github.com/tauri-apps/tao/commit/ad1f641f496c21a02c8d173167d77f1b31849273) fix(windows): fix tray event window showing up on click, closes [#506](https://github.com/tauri-apps/tao/pull/506) ([#507](https://github.com/tauri-apps/tao/pull/507)) on 2022-08-02
- Added `SystemTrayBuilder::with_id` and the `id` field to `Event::TrayEvent` for better multitray support.
  - [4ea78bcb](https://github.com/tauri-apps/tao/commit/4ea78bcb577f36ebd4f6b7ce4fcd31d7c02cafdb) feat(tray): add identifier to allow multiple tray setup ([#514](https://github.com/tauri-apps/tao/pull/514)) on 2022-08-04
- Hide the app indicator when dropping `SystemTray` on Linux
  - [9c6a543c](https://github.com/tauri-apps/tao/commit/9c6a543c1a748ed53ae408780a65043f6a9448f9) feat(tray): hide indicator on drop on Linux ([#521](https://github.com/tauri-apps/tao/pull/521)) on 2022-08-04

## \[0.13.1]

- On Linux, fix Window can't be displayed on wayland.
  - [eb880f48](https://github.com/tauri-apps/tao/commit/eb880f48932adb96bc428efdf69e2256fe989b6b) Fix window can't be displayed on wayland ([#504](https://github.com/tauri-apps/tao/pull/504)) on 2022-07-28

## \[0.13.0]

- On Linux, receive only one draw event per cycle to prevent receiving infinite draw events.
  - [b86ada73](https://github.com/tauri-apps/tao/commit/b86ada73ccc493340f4cee35d884867623287111) Receive only one draw event per cycle ([#500](https://github.com/tauri-apps/tao/pull/500)) on 2022-07-25
- - On Linux, add `EventLoopWindowTargetExtUnix` for methods to determine if the backend is x11 or wayland.
- On Linux, add `x11` module for glutin internal use. This is basically just x11-dl, but winit secretly exports it.
- On Linux, add `WindowBuilder::with_transparent_draw` to disable the internal draw for transparent window and allows users to draw it manually.
- [db7e5cb4](https://github.com/tauri-apps/tao/commit/db7e5cb4466133869f512487e605b061a6610560) feat(linux): Add necessary features for creating GL windows ([#495](https://github.com/tauri-apps/tao/pull/495)) on 2022-07-25
- **Breaking** Updated `raw-window-handle` to `0.5` and added `Window::raw_display_handle` and `EventLoopWindowTarget::raw_display_handle`.
  - [b905852d](https://github.com/tauri-apps/tao/commit/b905852d2e76dafaadc8c0ca5785328981628bf0) chore(deps): update `raw-window-handle` to `0.5` ([#493](https://github.com/tauri-apps/tao/pull/493)) on 2022-07-24
- On Windows, respect min/max inner sizes when creating the window.
  - [c1c6822e](https://github.com/tauri-apps/tao/commit/c1c6822e8bb4708857225491b939f076b120dec1) fix(windows): respect min/max sizes when creating window, closes [#498](https://github.com/tauri-apps/tao/pull/498) ([#499](https://github.com/tauri-apps/tao/pull/499)) on 2022-07-25

## \[0.12.2]

- On Windows, fix assigning the wrong mintor rect to undecorated maximized window. This caused a blank window downstream in wry and tauri.
  - [9d97e4a6](https://github.com/tauri-apps/tao/commit/9d97e4a646ce8a4372d3ed2c22d227d8a33ba8ba) fix(windows): get correct monitor in `WM_NCCALCSIZE`, closes [#471](https://github.com/tauri-apps/tao/pull/471) ([#472](https://github.com/tauri-apps/tao/pull/472)) on 2022-07-12
- Fixed set_inner_size is reset when resizable is set to false.
  - [17203d08](https://github.com/tauri-apps/tao/commit/17203d08a4ee49c8fa8decb24bcf76fe4c264ca7) fix: fixed inner_size even if resizable is set to false ([#461](https://github.com/tauri-apps/tao/pull/461)) on 2022-07-05
- On Windows, prevent ghost window from showing up in the taskbar after either several hours of use or restarting `explorer.exe`.
  - [feb21272](https://github.com/tauri-apps/tao/commit/feb212726da553397beda6428666092b0561fd12) fix(windows): prevent ghost window from showing up on taskbar ([#489](https://github.com/tauri-apps/tao/pull/489)) on 2022-07-21
- Add theme feature on Linux.
  - [74425e8e](https://github.com/tauri-apps/tao/commit/74425e8e5b032299cec99f8278d9a05ae650013c) feat: add theme feature on Linux ([#468](https://github.com/tauri-apps/tao/pull/468)) on 2022-07-10
- Fix maximizing window on Linux.
  - [01fb1d6c](https://github.com/tauri-apps/tao/commit/01fb1d6cdf17f01ebe5757f4a66a0d8e40222490) fix: maximizing window on linux, closes [#442](https://github.com/tauri-apps/tao/pull/442) ([#456](https://github.com/tauri-apps/tao/pull/456)) on 2022-07-12
- On macOS, fallback resize event for NSWindow to handle.
  - [ab2e57e9](https://github.com/tauri-apps/tao/commit/ab2e57e9ec056861fa772262a2128c2ac2e16d1b) On macOS, fallback resize event for NSWindow to handle on 2022-07-12
- Add `CustomMenuItem::set_icon`. Only implemented on macOS for now.
  - [13f9f182](https://github.com/tauri-apps/tao/commit/13f9f182754b0bfbaa9163330aed4d444b1e007a) feat(macos): implement CustomMenuItem::set_icon() ([#459](https://github.com/tauri-apps/tao/pull/459)) on 2022-07-07
- On Windows, subscribe to taskbar restart event and re-add the system tray icon.
  Also skip the window from the taskbar if it was already skipped.
  - [9450329e](https://github.com/tauri-apps/tao/commit/9450329e3ab70aa3608ef44207df19cfdddf45a0) fix(windows): subscribe to taskbar restart event, closes [#476](https://github.com/tauri-apps/tao/pull/476) ([#487](https://github.com/tauri-apps/tao/pull/487)) on 2022-07-21
- On Windows, fix focus events being sent to inactive windows.
  - [23ae71b7](https://github.com/tauri-apps/tao/commit/23ae71b717184e2eb0f2da0c683b7c8f0b5cd216) fix(windows): fix focus events being sent to inactive windows. ([#488](https://github.com/tauri-apps/tao/pull/488)) on 2022-07-21

## \[0.12.1]

- Revert #427 due to random crash caused by it.
  - [38f9a587](https://github.com/tauri-apps/tao/commit/38f9a587c5a394a88c767b28e1bf2ae40f990bae) Revert "Remove most RedrawWindow to event target window" ([#457](https://github.com/tauri-apps/tao/pull/457)) on 2022-07-01

## \[0.12.0]

- On macOS, fix native file dialogs hanging the event loop and
  having multiple windows would prevent `run_return` from ever returning.
  - [5c9cc21a](https://github.com/tauri-apps/tao/commit/5c9cc21a394b3d6b32c794453344263457f7d223) Fix native file dialogs freezing the event loop ([#440](https://github.com/tauri-apps/tao/pull/440)) on 2022-06-22
- Fix maximizing window.
- On Windows, fix wrong fullscreen monitors being recognized when handling `WM_WINDOWPOSCHANGING` messages
  - [054a34ec](https://github.com/tauri-apps/tao/commit/054a34ec504dc98235d1fafc9b1cdede7727193e) fix: fix assigning the wrong monitor when receiving Windows move events ([#438](https://github.com/tauri-apps/tao/pull/438)) on 2022-06-22
- Fix global hide others shortcut.
  - [dfae373e](https://github.com/tauri-apps/tao/commit/dfae373e58da44ab6adc977ffe24e3d55ed51de0) fix: global hide others shortcut ([#447](https://github.com/tauri-apps/tao/pull/447)) on 2022-06-25
- Fix window can't be hidden when maximized.
  - [cd9ad33a](https://github.com/tauri-apps/tao/commit/cd9ad33a088b3ab5dbcf3ff1681ebce323c5c61d) Fix window can't be hidden when maximized ([#384](https://github.com/tauri-apps/tao/pull/384)) on 2022-06-15
- On macOS, `WindowEvent::Resized` is now emitted in `frameDidChange` instead of `windowDidResize`.
  - [54062ca1](https://github.com/tauri-apps/tao/commit/54062ca1a96e3637acee771e9657ef0267933352) fix: emit resize event on frame_did_change on macOS, closes [#436](https://github.com/tauri-apps/tao/pull/436) ([#439](https://github.com/tauri-apps/tao/pull/439)) on 2022-06-22
- On Linux, adds `SystemTrayBuilderExtLinux::with_temp_icon_dir` which sets a custom temp icon dir to store generated icon files.
  This may be useful when the application requires icons to be stored in a specific location, such as when running in a Flatpak sandbox.
  - [ce209d39](https://github.com/tauri-apps/tao/commit/ce209d39ab21e493ba98bab83aa7827fa05d7050) feat(linux) add `with_temp_icon_dir` builder extension ([#452](https://github.com/tauri-apps/tao/pull/452)) on 2022-06-26
- On Linux, store tray icons in `$XDG_RUNTIME_DIR`.
  This is preferred over `/tmp`, because this directory (typically `/run/user/{uid}`)
  is only readable for the current user. While `/tmp` is shared with all users.
  - [01253829](https://github.com/tauri-apps/tao/commit/01253829dc23b7316db8941ed8c302d479890186) feat(linux): store tray icons in `XDG_RUNTIME_DIR` ([#449](https://github.com/tauri-apps/tao/pull/449)) on 2022-06-25
- Do not emit the `ThemeChanged` event when the window theme is set and the system theme changes (the window keeps its theme in this scenario).
  - [aae6bec9](https://github.com/tauri-apps/tao/commit/aae6bec9110c19ed0b6618f08e6ce48f483bbfd0) fix(macos): do not emit ThemeChanged event if window theme didn't change ([#430](https://github.com/tauri-apps/tao/pull/430)) on 2022-06-20
- Remvoe `core-video-sys` dependency.
  - [3bb09aa6](https://github.com/tauri-apps/tao/commit/3bb09aa6a03c39bb78378fb85e977c37d8a47a79) fix: remove core-video-sys dependency, closes [#435](https://github.com/tauri-apps/tao/pull/435) ([#441](https://github.com/tauri-apps/tao/pull/441)) on 2022-06-22
- The `theme` function now `Theme::Light` on macOS older than 10.14 and the initial theme setter has no effect instead of crashing the application.
  - [ba9c5571](https://github.com/tauri-apps/tao/commit/ba9c5571f408bdca8584b1b44cc1b95a927d8e34) fix(macos): guard theme APIs to not crash when running on 10.13 or older ([#429](https://github.com/tauri-apps/tao/pull/429)) on 2022-06-20
- Reduce `WM_PAINT` singal on event target window to prevent from webview2 delay.
  - [5ca39af1](https://github.com/tauri-apps/tao/commit/5ca39af1117677b469f92a8094769610c01419ad) Remove most RedrawWindow to event target window ([#427](https://github.com/tauri-apps/tao/pull/427)) on 2022-06-28

## \[0.11.2]

- Fixes the `Ivar menu_on_left_click not found on class TaoTrayHandler` panic on macOS.
  - [2cc163d2](https://github.com/tauri-apps/tao/commit/2cc163d2debba48457a63f4a839f1371b572e121) fix(macos): crash on tray class usage on 2022-06-14

## \[0.11.1]

- Fix macOS `SystemTrayExtMacOS` implementation.
  - [f42c1be1](https://github.com/tauri-apps/tao/commit/f42c1be13ce949a3ca47c9126c2c1e914dee179a) fix: fix wrong macOS trait implementation on 2022-06-14

## \[0.11.0]

- **Breaking change** `SystemTrayBuilder::new` and `SystemTray::set_icon` now takes `system_tray::Icon` on all platforms.
  - [0a98eb39](https://github.com/tauri-apps/tao/commit/0a98eb3993d9f24323f71520426712009bd9e272) refactor: system tray icons ([#328](https://github.com/tauri-apps/tao/pull/328)) on 2022-06-06
- Allow to disable system tray menu only on Left Click.
  - [0858356f](https://github.com/tauri-apps/tao/commit/0858356f3a14fcb6e1e1dfc8d2d35482388ccb43) feat(macos): allow to disable system tray menu on left click, closes [#317](https://github.com/tauri-apps/tao/pull/317) ([#329](https://github.com/tauri-apps/tao/pull/329)) on 2022-06-09
- Connect mouse wheel event with GTK window.
  - [f9e0b734](https://github.com/tauri-apps/tao/commit/f9e0b734c6a3737174d63a0ec8cb2ebc130f35f8) connect mouse wheel event with GTK window ([#412](https://github.com/tauri-apps/tao/pull/412)) on 2022-06-08
- Support child window on Linux.
  - [f1e8d755](https://github.com/tauri-apps/tao/commit/f1e8d7556eb9aea89769a9b38407e8fcd12675af) feat: support child window on linux, closes [#273](https://github.com/tauri-apps/tao/pull/273) ([#415](https://github.com/tauri-apps/tao/pull/415)) on 2022-06-13
- Support theme on macOS.
  - [8af4d8f0](https://github.com/tauri-apps/tao/commit/8af4d8f02149f08093cc348e278f5792dab4a423) feat: support theme on macOS ([#408](https://github.com/tauri-apps/tao/pull/408)) on 2022-06-01
- Add `Window::set_ignore_cursor_events`
  - [4fa87617](https://github.com/tauri-apps/tao/commit/4fa8761776d546ee3b1b0bb1a02a31d72eedfa80) feat: `Window::set_ignore_cursor_events`, closes [#184](https://github.com/tauri-apps/tao/pull/184) ([#421](https://github.com/tauri-apps/tao/pull/421)) on 2022-06-13

## \[0.10.0]

- Fix movable window background on macOS.
  - [e0520b48](https://github.com/tauri-apps/tao/commit/e0520b488bd95167c73c971448e83775032037af) fix: fix movable window background on macOS, closes [#406](https://github.com/tauri-apps/tao/pull/406) ([#405](https://github.com/tauri-apps/tao/pull/405)) on 2022-05-27
- Remove trivial tray features.
  - [f1bd25e6](https://github.com/tauri-apps/tao/commit/f1bd25e643b8a8a656ee64678e9f95135feafb39) Remove trivial tray features ([#411](https://github.com/tauri-apps/tao/pull/411)) on 2022-05-30

## \[0.9.1]

- Fix the size of the slice passed to `DragQueryFileW` by passing `std::mem::transmute(path_buf.spare_capacity_mut())` instead of `&mut path_buf`.
  - [d0dbfa1a](https://github.com/tauri-apps/tao/commit/d0dbfa1a1274f16348701a63b213e9e92776cd74) Fix drag drop on Windows ([#401](https://github.com/tauri-apps/tao/pull/401)) on 2022-05-23

## \[0.9.0]

- Add standalone webview ndk port.
  - [68c9f07e](https://github.com/tauri-apps/tao/commit/68c9f07e8e24690ddf449ebc545ebd90cd29e118) Implement standalone webview ndk ([#385](https://github.com/tauri-apps/tao/pull/385)) on 2022-05-19
- Update the `windows` crate to the latest 0.37.0 release.

The `#[implement]` macro in `windows-implement` and the `implement` feature in `windows` depend on some `const` generic features which stabilized in `rustc` 1.61. The MSRV on Windows targets is effectively 1.61, but other targets do not require these features.

Since developers on non-Windows platforms are not always able to upgrade their toolchain with `rustup`, the package remains at 1.56. Windows developers may get less friendly compiler errors about using unstable language features until they upgrade their toolchain if they build `tao` without `wry`, which has some Windows-specific dependencies that transitively raise the MSRV for `wry` to 1.61.

- [93c256f9](https://github.com/tauri-apps/tao/commit/93c256f9835b2da853129f2a1d77287aa714934e) Update the windows-rs crate to 0.37.0 ([#400](https://github.com/tauri-apps/tao/pull/400)) on 2022-05-23

## \[0.8.5]

- The `current_monitor` function now fallbacks to the primary monitor when the window is invisible.
  - [6cdb99fd](https://github.com/tauri-apps/tao/commit/6cdb99fd271acc6bc2f1132c799c202c444ec7f2) fix(linux): fallback to primary monitor on `current_monitor` impl ([#395](https://github.com/tauri-apps/tao/pull/395)) on 2022-05-18
- Change menubar background color to transparent on Linux when the window is transparent.
  - [a0d9408b](https://github.com/tauri-apps/tao/commit/a0d9408bb89f8dd60687885c2344f7d1069e73d0) fix(linux): make menubar background transparent ([#389](https://github.com/tauri-apps/tao/pull/389)) on 2022-05-14
- Rename full screen menu label to "Toggle Full Screen".
  - [8945f544](https://github.com/tauri-apps/tao/commit/8945f544e77d5cd22a1f3e2ad492c4a2dc986268) fix: rename full screen menu label, closes [#391](https://github.com/tauri-apps/tao/pull/391) ([#393](https://github.com/tauri-apps/tao/pull/393)) on 2022-05-17

## \[0.8.4]

- On Windows, remove the accelerator from `CustomMenuItem::title` returnd string.
  - [634116fe](https://github.com/tauri-apps/tao/commit/634116feb3fe983070fbea79e780caea6d4e7581) fix(Windows): remove accel str from `CustomMenuItem::title` returned string ([#377](https://github.com/tauri-apps/tao/pull/377)) on 2022-04-28
- On Windows and Linux, increase the resizing area for borderless windows based on scale factor.
  - [8701f64a](https://github.com/tauri-apps/tao/commit/8701f64aaec862dc15510ee725498d954521f123) fix: scale borderless resizing inset based on scale_factor, closes [#376](https://github.com/tauri-apps/tao/pull/376) ([#379](https://github.com/tauri-apps/tao/pull/379)) on 2022-05-01

## \[0.8.3]

- Implement `Window::set_cursor_position` for Linux.
  - [afffaeae](https://github.com/tauri-apps/tao/commit/afffaeae665e06804ec1f2a7056afb27431baf10) feat(linux): implement `Window::set_cursor_position` ([#373](https://github.com/tauri-apps/tao/pull/373)) on 2022-04-23

## \[0.8.2]

- Do not fire `WindowEvent::Moved` when `is_maximized` is called on macOS.
  - [25890b94](https://github.com/tauri-apps/tao/commit/25890b943f3566cb8b2fc6d5abaff15921caed93) fix(macos): do not fire Event::Moved when checking is_maximized ([#366](https://github.com/tauri-apps/tao/pull/366)) on 2022-04-13

## \[0.8.1]

- Fixes compilation when only the `tray` feature is enabled.
  - [da938957](https://github.com/tauri-apps/tao/commit/da9389573daa04217baa8465709328e9c6e35f27) fix(tao): compilation when only the tray feature is enabled ([#363](https://github.com/tauri-apps/tao/pull/363)) on 2022-04-05

## \[0.8.0]

- Add `EventLoopWindowTargetExtMacOS::set_activation_policy_at_runtime`.
  - [ef06c508](https://github.com/tauri-apps/tao/commit/ef06c508f1d29e62834eba63b604bf7566b1fef6) Set activation policy at runtime ([#353](https://github.com/tauri-apps/tao/pull/353)) on 2022-03-30
- On Windows and Linux, disable resizing maximized borderless windows.
  - [13c5c996](https://github.com/tauri-apps/tao/commit/13c5c996d15cee9ed829f6e67e786721d8d2eda8) fix(win,linux): disable resizing maximized borderless windows ([#356](https://github.com/tauri-apps/tao/pull/356)) on 2022-03-30
- **Breaking change:** Renamed the `ayatana` Cargo feature to `ayatana-tray`, now the default feature for tray on Linux, and added the `gtk-tray` feature.
  - [40ec796d](https://github.com/tauri-apps/tao/commit/40ec796de4da91640872161ae372124d14777d7f) refactor(tray): split gtk and ayatana appindicator features ([#362](https://github.com/tauri-apps/tao/pull/362)) on 2022-04-05
- - On Windows, Fix random characters when changing menu items title through `CustomMenunItem::set_title`.
  - [e4725bf5](https://github.com/tauri-apps/tao/commit/e4725bf50fb46e830fa5265d765ee596b80e3085) fix(Windows): fix random chars when changing menu item title ([#361](https://github.com/tauri-apps/tao/pull/361)) on 2022-03-31
- On Windows, Fix `Window::set_inner_size` setting a bigger size than requested.
  - [089f3878](https://github.com/tauri-apps/tao/commit/089f3878c5b0ce221d3f405c1215c895ee9fb1ce) fix(Windows): fix `set_inner_size` setting a bigger size, closes [#194](https://github.com/tauri-apps/tao/pull/194) ([#354](https://github.com/tauri-apps/tao/pull/354)) on 2022-04-03

## \[0.7.0]

- Fire `Event::LoopDestroyed` when the macOS dock `Quit` menu item is clicked.
  - [34257a75](https://github.com/tauri-apps/tao/commit/34257a75c71e40433a57942cc61ce9976b80c152) feat(macos): fire `LoopDestroyed` when the dock's `Quit` item is clicked ([#351](https://github.com/tauri-apps/tao/pull/351)) on 2022-03-27
- Added `Event::DecorationsClick` (Windows only).
  - [411af5b1](https://github.com/tauri-apps/tao/commit/411af5b16d71eec90be47210fc6242526ab43c6c) feat(windows): add `Event::DecorationsClick` ([#352](https://github.com/tauri-apps/tao/pull/352)) on 2022-03-27
- Enhance the `MenuItem::About` menu on Linux.
  **Breaking change:** The About variant now uses an struct instead of a string.
  - [84c677fd](https://github.com/tauri-apps/tao/commit/84c677fd13234c81bbbe63b25d7dc563825c7829) refactor: fix and enhance the about menu on Linux ([#347](https://github.com/tauri-apps/tao/pull/347)) on 2022-03-25
- Fixes the About menu on Linux not being shown.
  - [84c677fd](https://github.com/tauri-apps/tao/commit/84c677fd13234c81bbbe63b25d7dc563825c7829) refactor: fix and enhance the about menu on Linux ([#347](https://github.com/tauri-apps/tao/pull/347)) on 2022-03-25
- Properly fire `WindowEvent::Destroyed` on Linux when the `Window` is dropped.
  - [cdd4ac32](https://github.com/tauri-apps/tao/commit/cdd4ac3281ad9c2cf15561e8cf3110ed34ae93f0) fix(events): properly fire `WindowEvent::Destroyed` on Linux ([#349](https://github.com/tauri-apps/tao/pull/349)) on 2022-03-25
- Properly change the window to fullscreen state if the builder instructs it to use `Fullscreen::Borderless(None)`.
  - [5ecbac19](https://github.com/tauri-apps/tao/commit/5ecbac1958518eaacd263eaaee440f89b2edf122) fix(window): fullscreen on Linux when builder is set to Borderless(None) ([#348](https://github.com/tauri-apps/tao/pull/348)) on 2022-03-25
- Fixes system tray item titles on Windows by forcing the string to be null-terminated.
  - [7f900a16](https://github.com/tauri-apps/tao/commit/7f900a167e0077c354fb26ab6d34ae06591a67c5) fix(tray): force item title string to be null-terminated ([#340](https://github.com/tauri-apps/tao/pull/340)) on 2022-03-09
- Properly fire `WindowEvent::Destroyed` on macOS when the `Window` is dropped.
  - [efd3eecc](https://github.com/tauri-apps/tao/commit/efd3eecc76c45619e36aa8b253316192eefec0d1) fix(window): properly fire `WindowEvent::Destroyed` on macOS ([#350](https://github.com/tauri-apps/tao/pull/350)) on 2022-03-25
- Fix inconsist behaviour when setting menu on mac.
  - [5abdbd1f](https://github.com/tauri-apps/tao/commit/5abdbd1ff7e46f94a686052a65d528725c5647be) Fix inconsist behaviour when setting menu on mac ([#345](https://github.com/tauri-apps/tao/pull/345)) on 2022-03-17

## \[0.6.4]

- Fix a deadlock on Windows when using `Window::set_visible(true)` in the `EventLoop::run` closure.
  - [475e64d2](https://github.com/tauri-apps/tao/commit/475e64d2873c233e60cb74e52e91282d18e13780) fix(Windows): fix a deadlock in `WindowState` ([#338](https://github.com/tauri-apps/tao/pull/338)) on 2022-03-06
- On Windows, apply maximize state before minimize. Fixes `Window::set_minimized` not working when the window is maximized.
  - [11dac102](https://github.com/tauri-apps/tao/commit/11dac10241330c30aae660a2621d43ee5eb3775d) fix(windows): apply maximize state before minimize ([#334](https://github.com/tauri-apps/tao/pull/334)) on 2022-03-01

## \[0.6.3]

- Revert Global Shortcut fix on Linux. See [#331](https://github.com/tauri-apps/tao/issues/331) for more information.
  - [f5e19e0f](https://github.com/tauri-apps/tao/commit/f5e19e0ff83a65410e18ee7e47cac984f87236c9) Revert "Implement global shortcut on Linux, close #307 (#308)" ([#330](https://github.com/tauri-apps/tao/pull/330)) on 2022-03-01

## \[0.6.2]

- Fixes the `set_fullscreen` implementation on Linux when the `Fullscreen::Borderless` value is set to `None`.
  - [456147de](https://github.com/tauri-apps/tao/commit/456147de99e1135b145447a9c8ebb397d3ecd1e1) fix(linux): fullscreen on current monitor ([#320](https://github.com/tauri-apps/tao/pull/320)) on 2022-02-13

## \[0.6.1]

- Fix global shortcut support on Linux (both x11 and wayland).
  - [9c2841f7](https://github.com/tauri-apps/tao/commit/9c2841f7f5382e92efaa8b7bb137d8d30f3e0338) Implement global shortcut on Linux, close [#307](https://github.com/tauri-apps/tao/pull/307) ([#308](https://github.com/tauri-apps/tao/pull/308)) on 2022-02-10

## \[0.6.0]

- Update to gtk 0.15
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Emit errors when parsing an invalid accelerator from a string.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Add support for more accelerator keys: `,` `-` `.` `=` `;` `/` `\` `'` `` ` `` `[` `]` `Space` `Tab` and `F13`-`F24`
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Increased Borderless window resizing inset.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Update to 2021 edition and msrv to 1.56
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- **Breaking:** Rename the `Exit` variant of `ControlFlow` to `ExitWithCode`, which holds a value to control the exit code after running. Add an `Exit` constant which aliases to `ExitWithCode(0)` instead to avoid major breakage. This shouldn't affect most existing programs.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Fixes the `MenuItem::Quit` behavior on Windows.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Add support for `SPACE` shortcut key on Windows.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- - Fix redrawn event that causing infinite lopp on Linux
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Fix linux native menu items not working.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- - Fix resizing undecorated window on Linux.
- Undecorated window can be resized using touch on Linux.
- [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Fix focus events not firing on Linux
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Add monitor selection when fullscreen on Linux and close possible way to create VideoMode on Linux since gtk doesn't acutally have such feature.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- - Add `RedrawEventsCleared` and `RedrawRequested` on Linux
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Add run_return trait on Linux
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- `window.set_skip_taskbar()` on Linux will now also skip the pager (Alt+Tab), this matches the behavior on Windows.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Update tray dependency version.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Fix deadlock when unregistering shortcut on Linux.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Fire `WindowEvent::Resized` and `WindowEvent::Moved` when window is min/maximized on Linux to align with Windows behavior.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Fix menubar missing on borderless window.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Fix core-video-sys dependency.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Fix linking to the `ColorSync` framework on macOS 10.7, and in newer Rust versions.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Allow more strings to parse to keycode, for example `,` is now parsed as a comma.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- - Update `raw-window-handle` to `0.4`
- Add `raw_window_handle()` implementation on linux.
- [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Fix click events missing whe tray has menu.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Add macOS `show_application()` method
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Add new_any_thread to Unix event loop.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Replace all of the `winapi` crate references with the `windows` crate. The generated bindings are in the `webview2-com-sys` crate to share types with WRY later.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Implement `Clone` for `EventLoopWindowTarget`.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Update the `windows` crate to 0.25.0, which comes with pre-built libraries. Tao no longer depends on `webview2-com-sys` to generate bindings shared with WRY.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Update the `windows` crate to 0.29.0.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Update the `windows` crate to 0.30.0. This version re-introduced a lot of new-types for things like HWND, LRESULT, WPARAM, LPARAM, etc.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Fix using `WindowBuilder::with_visible` and `WindowBuilder::with_maximized` not behaving correctly.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- On Windows, send correct position on system tray events.
  - [0dd71973](https://github.com/tauri-apps/tao/commit/0dd7197326fc885150f07ebc3127fd8452a36f00) Merge next back to dev branch ([#305](https://github.com/tauri-apps/tao/pull/305)) on 2022-02-05
- Add support for more accelerator keys: `,` `-` `.` `=` `;` `/` `\` `'` `` ` `` `[` `]` `Space` `Tab` and `F13`-`F24`
  - [b047ae41](https://github.com/tauri-apps/tao/commit/b047ae41a83b94cb140cb3a3decd1ddb4ea6d405) feat: support accelerator key strings `,` `-` `.` `Space` `Tab` and `F13`-`F24` ([#228](https://github.com/tauri-apps/tao/pull/228)) on 2021-11-15
- Allow more strings to parse to keycode, for example `,` is now parsed as a comma.
  - [f0a3dcee](https://github.com/tauri-apps/tao/commit/f0a3dceec585cbfa6749746fdf59054d17ab4d0b) feat: Allow more strings to parse to keycode ([#229](https://github.com/tauri-apps/tao/pull/229)) on 2021-11-03
- Add macOS `show_application()` method
  - [7e10b0df](https://github.com/tauri-apps/tao/commit/7e10b0dfd5018aa3dc326ca99f663efe92a255df) feat(macos): Add `unhide_application` method, closes [#182](https://github.com/tauri-apps/tao/pull/182) ([#231](https://github.com/tauri-apps/tao/pull/231)) on 2021-11-03

## \[0.5.2]

- Fix missing `Sync` trait on EventLoopProxy. This commit also introduces `crossbeam-channel` crate which could also improve the performance.
  - [6122dc5e](https://github.com/tauri-apps/tao/commit/6122dc5ec2041225a697d9673f350bcc2dda44a3) feat: add crossbeam-channel ([#181](https://github.com/tauri-apps/tao/pull/181)) on 2021-08-12

## \[0.5.1]

- Remove feature flag that break doc builds
  - [324eca05](https://github.com/tauri-apps/tao/commit/324eca05d3075e5610f83d1161e23ace656f586e) Update README.md on 2021-05-08
  - [2a90e63c](https://github.com/tauri-apps/tao/commit/2a90e63c8efde9f291172fd89bc1564bfe336aeb) publish new versions on 2021-05-08
  - [eb6f4f9e](https://github.com/tauri-apps/tao/commit/eb6f4f9e18ae7b7537613653ade4f32cb32c5700) Remove feature flag that breaks doc build ([#178](https://github.com/tauri-apps/tao/pull/178)) on 2021-08-09

## \[0.5.0]

- Move `global_shortcut` mod to the lib root.
  - [6e72e54c](https://github.com/tauri-apps/tao/commit/6e72e54c9bb897a8f4319f3f5ec8f7d96bec75d2) refactor: move `global_shortcut` mod to lib roo ([#145](https://github.com/tauri-apps/tao/pull/145)) on 2021-07-20

- Bump gtk-rs to version 0.14. This also introduces a new feature `ayatana` for developers to use updated
  `libayatana-appindicator` since the original `libappindicator` is no longer maintained.
  - [1c0f5274](https://github.com/tauri-apps/tao/commit/1c0f5274cb7246a80b382ecf7ae3f7e6c858f2f2) chore: bump gtk to v0.14 ([#173](https://github.com/tauri-apps/tao/pull/173)) on 2021-08-06

- Remove Clipboard MenuItem on Linux since they only work on a few sepcific widget.
  - [969052ab](https://github.com/tauri-apps/tao/commit/969052abab84ff6ad7b43f39e444e944b2de862d) fix(linux): remove clipboard menuitems on Linux ([#150](https://github.com/tauri-apps/tao/pull/150)) on 2021-07-21

- Fixes incorrect monitor size on Linux.
  - [eb051931](https://github.com/tauri-apps/tao/commit/eb0519316626d02a00569176f835ec8bc4f7746e) fix(linux): incorrect monitor size, fixes: [#175](https://github.com/tauri-apps/tao/pull/175) ([#176](https://github.com/tauri-apps/tao/pull/176)) on 2021-08-08

- Fix `no key equivalent for Accelerator` for `Space`, `Escape`, `Minus` and `Equal` keycode.
  - [ecd3c405](https://github.com/tauri-apps/tao/commit/ecd3c405cd57852e3a55a9a6539784b99f32bf6c) fix(accelerator): add missing KeyCode to prevent `no key equivalent for Accelerator` ([#148](https://github.com/tauri-apps/tao/pull/148)) on 2021-07-20

- Fix incorrect macOS Redo and Close Window shortcuts
  - [f4d718a8](https://github.com/tauri-apps/tao/commit/f4d718a89a03ca0ae53817e7895acb953175ef6b) fix(macos): Fix incorrect Redo and CloseWindow accelerators ([#166](https://github.com/tauri-apps/tao/pull/166)) on 2021-08-03

- - Support [macOS tray icon template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc) to adjust automatically based on taskbar color.

- Images you mark as template images should consist of only black and clear colors. You can use the alpha channel in the image to adjust the opacity of black content, however.

- [577458c4](https://github.com/tauri-apps/tao/commit/577458c4588d667524ef483956d85a4420402bb2) feat(tray): Support macOS icon template ([#162](https://github.com/tauri-apps/tao/pull/162)) on 2021-07-29

- macOS: Add `with_parent_window()` on `WindowBuilder`.
  - [73c7aac7](https://github.com/tauri-apps/tao/commit/73c7aac7b04355862c68a42179f52c9f9dec727d) feat(macOS): Allow creation of child Window ([#160](https://github.com/tauri-apps/tao/pull/160)) on 2021-08-04

- Removed `SystemTrayExtWindows::remove()`, the icon will be automatically removed when `SystemTray` is dropped.
  - [cc9d2b17](https://github.com/tauri-apps/tao/commit/cc9d2b1726d378ee28ca69e6e5e2255c2d71f214) refactor: refactor `system_tray` impl on windows ([#153](https://github.com/tauri-apps/tao/pull/153)) on 2021-07-22

- Add `MenuItem::SelectAll` implementation on windows.
  - [222adeb2](https://github.com/tauri-apps/tao/commit/222adeb238c290b8101507aed976869c34a8d4ab) feat(window): add `Select all` native menu item ([#146](https://github.com/tauri-apps/tao/pull/146)) on 2021-07-21

- Add flags to support all other possible unix systems.
  - [546f51a3](https://github.com/tauri-apps/tao/commit/546f51a3974c15af0d6c84525d3c8e448911f8a8) Add flags to support other unix systems. ([#142](https://github.com/tauri-apps/tao/pull/142)) on 2021-07-20

- Fix confliction between `set_skip_taksbar(true)` and `set_visible(false)`.
  - [226e6611](https://github.com/tauri-apps/tao/commit/226e66113aa96495d50e8c6024afbf991f41dc7d) fix(Windows): conflict between taskbar and visible ([#172](https://github.com/tauri-apps/tao/pull/172)) on 2021-08-06

## \[0.4.0]

- On Windows, Allow resizing of `decorations: false` aka borderless window.
  - [f35dd03d](https://github.com/tauri-apps/tao/commit/f35dd03dc6f15d51fb348c6b404c195ba2401339) fix(windows): fix aero-snap and resizing of borderless window, fixes [#103](https://github.com/tauri-apps/tao/pull/103) [#104](https://github.com/tauri-apps/tao/pull/104) ([#110](https://github.com/tauri-apps/tao/pull/110)) on 2021-07-07
- Do not close the window on `CloseRequested` event and let the user handle it, keeping compatibility with macOS and Windows behavior.
  - [ea7330ef](https://github.com/tauri-apps/tao/commit/ea7330eff77dba8ab0b5e65d82313a7b15733190) fix(linux): do not close window on `CloseRequested` event ([#114](https://github.com/tauri-apps/tao/pull/114)) on 2021-07-05
- On Windows, fix Aero-Snap for `decorations: false` aka borderless window.
  - [f35dd03d](https://github.com/tauri-apps/tao/commit/f35dd03dc6f15d51fb348c6b404c195ba2401339) fix(windows): fix aero-snap and resizing of borderless window, fixes [#103](https://github.com/tauri-apps/tao/pull/103) [#104](https://github.com/tauri-apps/tao/pull/104) ([#110](https://github.com/tauri-apps/tao/pull/110)) on 2021-07-07
- Implement `MonitorHandle` and related methods on Linux.
  - [6fcfa629](https://github.com/tauri-apps/tao/commit/6fcfa62959800e6205068e74fa8648b5e12c6103) feat(linux): implement `MonitorHandle` and related methods ([#125](https://github.com/tauri-apps/tao/pull/125)) on 2021-07-12
- Add `is_menu_visilbe` getter on `Window`
  - [308411ca](https://github.com/tauri-apps/tao/commit/308411caeacc3b7c701d8d857964248a3411dfaa) feat: add `is_menu_visible` ([#108](https://github.com/tauri-apps/tao/pull/108)) on 2021-07-06
- On macOS, make sure the `set_focus` is triggered even if the window is not visible.
  - [3da167aa](https://github.com/tauri-apps/tao/commit/3da167aad9dad8ec2e3b3af52175a74a5ef07b99) fix(macos): `set_focus` should be triggered even if the window isn't visible ([#128](https://github.com/tauri-apps/tao/pull/128)) on 2021-07-14
- Fix `with_visible(bool)` in `WindowBuilder` for macOS.
  - [a0ac7075](https://github.com/tauri-apps/tao/commit/a0ac7075bdc5b9e37900c0f38b97a86071ce1dfd) fix(macos): Window state (`visible`) ([#119](https://github.com/tauri-apps/tao/pull/119)) on 2021-07-06
- Mark enums as `#[non_exhaustive]` to prevent breaking changes on enum update.
  - [9b906f50](https://github.com/tauri-apps/tao/commit/9b906f508477f0a67cc3b853909c24fe754b86c9) refactor: add `#[non_exhaustive]` attributes to enums ([#90](https://github.com/tauri-apps/tao/pull/90)) on 2021-07-07
- Remove `with_focus` and `focus` field in `WindowAttribute`. Use `set_focus` instead in most cases.
  - [e2399bc9](https://github.com/tauri-apps/tao/commit/e2399bc92642601d999a2579c0626dfe017a262c) Remove `with_focus` and `focus` field in `WindowAttribute` ([#121](https://github.com/tauri-apps/tao/pull/121)) on 2021-07-06
- Revert d344825 and move `set_skip_taskbar` back behind a `WindowExtWindows` and `WindowExtUnix`.
  - [a641d3a3](https://github.com/tauri-apps/tao/commit/a641d3a317c132661abf08723e4f87fb515e00ed) refactor: Revert d344825, move `set_skip_taskbar` behind platform-ext ([#118](https://github.com/tauri-apps/tao/pull/118)) on 2021-07-06
- `SystemTray` expose `set_menu` to update the system tray menu once created.
  - [578dd23e](https://github.com/tauri-apps/tao/commit/578dd23e02bbc63a2d2362b823730c470c1c029c) feat: implement `set_menu` for system tray ([#106](https://github.com/tauri-apps/tao/pull/106)) on 2021-07-14
- Only show window behaviour when it is visible. winuser::ShowWindow will show the window and make with_visible(false) obsolete.
  - [ff0903f6](https://github.com/tauri-apps/tao/commit/ff0903f62b7e206fd9018a7140f5d2729d4ab8ba) Only show window behaviour when it is visible ([#126](https://github.com/tauri-apps/tao/pull/126)) on 2021-07-14
- Add `with_skip_taskbar` behind `WindowBuilderExtWindows` and `WindowBuilderExtUnix`.
  - [e7cdb950](https://github.com/tauri-apps/tao/commit/e7cdb950c719a0efd93538324ba8773dd2c7abcc) feat(taskbar): add `with_skip_taskbar` for windows and linux ([#127](https://github.com/tauri-apps/tao/pull/127)) on 2021-07-14

## \[0.3.1]

- Add `window_id` to `MenuEvent`.
  - [96651dcc](https://github.com/tauri-apps/tao/commit/96651dccd2f81229a6a7d8a0fc8ffb122c099b30) feat(menu): Add `window_id` to `MenuEvent` ([#89](https://github.com/tauri-apps/tao/pull/89)) on 2021-06-22
- Prevent duplicate `MenuEvent` on window menu in Windows.
  - [8cf4033f](https://github.com/tauri-apps/tao/commit/8cf4033f81e1b0cdfd88e86cf34cdcd174e1a3a9) fix(windows): menu event ([#91](https://github.com/tauri-apps/tao/pull/91)) on 2021-06-22

## \[0.3.0]

- Drop the event callback before exiting on macOS.
  - [52ebebbc](https://github.com/tauri-apps/tao/commit/52ebebbc5cc2913bb4b50ea7743e9f6be21e6657) Drop the event callback before exiting ([#86](https://github.com/tauri-apps/tao/pull/86)) on 2021-06-18
- Add `clipboard` api exposing `read_text` and `write_text`.
  - [cf22c902](https://github.com/tauri-apps/tao/commit/cf22c902d4c961be0f6cfba6a8c865e11073b027) feat: clipboard api ([#85](https://github.com/tauri-apps/tao/pull/85)) on 2021-06-21
- Fix LoopDestroyed to really exit the application.
  - [55e52a91](https://github.com/tauri-apps/tao/commit/55e52a9149adf37131e365fa667a97f9a24f1e4f) Fix LoopDestroy condition to really exit the app on 2021-06-01
- Implement all control flow variants
  - [16e2ac06](https://github.com/tauri-apps/tao/commit/16e2ac06c2f0c180a17fb8021005dec51a57af49) Add change file on 2021-05-19
- Add checks before focusing window
  - [1bd3b1c0](https://github.com/tauri-apps/tao/commit/1bd3b1c0fbdbf4e8a9bfade34a7e217f26114859) Add change file on 2021-05-22
- Add `is_visible` getter on `Window`
  - [c402a38b](https://github.com/tauri-apps/tao/commit/c402a38b1bbf240f4d2553c3f2b4edf86f03c270) feat: Add `is_visible` getter to `Window` ([#61](https://github.com/tauri-apps/tao/pull/61)) on 2021-05-27
- **Breaking change**: New keyboard API, including `Accelerator` and `GlobalShortcut`.

`WindowEvent::ModifiersChanged` is emitted when a new keyboard modifier is pressed. This is your responsibility to keep a local state. When the modifier is released, `ModifiersState::empty()` is emitted.

`WindowEvent::KeyboardInput` as been refactored and is exposing the event `KeyEvent`.

All menus (`ContextMenu` and `MenuBar`), now includes `Accelerator` support on Windows, macOS and Linux.

New modules available: `keyboard`, `accelerator` and `platform::global_shortcut`.

*Please refer to the docs and examples for more details.*

- [01fc43b0](https://github.com/tauri-apps/tao/commit/01fc43b05ea41463d512c0e3497971edc543ac9d) refactor: keyboards events ([#82](https://github.com/tauri-apps/tao/pull/82)) on 2021-06-21
- **Breaking change**: New menu/tray API.

System tray now expose `set_icon()` to update the tray icon after initialization. The `system_tray::SystemTrayBuilder` has been moved to the root of the package as a module and available on Windows, Linux and macOS, only when the `tray` feature is enabled. Windows expose a `remove()` function available with `SystemTrayExtWindows`.

Menu builder has been rebuilt from scratch, exposing 2 different types, `ContextMenu` and `MenuBar`.

Please refer to the docs and examples for more details.

- [7546dbd1](https://github.com/tauri-apps/tao/commit/7546dbd157b5387249337c5166849e2868c0ab7c) refactor: menu & tray ([#77](https://github.com/tauri-apps/tao/pull/77)) on 2021-06-03
- Fix match branch of run loop observer on iOS.
  - [4e9fede6](https://github.com/tauri-apps/tao/commit/4e9fede6b63d4cf60ea0a6b974a61a00bd2b47df) Add change file on 2021-05-23
- - `skip_taskbar` is renamed to `set_skip_taskbar`.
- `set_skip_taskbar` is now available on `Window` and is no longer behind a PlatformExt.
- `set_skip_taskbar` takes a boolean to either show or hide the window icon from the taskbar.
- Add `with_skip_taskbar` to `WindowBuilder`.
- [c0aac091](https://github.com/tauri-apps/tao/commit/c0aac0911e77b8b0b709d20fa9d1a3297b38e7ee) add `with_skip_taskbar` on 2021-05-29
- Add `skip_taskbar` implementation for windows
  - [83341701](https://github.com/tauri-apps/tao/commit/83341701843122f57139cf7583db345385b81d3c) feat: add `skip_taskabr` impl for windows ([#78](https://github.com/tauri-apps/tao/pull/78)) on 2021-05-29

## \[0.2.6]

- Add `is_decorated` getter on `Window`
  - [8237e2f3](https://github.com/tauri-apps/tao/commit/8237e2f3d54a012f3a89aea84112fb125863b582) add changefile on 2021-05-13
- Add `is_resizable` getter on `Window`
  - [c87f3bf9](https://github.com/tauri-apps/tao/commit/c87f3bf925df3692a44982c72cfea7484758bccc) add changefile on 2021-05-13
- Fix panic from borrowing in event loop on linux.
  - [12d7ccbc](https://github.com/tauri-apps/tao/commit/12d7ccbc4dd31ae35a45dd56542c5377d5235ecc) Fix event loop on linux on 2021-05-17
- Implement `set_focus()` and `with_focus()` for macOS, Windows and Linux.
  - [448e4c17](https://github.com/tauri-apps/tao/commit/448e4c17e2ba58257b6c23041faeae0551189536) Add change file on 2021-05-07

## \[0.2.5]

- Fix Priority import on Linux.
  - [20128896](https://github.com/tauri-apps/tao/commit/201288960e6af87a2e4ae9a75b3d53a874edbd3e) Add change file on 2021-05-17

## \[0.2.4]

- Refactor control flow implementation to wait.
  - [f5514f04](https://github.com/tauri-apps/tao/commit/f5514f04819f13493e75dfc844c7b06ec17bb071) Add change file on 2021-05-15

## \[0.2.3]

- Split feature flags (menu and tray).
  - [0035ac31](https://github.com/tauri-apps/tao/commit/0035ac31ea32fd1b04339a678e321411b779a5b6) Add changefile on 2021-05-10

## \[0.2.2]

- Add dox flag to skip link lib when building doc.
  - [565114c1](https://github.com/tauri-apps/tao/commit/565114c16a27b321e326195ad1f248ffa721c3a3) Add dox flag on 2021-05-09

## \[0.2.1]

- Update covector script to fix doc build.
  - [25f291f2](https://github.com/tauri-apps/tao/commit/25f291f2385b9505b31b8db2c74fd2aad4b7d699) Update covector script to fix doc build on 2021-05-09

## \[0.2.0]

- Update README and bump version.
  - [324eca05](https://github.com/tauri-apps/tao/commit/324eca05d3075e5610f83d1161e23ace656f586e) Update README.md on 2021-05-08
- Implement menu item varients for Linux.
  - [0637570f](https://github.com/tauri-apps/tao/commit/0637570f151f3ad6d5fcaa37dbacb84a5b53624a) Add change file on 2021-05-06
- Implement status bar on Linux.
  - [e17bce40](https://github.com/tauri-apps/tao/commit/e17bce40df35f4b8e9ff018a5e9b14f446eee899) Add change file on 2021-05-07
  - [86743720](https://github.com/tauri-apps/tao/commit/867437209f820f461239505201e7b21d8d66495c) \[skip ci] Update change file description on 2021-05-07
- Implement basic menu builder for macOS, Windows and Linux.
  - [ecd528d6](https://github.com/tauri-apps/tao/commit/ecd528d6c137c7d3ca8d5f16bb3f082b961db6d9) Add change file on 2021-05-04
  - [47640216](https://github.com/tauri-apps/tao/commit/476402167ce7209b57a180453733132b777c44f5) \[skip ci] Update changelog on 2021-05-05
- Add menu feature flag and rename status bar to system tray.
  - [06d95ad0](https://github.com/tauri-apps/tao/commit/06d95ad03c15b3da1601007ef1b81ea90310d177) Cargo fmt & clippy on 2021-05-08
- Implement basic menu builder for macOS, Windows and Linux.
  - [63868365](https://github.com/tauri-apps/tao/commit/63868365613bfd3f6c6d2155e9b3120f7fb9962e) Add change file on 2021-05-05
  - [94254074](https://github.com/tauri-apps/tao/commit/942540745e3c8365ac4d0813e9ae40dc7090a2cd) \[ci skip] Update changelog on 2021-05-06
  - [e17bce40](https://github.com/tauri-apps/tao/commit/e17bce40df35f4b8e9ff018a5e9b14f446eee899) Add change file on 2021-05-07
  - [86743720](https://github.com/tauri-apps/tao/commit/867437209f820f461239505201e7b21d8d66495c) \[skip ci] Update change file description on 2021-05-07
