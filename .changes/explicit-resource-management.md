---
"@tauri-apps/api": minor:enhance
---

Add ECMAScript Explicit Resource Management to Resource. You can now use the `using` syntax in supported browsers or with polyfills:

```javascript
import { create, BaseDirectory } from "@tauri-apps/plugin-fs"
...
{
  await using file = await create("foo/bar.txt", { baseDir: BaseDirectory.AppConfig });
  await file.write(new TextEncoder().encode("Hello world"));
  // Before `file` goes out of scope, it is disposed by calling `file[Symbol.asyncDispose]()` and awaited.
}
```
