import { RustAbility } from '@ohos-rs/ability'
import Want from '@ohos.app.ability.Want'
import { AbilityConstant } from '@kit.AbilityKit';
import window from '@ohos.window';

interface BackwardCapable {
  backwardIfPossible: () => boolean;
}

export default class EntryAbility extends RustAbility {
  // change to dynamic library name
  public moduleName: string = "{{app.lib-name}}"
  public defaultPage: boolean = true;
  public mode: 'xcomponent' | 'webview' = 'webview'

  async onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): Promise<void> {
    super.onCreate(want, launchParam);
  }

  async onWindowStageCreate(windowStage: window.WindowStage): Promise<void> {
    const window = windowStage.getMainWindowSync();
    await window.setWindowLayoutFullScreen(false);
    super.onWindowStageCreate(windowStage);
  }

  /**
   * Back key handling at the ability level. The page-level onBackPress
   * (MainPage) navigates the WebView back first; when there is no history
   * the event reaches here and we move the app to the background instead of
   * destroying it (true keeps the instance alive, false destroys it). The
   * controller is registered to AppStorage by DefaultXComponent under this
   * key when the WebView is created (this SDK exposes no
   * getAllWebControllers / findControllerByName static API).
   */
  onBackPressed(): boolean {
    try {
      const controller = AppStorage.get<BackwardCapable>(
        "ohos.rs.ability.webviewController",
      );
      if (controller && typeof controller.backwardIfPossible === "function") {
        controller.backwardIfPossible();
      }
    } catch (e) {
      console.error('[EntryAbility] back handling failed:', e);
    }
    return true;
  }
}
