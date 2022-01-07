---
title: Frequently Asked Questions
---

# error: could not find native static libraryWebView2LoaderStatic, perhaps an -L flag is missing?

The WebView2 crate build pipeline requires `NuGet` to have a `PackageSource` to install the `Microsoft.Web.WebView2` package. If you never used `NuGet` before, you might need to create a file named `NuGet.Config` on `%APPDATA%/NuGet` folder, with the following contents:

```
<?xml version="1.0" encoding="utf-8"?>
<configuration>
  <packageSources>
    <add key="nuget.org" value="https://api.nuget.org/v3/index.json" protocolVersion="3" />
  </packageSources>
</configuration>
```

This configuration enables the default `NuGet` registry.
