// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const HEADERS = `// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT`

const fs = require('fs')

for (const file of ['index.js', 'index.d.ts']) {
  const content = fs.readFileSync(file, 'utf8')
  const newContent = `${HEADERS}\n\n${content}`
  fs.writeFileSync(file, newContent, 'utf8')
}
