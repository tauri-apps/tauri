// Copyright 2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
#import <Foundation/Foundation.h>
#import <Security/Security.h>
#include <unistd.h>

// Mirrors Chromium's DynamicCodeObjectForCurrentProcess and
// OuterBundleCachedInfoPlistData. No Keychain access or signing key required.
int main(void) {
  @autoreleasepool {
    NSMutableDictionary *info = [[[NSBundle mainBundle] infoDictionary] mutableCopy];
    [info removeObjectForKey:@"CFBundleNumericVersion"];
    NSData *xml = [NSPropertyListSerialization dataWithPropertyList:info
        format:NSPropertyListXMLFormat_v1_0 options:0 error:nil];
    NSDictionary *attributes = @{
      (__bridge NSString *)kSecGuestAttributePid: @(getpid()),
      (__bridge NSString *)kSecGuestAttributeDynamicCode: @YES,
      (__bridge NSString *)kSecGuestAttributeDynamicCodeInfoPlist: xml,
    };
    SecCodeRef code = NULL;
    OSStatus status = SecCodeCopyGuestWithAttributes(NULL,
        (__bridge CFDictionaryRef)attributes, kSecCSDefaultFlags, &code);
    if (status == errSecSuccess) {
      status = SecStaticCodeCheckValidity(code, kSecCSBasicValidateOnly, NULL);
      CFRelease(code);
    }
    printf("%d\n", (int)status);
    return 0;
  }
}
