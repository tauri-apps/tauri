import SwiftRs
import MetalKit
import os.log

@_cdecl("init_invoke")
func initInvoke() -> Invoke {
    return toRust(Invoke(sendResponse: { (success: NSDictionary?, error: NSDictionary?) -> Void in
        let log = OSLog(subsystem: "com.tauri.api", category: "com.tauri.api")
        os_log("SENDING RESPONS !!!!", log: log, type: .error)
    }, data: [:]))
}
