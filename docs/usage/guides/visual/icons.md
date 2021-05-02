---
title: Icons
---

import Command from '@theme/Command'
import Alert from '@theme/Alert'

Tauri ships with a default iconset based on its logo. This is probably NOT what you want when you ship your application. To remedy this common situation, Tauri provides the `icon` command that will take an input file and create all the icons needed for the various platforms:

<Command name="icon"/>

```sh
Options
  --help, -h          Displays this message
  --log, l            Logging [boolean]
  --icon, i           Source icon (png, 1240x1240 with transparency)
  --target, t         Target folder (default: 'src-tauri/icons')
  --compression, c    Compression type [pngquant|optipng|zopfli]
```

These will be placed in your `src-tauri/icons` folder where they will automatically be included in your built app.

If you need to source your icons from some other location, you can edit this part of the `src-tauri/tauri.conf.json` file:

```json
{
  "tauri": {
    "bundle": {
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ]
    }
  }
}
```

<Alert type="info" icon="info-alt" title="Note on filetypes">

  - icon.icns = macOS
  - icon.ico = MS Windows
  - \*.png = Linux

</Alert>
