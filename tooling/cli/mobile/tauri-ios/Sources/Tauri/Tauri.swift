import SwiftRs
import MetalKit

@_cdecl("init_invoke")
func initInvoke() -> Invoke {
    return toRust(Invoke(sendResponse: { (success: NSDictionary?, error: NSDictionary?) -> Void in
        
    }, data: [:]))
}
