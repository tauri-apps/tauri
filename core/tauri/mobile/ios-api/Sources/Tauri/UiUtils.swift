// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import UIKit

public class UIUtils {
    public static func centerPopover(rootViewController: UIViewController?, popoverController: UIViewController) {
        if let viewController = rootViewController {
            popoverController.popoverPresentationController?.sourceRect = CGRectMake(viewController.view.center.x, viewController.view.center.y, 0, 0)
            popoverController.popoverPresentationController?.sourceView = viewController.view
            popoverController.popoverPresentationController?.permittedArrowDirections = UIPopoverArrowDirection.up
        }
    }
}
