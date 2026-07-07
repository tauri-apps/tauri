# Changelog

## \[0.24.1]

- [`7adc007`](https://www.github.com/tauri-apps/tray-icon/commit/7adc007bb54a91e7e6de4baac87ae534e1aa0550) ([#325](https://www.github.com/tauri-apps/tray-icon/pull/325) by [@Y-ASLant](https://www.github.com/tauri-apps/tray-icon/../../Y-ASLant)) On Windows, preserve tray icon visibility on Explorer restart so a hidden tray icon won't become visible.

## \[0.24.0]

- [`19bcab3`](https://www.github.com/tauri-apps/tray-icon/commit/19bcab382753cbbb883b7d6ae96d07111fc73e67) ([#305](https://www.github.com/tauri-apps/tray-icon/pull/305) by [@expenses](https://www.github.com/tauri-apps/tray-icon/../../expenses)) Make gtk an optional feature (enabled by default)

## \[0.23.1]

- [`7e5953c`](https://www.github.com/tauri-apps/tray-icon/commit/7e5953c227065f92dc3113268c86d6d0f05739a4) ([#311](https://www.github.com/tauri-apps/tray-icon/pull/311) by [@Fullzoon](https://www.github.com/tauri-apps/tray-icon/../../Fullzoon)) Fixed a Windows tray menu issue reported in #303 where the context menu could appear behind the taskbar after launching from the Start Menu by aligning the tray menu handling with the expected shell behavior, including showing the right-click menu on button release and posting `WM_NULL` after `TrackPopupMenu`.

## \[0.23.0]

- [`b74676a`](https://www.github.com/tauri-apps/tray-icon/commit/b74676a487cd74d1cf12b2d355a91473ac476369) ([#312](https://www.github.com/tauri-apps/tray-icon/pull/312) by [@lucasfernog](https://www.github.com/tauri-apps/tray-icon/../../lucasfernog)) Updated `muda` to 0.19.

## \[0.22.2]

- [`215e972`](https://www.github.com/tauri-apps/tray-icon/commit/215e97221e3a9e9cbc71b29c10bcfb234e1cc022) ([#262](https://www.github.com/tauri-apps/tray-icon/pull/262) by [@renovate](https://www.github.com/tauri-apps/tray-icon/../../renovate)) Relaxed `windows-sys` dependency to `>=0.60, <=0.61` instead of `0.60`

## \[0.22.1]

- [`eabe0f4`](https://www.github.com/tauri-apps/tray-icon/commit/eabe0f48f89c3df1cde48e8111d46bd9fec1ca30) ([#306](https://www.github.com/tauri-apps/tray-icon/pull/306) by [@Legend-Master](https://www.github.com/tauri-apps/tray-icon/../../Legend-Master)) Fix tray icon gets blurry after changing dpi on Windows

## \[0.22.0]

- [`f912921`](https://www.github.com/tauri-apps/tray-icon/commit/f912921a56584251dc0b5670b5281cbf7e4106b0) ([#294](https://www.github.com/tauri-apps/tray-icon/pull/294) by [@felipecrs](https://www.github.com/tauri-apps/tray-icon/../../felipecrs)) Added `with_menu_on_right_click` builder method, `set_show_menu_on_right_click` to control whether the context menu is shown on right click (analogous to the existing left click option), and `show_menu()` to programmatically display the tray menu.

  Together these enable dynamic menu workflows where the menu content is updated before being shown, for example by disabling automatic right-click menu, listening for the click event, updating items, and then calling `show_menu()`.
- [`0a5835b`](https://www.github.com/tauri-apps/tray-icon/commit/0a5835b0e6828e37a1f781de9c2d671ae7a939e6) ([#295](https://www.github.com/tauri-apps/tray-icon/pull/295) by [@thief-sty](https://www.github.com/tauri-apps/tray-icon/../../thief-sty)) Update png dependency version to 0.18.

  This avoids duplicated dependencies in downstream crates.
- [`0a5835b`](https://www.github.com/tauri-apps/tray-icon/commit/0a5835b0e6828e37a1f781de9c2d671ae7a939e6) ([#295](https://www.github.com/tauri-apps/tray-icon/pull/295) by [@thief-sty](https://www.github.com/tauri-apps/tray-icon/../../thief-sty)) Update rust version to 1.73.

## \[0.21.3]

- [`d3c2468`](https://www.github.com/tauri-apps/tray-icon/commit/d3c24683fd63d197fa064bfbf205ad2fb464e338) ([#293](https://www.github.com/tauri-apps/tray-icon/pull/293) by [@Slinetrac](https://www.github.com/tauri-apps/tray-icon/../../Slinetrac)) Handle Windows tray icon creation when the taskbar is not ready by keeping the message window alive and re-registering on TaskbarCreated.
- [`cb22cd5`](https://www.github.com/tauri-apps/tray-icon/commit/cb22cd5df6b0938aaeebd6c302ec50bc696d8b1a) ([#290](https://www.github.com/tauri-apps/tray-icon/pull/290) by [@FabianLars](https://www.github.com/tauri-apps/tray-icon/../../FabianLars)) The default `id` is now derived from the process id to prevent issues when multiple open apps use the `tray-icon` crate on Linux.

## \[0.21.2]

- [`c9f0d21`](https://www.github.com/tauri-apps/tray-icon/commit/c9f0d2108202b3743efb7696a334759b403b9856) ([#278](https://www.github.com/tauri-apps/tray-icon/pull/278) by [@aarol](https://www.github.com/tauri-apps/tray-icon/../../aarol)) Fix infinite loops when printing the full error chain (for example "{err:?}" with anyhow) for `BadIcon`
- [`4bbabba`](https://www.github.com/tauri-apps/tray-icon/commit/4bbabba49cd82e7eb8b5f58cfdfe2b00812517af) ([#277](https://www.github.com/tauri-apps/tray-icon/pull/277) by [@ancwrd1](https://www.github.com/tauri-apps/tray-icon/../../ancwrd1)) Fix a problem on Windows platform where the created window was not destroyed correctly in case the tray icon creation fails.
- [`343599f`](https://www.github.com/tauri-apps/tray-icon/commit/343599f27fb59bf393793140a09511f1331aeda6) ([#284](https://www.github.com/tauri-apps/tray-icon/pull/284) by [@FabianLars](https://www.github.com/tauri-apps/tray-icon/../../FabianLars)) This hotfix reverts https://github.com/tauri-apps/tray-icon/pull/268 because it caused `assertion failed: flush_paint_messages` panics.

## \[0.21.1]

- [`1470763`](https://www.github.com/tauri-apps/tray-icon/commit/147076368596bac8626f51d70563ce3a1b048cfb) ([#272](https://www.github.com/tauri-apps/tray-icon/pull/272) by [@ahonn](https://www.github.com/tauri-apps/tray-icon/../../ahonn)) Add platform specific methods to access the underlying native handles of the tray (similar to `TrayIcon::window_handle`):

  - `TrayIcon::ns_status_item` for macOS
  - `TrayIcon::app_indicator` for Linux.
- [`2a6a19b`](https://www.github.com/tauri-apps/tray-icon/commit/2a6a19bbac9848d3887b12599b2860ba6653c8f7) ([#268](https://www.github.com/tauri-apps/tray-icon/pull/268) by [@iKineticate](https://www.github.com/tauri-apps/tray-icon/../../iKineticate)) On Windows, fix tray menu freeze on device change events

## \[0.21.0]

- [`986007b`](https://www.github.com/tauri-apps/tray-icon/commit/986007b4113f3050d65b6fd1bb43d2bd4b97dcc1) ([#263](https://www.github.com/tauri-apps/tray-icon/pull/263) by [@renovate](https://www.github.com/tauri-apps/tray-icon/../../renovate)) Updated `muda` to `0.17`.

## \[0.20.1]

- [`47fc3f3`](https://www.github.com/tauri-apps/tray-icon/commit/47fc3f3ef8ad9a60476d6cf7b5eaa87c621c380a) ([#248](https://www.github.com/tauri-apps/tray-icon/pull/248) by [@aschey](https://www.github.com/tauri-apps/tray-icon/../../aschey)) Add `window_handle` method to retrieve the underlying `hwnd` on Windows
- [`1f28d6f`](https://www.github.com/tauri-apps/tray-icon/commit/1f28d6fc6bc52dfcea0358bae17d9119ec99c4b6) ([#241](https://www.github.com/tauri-apps/tray-icon/pull/241) by [@Legend-Master](https://www.github.com/tauri-apps/tray-icon/../../Legend-Master)) Correct `with_menu_on_left_click`'s docs to include window support

## \[0.20.0]

- [`e94976b`](https://www.github.com/tauri-apps/tray-icon/commit/e94976bb48bbe97ac5ab215c6da7c7ca746a5c8a) ([#237](https://www.github.com/tauri-apps/tray-icon/pull/237) by [@renovate](https://www.github.com/tauri-apps/tray-icon/../../renovate)) Updated `muda` to 0.16.0

## \[0.19.3]

- [`d6fee6f`](https://www.github.com/tauri-apps/tray-icon/commit/d6fee6ff627e6ff08bf8bf9a2880197d0f07271e) ([#226](https://www.github.com/tauri-apps/tray-icon/pull/226) by [@madsmtm](https://www.github.com/tauri-apps/tray-icon/../../madsmtm)) Update `objc2` to v0.6.

## \[0.19.2]

- [`1f0e1f8`](https://www.github.com/tauri-apps/tray-icon/commit/1f0e1f8f0d0ad65cd0ab549655fac26c0f524de6) ([#204](https://www.github.com/tauri-apps/tray-icon/pull/204) by [@mrexox](https://www.github.com/tauri-apps/tray-icon/../../mrexox)) Add `set_icon_with_as_template` method to update icon and `is_template` property, preventing glitchy effects during icon animation on macOS.

## \[0.19.1]

- [`19e67de`](https://www.github.com/tauri-apps/tray-icon/commit/19e67de6ff0b66241fddde507eb82e96781b6c36) ([#199](https://www.github.com/tauri-apps/tray-icon/pull/199) by [@Klemen2](https://www.github.com/tauri-apps/tray-icon/../../Klemen2)) Implemented `TrayIcon::set_show_menu_on_left_click` on windows

## \[0.19.0]

- [`bf5cec4`](https://www.github.com/tauri-apps/tray-icon/commit/bf5cec4c3242534cb068978bb27e37551bcb63f9) ([#196](https://www.github.com/tauri-apps/tray-icon/pull/196) by [@amrbashir](https://www.github.com/tauri-apps/tray-icon/../../amrbashir)) **Breaking change** Changed `serde` derive implementation for `TrayIconEvent` to use `serde(tag = "type")` and `rename_all = "camelCase"` on variants so the expected JSON serialization would look like this

  ```json
  {
    "type": "Click",
    "button": "Left",
    "buttonState": "Down",
    "id": "some id",
    "position": {
      "x": 0,
      "y": 0
    },
    "rect": {
      "size": {
        "width": 0,
        "height": 0
      },
      "position": {
        "x": 0,
        "y": 0
      }
    }
  }
  ```

## \[0.18.0]

- [`c63733c`](https://www.github.com/tauri-apps/tray-icon/commit/c63733c45f5fd34bc16c9310cb4f1a063e5e21c7) ([#193](https://www.github.com/tauri-apps/tray-icon/pull/193) by [@amrbashir](https://www.github.com/tauri-apps/tray-icon/../../amrbashir)) Update `muda` crate to `0.15`

## \[0.17.0]

- [`e711c1f`](https://www.github.com/tauri-apps/tray-icon/commit/e711c1f5e6aef3052694ee8da33b2de624093ec8) ([#189](https://www.github.com/tauri-apps/tray-icon/pull/189) by [@htngr](https://www.github.com/tauri-apps/tray-icon/../../htngr)) Implemented `TrayIcon::with_menu_on_left_click` on windows

## \[0.16.0]

- [`20819e4`](https://www.github.com/tauri-apps/tray-icon/commit/20819e445e23f1d3749d03534eba9641404a8db6) ([#187](https://www.github.com/tauri-apps/tray-icon/pull/187) by [@amrbashir](https://www.github.com/tauri-apps/tray-icon/../../amrbashir)) Removed `button_state` field in `TrayIconEvent::DoubleClick` variant.

## \[0.15.2]

- [`a1303c3`](https://www.github.com/tauri-apps/tray-icon/commit/a1303c39020befc977d818d7cce1a039d416ccd2) ([#185](https://www.github.com/tauri-apps/tray-icon/pull/185) by [@amrbashir](https://www.github.com/tauri-apps/tray-icon/../../amrbashir)) On Windows, Add and emit `DoubleClick` variant for `TrayIconEvent`.

## \[0.15.1]

- [`5a381ff`](https://www.github.com/tauri-apps/tray-icon/commit/5a381ffd3d0f8ab8b1a88a95e557c5837c17a1b7) Update `core-foundation` crate to `0.10` and `core-graphics` to `0.24`

## \[0.15.0]

- [`d407869`](https://www.github.com/tauri-apps/tray-icon/commit/d4078696edba67b0ab42cef67e6a421a0332c96f) ([#172](https://www.github.com/tauri-apps/tray-icon/pull/172)) Added a new variant `NotMainThread` to the `Error` enum, which is emitted on macOS when trying to create tray icons from a thread that is not the main thread.
- [`8857b7d`](https://www.github.com/tauri-apps/tray-icon/commit/8857b7dd12ba523532ac7a58bf08302316af13c8) Updated `muda` crate to `0.14`
- [`d407869`](https://www.github.com/tauri-apps/tray-icon/commit/d4078696edba67b0ab42cef67e6a421a0332c96f) ([#172](https://www.github.com/tauri-apps/tray-icon/pull/172)) Rewrite the internals of the crate to use `objc2` instead of `objc`.

  This should have no user-facing changes, other than improved memory safety, and less leaking.

## \[0.14.3]

- [`e257d6b`](https://www.github.com/tauri-apps/tray-icon/commit/e257d6bf510b34707d48964a2914ee5c91b13570) ([#169](https://www.github.com/tauri-apps/tray-icon/pull/169)) On Windows, fix `Enter` event emitted only once and never emitted again.

## \[0.14.2]

- [`f1f3adb`](https://www.github.com/tauri-apps/tray-icon/commit/f1f3adb5ec726335226ab8ec1d8c6c41012cb9c5)([#166](https://www.github.com/tauri-apps/tray-icon/pull/166)) Switch from `dirs_next` to `dirs` as `dirs_next` is now unmaintained while `dirs` is

## \[0.14.1]

- [`b491c98`](https://www.github.com/tauri-apps/tray-icon/commit/b491c9886619d3a26876476b078d99a0ae788918)([#164](https://www.github.com/tauri-apps/tray-icon/pull/164)) Fix tray icon rect scaled by dpi on Windows

## \[0.14.0]

- [`587292b`](https://www.github.com/tauri-apps/tray-icon/commit/587292b2e7bfbebdd2677c51b34c6362730d5111)([#161](https://www.github.com/tauri-apps/tray-icon/pull/161)) This release contains **breaking change** to the event structs in order to be able to add new `Enter`, `Move` and `Leave` events:

  - Changed `TrayIconEvent` to be an enum instead of a struct.
  - Added new events for when the mouse enters, moves or leaves the tray icon region.
  - Removed `ClickType` enum and replaced it with `MouseButton` enum.
  - Added `MouseButtonState` enum.

## \[0.13.5]

- [`a1cd50e`](https://www.github.com/tauri-apps/tray-icon/commit/a1cd50e53021474ad87cdf2e269acfb56d36cc14)([#145](https://www.github.com/tauri-apps/tray-icon/pull/145)) Fix tray icon gets blurry after changing dpi on Windows
- [`ad317c7`](https://www.github.com/tauri-apps/tray-icon/commit/ad317c7dab271145c641f0c4c22e283bb2aa0c91)([#150](https://www.github.com/tauri-apps/tray-icon/pull/150)) On macOS, fix tray event position not scaled properly.
- [`6d099ee`](https://www.github.com/tauri-apps/tray-icon/commit/6d099ee2a4c455561f4c6f86ea995df267469eca)([#149](https://www.github.com/tauri-apps/tray-icon/pull/149)) On macOS, fix the `y` position of the tray icon to be top-left not bottom-left of the icon.
- [`599bb8f`](https://www.github.com/tauri-apps/tray-icon/commit/599bb8f55546d674892a80051766d36656975e86)([#147](https://www.github.com/tauri-apps/tray-icon/pull/147)) Add `TrayIcon::rect` method to retrieve the tray icon rectangle on Windows and macOS.

## \[0.13.4]

- [`6b09b8e`](https://www.github.com/tauri-apps/tray-icon/commit/6b09b8e920e79d7768c3a55324431cbd0acadb27)([#136](https://www.github.com/tauri-apps/tray-icon/pull/136)) Add `Icon::from_resource_name` to support icon resource without a ordinal id on Windows

## \[0.13.3]

- [`646f56c`](https://www.github.com/tauri-apps/tray-icon/commit/646f56cb6786377b8dbae1e742bb94e7b6f1bb09)([#138](https://www.github.com/tauri-apps/tray-icon/pull/138)) Fix unexpected crashes on I/O or Png encoding errors on macOS and Linux.

## \[0.13.2]

- [`c368bbc`](https://www.github.com/tauri-apps/tray-icon/commit/c368bbc6a24b24767c902508651d856413039108)([#134](https://www.github.com/tauri-apps/tray-icon/pull/134)) Fix incorrect icon size reported in events on macOS

## \[0.13.1]

- [`784e01e`](https://www.github.com/tauri-apps/tray-icon/commit/784e01e5b4392a39fbec47f17cdcbee7f27af2bc)([#130](https://www.github.com/tauri-apps/tray-icon/pull/130)) On macOS, reset the tray icon when using `setIconAsTemplate` to avoid artifacts.

## \[0.13.0]

- [`63abc69`](https://www.github.com/tauri-apps/tray-icon/commit/63abc69affffdd2849d3d42178d76b9bf1ea994a)([#127](https://www.github.com/tauri-apps/tray-icon/pull/127)) Update `muda` dependency to `0.13`
- [`63abc69`](https://www.github.com/tauri-apps/tray-icon/commit/63abc69affffdd2849d3d42178d76b9bf1ea994a)([#127](https://www.github.com/tauri-apps/tray-icon/pull/127)) Added `dpi` module and changed position and sizes in `TrayIconEvent` to use the new `dpi` module:

  - Removed `TrayIconEvent.x` and `TrayIconEvent.y` and replaced with `TrayIconEvent.position`
  - Replaced `Rectangle` type with `Rect` which has just two fields `position` and `size`.

## \[0.12.0]

- [`91a5bf6`](https://www.github.com/tauri-apps/tray-icon/commit/91a5bf65d7e3895e9f2eedf4e7ffaf7cc9d082ad)([#119](https://www.github.com/tauri-apps/tray-icon/pull/119)) Updated `muda` dependency to `0.12`

## \[0.11.3]

- [`5407f14`](https://www.github.com/tauri-apps/tray-icon/commit/5407f140e12aa83984f6a5402ab99e70a4d4f82c)([#114](https://www.github.com/tauri-apps/tray-icon/pull/114)) On Linux, fix `TrayIcon::set_visible` incorrect inverted behavior.

## \[0.11.2]

- [`ca3bed5`](https://www.github.com/tauri-apps/tray-icon/commit/ca3bed51b5d6e8b7e04429f8f90a2d514393b034)([#109](https://www.github.com/tauri-apps/tray-icon/pull/109)) On Windows, add `Icon::from_handle`

## \[0.11.1]

- [`6382ea5`](https://www.github.com/tauri-apps/tray-icon/commit/6382ea5b47813ce1546dff6e8a69ca053dc6f145)([#103](https://www.github.com/tauri-apps/tray-icon/pull/103)) On Linux, fix tray menu failing to show.

## \[0.11.0]

- [`6e8374a`](https://www.github.com/tauri-apps/tray-icon/commit/6e8374a81a2e84bf38c8678085986e569e517e76) Update `muda` crate to `0.11`

## \[0.10.0]

- [`8463328`](https://www.github.com/tauri-apps/tray-icon/commit/84633285a0b465fe4c261ff0c7be035ce7615715)([#92](https://www.github.com/tauri-apps/tray-icon/pull/92)) Upgraded `gtk` to 0.18 and bumped MSRV to 1.70.0.

## \[0.9.0]

- [`32b3523`](https://www.github.com/tauri-apps/tray-icon/commit/32b352371b6da730abbb024730015492f87205c0) Update `muda` crate to `0.9`

## \[0.8.3]

- [`75fed4a`](https://www.github.com/tauri-apps/tray-icon/commit/75fed4aeca82c5614777865a9f6fa2d4457f47a1) Derive `serde` for more types.

## \[0.8.2]

- [`cd6fb13`](https://www.github.com/tauri-apps/tray-icon/commit/cd6fb1300e2b2bf78781777de45302c98cfcabd4)([#80](https://www.github.com/tauri-apps/tray-icon/pull/80)) Add `PartialEq<&str> for &TrayIconId` and `PartialEq<String> for &TrayIconId` implementations. Also add a blanket `From<T> for TrayIconId` where `T: ToString` implementation.

## \[0.8.1]

- [`0cf36ad`](https://www.github.com/tauri-apps/tray-icon/commit/0cf36ad6afd1ddd93b7087e8eb4475410fb9be8a)([#77](https://www.github.com/tauri-apps/tray-icon/pull/77)) Add `TrayIconId::new` convenience method.

## \[0.8.0]

- [`95c1be8`](https://www.github.com/tauri-apps/tray-icon/commit/95c1be8a459f2ef146ccaccfe858c427678613af)([#75](https://www.github.com/tauri-apps/tray-icon/pull/75)) Th `icon` module has been removed and instead its types are exported from crate root.
- [`95c1be8`](https://www.github.com/tauri-apps/tray-icon/commit/95c1be8a459f2ef146ccaccfe858c427678613af)([#75](https://www.github.com/tauri-apps/tray-icon/pull/75)) Update to `muda@0.8`
- [`f93b57d`](https://www.github.com/tauri-apps/tray-icon/commit/f93b57d08a84a8c7ff7f9035f8cc73a3e48e90b9) Add `TrayIconId` struct an changed all `.id()` methods to return `TrayIconId` instead of a u32.
- [`95c1be8`](https://www.github.com/tauri-apps/tray-icon/commit/95c1be8a459f2ef146ccaccfe858c427678613af)([#75](https://www.github.com/tauri-apps/tray-icon/pull/75)) Changed the order of arguments for `TrayIcon::with_id` function to take the `id` as the first argument instead of the second.

## \[0.7.7]

- [`197f431`](https://www.github.com/tauri-apps/tray-icon/commit/197f43161cd1806fcae15b19b4f8335d9b3492b6)([#73](https://www.github.com/tauri-apps/tray-icon/pull/73)) Always highlight tray icon on click on macOS.

## \[0.7.6]

- [`a458317`](https://www.github.com/tauri-apps/tray-icon/commit/a458317ad1d85ac9477a019f86580a14d4082c7f)([#71](https://www.github.com/tauri-apps/tray-icon/pull/71)) Fixes a crash on mouse events on macOS.

## \[0.7.5]

- [`54fc7de`](https://www.github.com/tauri-apps/tray-icon/commit/54fc7de37c3568312b27c30bdd22e830b1f15a3b)([#69](https://www.github.com/tauri-apps/tray-icon/pull/69)) Refactor macOS implementation to fix missing click issues.

## \[0.7.4]

- [`71d25a1`](https://www.github.com/tauri-apps/tray-icon/commit/71d25a14ecd2bf0996223127b2fa01ec7f915fce)([#66](https://www.github.com/tauri-apps/tray-icon/pull/66)) On Linux, fix the issue that gtk caches the icon if you use `TrayIcon::set_icon` repeatedly.

## \[0.7.3]

- [`c0d16c5`](https://www.github.com/tauri-apps/tray-icon/commit/c0d16c5f90c3e3b4acadee9c5c83bd5e9a3671f6)([#63](https://www.github.com/tauri-apps/tray-icon/pull/63)) Fixes multiple `set_menu` calls not updating the tray menu on macOS.

## \[0.7.2]

- [`d0a25b0`](https://www.github.com/tauri-apps/tray-icon/commit/d0a25b0e980d01306344dd4903c1e2e8ef4519ac)([#61](https://www.github.com/tauri-apps/tray-icon/pull/61)) On Windows, fix dropping tray icon caused the whole process to close.
- [`d0a25b0`](https://www.github.com/tauri-apps/tray-icon/commit/d0a25b0e980d01306344dd4903c1e2e8ef4519ac)([#61](https://www.github.com/tauri-apps/tray-icon/pull/61)) On Windows, fix `TrayIcon::set_menu` not firing events for the new menu.

## \[0.7.1]

- [`04ed58f`](https://www.github.com/tauri-apps/tray-icon/commit/04ed58f954b113e1f4d52c161231d52c9f5c3546) Remove accidental impl of `Sync` and `Send` for `TrayIcon` where it is not.

## \[0.7.0]

- [`d8d6082`](https://www.github.com/tauri-apps/tray-icon/commit/d8d6082c73b1fa6047ead13d228cf7de1ad0d71c)([#57](https://www.github.com/tauri-apps/tray-icon/pull/57)) Add `TrayIconBuilder::id` to access the unique id that will be assigend to the tray icon upon creation.
- [`dd63ef3`](https://www.github.com/tauri-apps/tray-icon/commit/dd63ef3b68c35fc8b8fbc1d59975d8826420ae51) Add `TrayIconEvent::id` method.
- [`3901519`](https://www.github.com/tauri-apps/tray-icon/commit/3901519a48f76b57174b36ce36c7f803dbfb5536) Update to `muda@0.7`
- [`13d448a`](https://www.github.com/tauri-apps/tray-icon/commit/13d448a9ee7c013f0cc13391ea498da93e806551)([#55](https://www.github.com/tauri-apps/tray-icon/pull/55)) Implement `Clone` for `TrayIcon`.
- [`13d448a`](https://www.github.com/tauri-apps/tray-icon/commit/13d448a9ee7c013f0cc13391ea498da93e806551)([#55](https://www.github.com/tauri-apps/tray-icon/pull/55)) - **Breaking change**: `TrayEvent` has been renamed to `TrayIconEvent` for consistency with other struct names.
  - **Breaking change**: `ClickEvent` enum has been renamed to `ClickType` and `TrayEvent`'s `event` field has been renamed to `click_type`
- [`d8d6082`](https://www.github.com/tauri-apps/tray-icon/commit/d8d6082c73b1fa6047ead13d228cf7de1ad0d71c)([#57](https://www.github.com/tauri-apps/tray-icon/pull/57)) Add `TrayIcon::with_id` and `TrayIconBuilder::with_id` to create the tray icon with specified id.

## \[0.6.0]

- [`934b927`](https://www.github.com/tauri-apps/tray-icon/commit/934b927e552641c3d319981cdeae84ca901ae399)([#49](https://www.github.com/tauri-apps/tray-icon/pull/49)) Expose `muda` crate feature flags.

## \[0.5.2]

- [`9409f36`](https://www.github.com/tauri-apps/tray-icon/commit/9409f36c5293e7fb0c8dd7d0fd74a59472aedfcb)([#46](https://www.github.com/tauri-apps/tray-icon/pull/46)) Fix compiling on `i686-pc-windows-msvc` target

## \[0.5.1]

- [`ff7f7bc`](https://www.github.com/tauri-apps/tray-icon/commit/ff7f7bc4400a6f7aa0b5c025c85ab6c4f89e9109)([#40](https://www.github.com/tauri-apps/tray-icon/pull/40)) Fix building for `i686-pc-windows-msvc` target.

## \[0.5.0]

- On macOS, fix `set_visible(false)` still occupying space on the system menu bar.
  - [71f9d29](https://www.github.com/tauri-apps/tray-icon/commit/71f9d292dd69b498e57fcebeb76ad6a1365144cd) fix(macos): remove tray icon when `set_visible(false)` ([#37](https://www.github.com/tauri-apps/tray-icon/pull/37)) on 2023-04-18

## \[0.4.4]

- Make Rectangle's point fields public.
  - [12a0daf](https://www.github.com/tauri-apps/tray-icon/commit/12a0daf92352fbecddd7b0afdfc0c633232fb15c) Make Rectangle's point fields public. ([#33](https://www.github.com/tauri-apps/tray-icon/pull/33)) on 2023-03-23

## \[0.4.3]

- Update documentation.
  - [258b49a](https://www.github.com/tauri-apps/tray-icon/commit/258b49aaebd81b6e4327cca1a1a0a2d9bb64188a) docs: update docs on 2023-02-08
  - [3293885](https://www.github.com/tauri-apps/tray-icon/commit/3293885ae5ef19e14f2fe1baaf4d35719f3b3344) Apply Version Updates From Current Changes ([#22](https://www.github.com/tauri-apps/tray-icon/pull/22)) on 2023-02-08
  - [e58a6ce](https://www.github.com/tauri-apps/tray-icon/commit/e58a6cecfffa63096d459429c5d31ec5b3475a9b) docs: document menu and icon relation on Linux on 2023-02-12

## \[0.4.2]

- Update docs.
  - [258b49a](https://www.github.com/tauri-apps/tray-icon/commit/258b49aaebd81b6e4327cca1a1a0a2d9bb64188a) docs: update docs on 2023-02-08

## \[0.4.1]

- Bump `muda` to `0.4` and `libappindicator` to `0.8`
  - [d92dd6d](https://www.github.com/tauri-apps/tray-icon/commit/d92dd6dc25d268befe9c14cfe193e1de10bc5717) chore(deps): update deps ([#17](https://www.github.com/tauri-apps/tray-icon/pull/17)) on 2023-01-26

## \[0.4.0]

- On macOS and Linux, add `TrayIconBuilder::with_title` and `TrayIcon::set_title` to optionally add a text next to the icon.
  - [6df6fc7](https://www.github.com/tauri-apps/tray-icon/commit/6df6fc78885204be5189b41527a39324851c9671) feat: add `with_title` and `set_title` ([#11](https://www.github.com/tauri-apps/tray-icon/pull/11)) on 2023-01-10
  - [b83f14e](https://www.github.com/tauri-apps/tray-icon/commit/b83f14ee66f9d3801535697c30f54bccc433cce1) chore: adjust change bumps on 2023-01-12
- Add `TrayIcon::set_visible`.
  - [ba4580e](https://www.github.com/tauri-apps/tray-icon/commit/ba4580ec8bd061a76575859b5ead8ec16e3b7817) feat: add `set_visible` ([#14](https://www.github.com/tauri-apps/tray-icon/pull/14)) on 2023-01-12
  - [b83f14e](https://www.github.com/tauri-apps/tray-icon/commit/b83f14ee66f9d3801535697c30f54bccc433cce1) chore: adjust change bumps on 2023-01-12

## \[0.3.0]

- Add `TrayEvent::set_event_handler` to set a handler for new tray events.
  - [9247abb](https://www.github.com/tauri-apps/tray-icon/commit/9247abb69ce297096b2c388d67b250509fe44efa) refactor: allow changing the menu event sender ([#8](https://www.github.com/tauri-apps/tray-icon/pull/8)) on 2023-01-03
- Update `muda` to `0.3`.
  - [9247abb](https://www.github.com/tauri-apps/tray-icon/commit/9247abb69ce297096b2c388d67b250509fe44efa) refactor: allow changing the menu event sender ([#8](https://www.github.com/tauri-apps/tray-icon/pull/8)) on 2023-01-03
  - [b64b57e](https://www.github.com/tauri-apps/tray-icon/commit/b64b57ec565dada4bc06201f5b4529725bb0009f) chore: update changefile on 2023-01-03
- **Breaking change** Remove `tray_event_receiver` function, use `TrayEvent::receiver` instead.
  - [9247abb](https://www.github.com/tauri-apps/tray-icon/commit/9247abb69ce297096b2c388d67b250509fe44efa) refactor: allow changing the menu event sender ([#8](https://www.github.com/tauri-apps/tray-icon/pull/8)) on 2023-01-03

## \[0.2.0]

- Update `muda` dependency to `0.2`.
  - [aa3aa1e](https://www.github.com/tauri-apps/tray-icon/commit/aa3aa1ec0bdcb48ecf9d17204809802c4e6559fc) chore: add change file on 2022-12-30

## \[0.1.1]

- Initial Release.
  - [0651773](https://www.github.com/tauri-apps/tray-icon/commit/0651773ad248d34141fbefc1c65a8889a90a8c9b) chore: prepare for initial release on 2022-12-05
