import { RustAbility } from '@ohos-rs/ability'
import Want from '@ohos.app.ability.Want'
import { AbilityConstant } from '@kit.AbilityKit';
import window from '@ohos.window';
import display from '@ohos.display';

interface FoldStateModule {
  tauriUpdateFoldState?: (foldable: boolean | null, status: number | null) => void;
}

export default class EntryAbility extends RustAbility {
  private foldStateModule: FoldStateModule | null = null;
  private foldListening: boolean = false;
  private onFoldStatusChange = (_status: display.FoldStatus): void => {
    this.publishFoldState();
  };

  private publishFoldState(): void {
    if (!this.foldStateModule) return;
    try {
      const foldable = display.isFoldable();
      this.foldStateModule.tauriUpdateFoldState?.(foldable, foldable ? display.getFoldStatus() : 0);
    } catch (_) {
      this.foldStateModule.tauriUpdateFoldState?.(null, null);
    }
  }

  onForeground(): void {
    super.onForeground();
    this.publishFoldState();
  }

  onDestroy(): void | Promise<void> {
    if (this.foldListening) {
      try {
        display.off('foldStatusChange', this.onFoldStatusChange);
      } catch (error) {
        console.warn('Unable to remove fold-status listener', error);
      }
      this.foldListening = false;
    }
    this.foldStateModule?.tauriUpdateFoldState?.(null, null);
    this.foldStateModule = null;
    return super.onDestroy();
  }

  // change to dynamic library name
  public moduleName: string = "{{app.lib-name}}"
  public defaultPage: boolean = true;
  public mode: 'xcomponent' | 'webview' = 'webview'

  async onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): Promise<void> {
    await super.onCreate(want, launchParam);
    const native = loadNativeModule(`lib${this.moduleName}.so`) as FoldStateModule;
    if (typeof native.tauriUpdateFoldState === 'function') {
      this.foldStateModule = native;
      try {
        display.on('foldStatusChange', this.onFoldStatusChange);
        this.foldListening = true;
      } catch (_) {
        // Unsupported devices still get snapshots on creation and foreground.
      }
      // Subscribe before sampling so a change during initialization is not lost.
      this.publishFoldState();
    }
  }

  async onWindowStageCreate(windowStage: window.WindowStage): Promise<void> {
    const window = windowStage.getMainWindowSync();
    await window.setWindowLayoutFullScreen(false);
    super.onWindowStageCreate(windowStage);
  }
}
