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
   * Back key handling: when any WebView can navigate back, go back and
   * consume the event instead of exiting the app; fall through to the
   * system default when there is no history. The controller is registered
   * by the openharmony-ability DefaultXComponent under this key when the
   * WebView is created (this SDK exposes no getAllWebControllers /
   * findControllerByName static API).
   */
  onBackPressed(): boolean {
    try {
      const controller = AppStorage.get<BackwardCapable>(
        "ohos.rs.ability.webviewController",
      );
      if (controller && typeof controller.backwardIfPossible === "function") {
        return controller.backwardIfPossible();
      }
    } catch (e) {
      console.error('[EntryAbility] back handling failed:', e);
    }
    return false;
  }
}
