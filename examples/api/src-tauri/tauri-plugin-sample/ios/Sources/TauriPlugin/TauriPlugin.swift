import SwiftRs
import MetalKit

class TauriPlugin: NSObject {
    public override init() {
        Swift.print("test")
    }
}

@_cdecl("init_plugin")
func initPlugin() -> TauriPlugin {
    return toRust(TauriPlugin())
}
