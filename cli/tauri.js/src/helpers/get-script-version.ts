// SPDX-License-Identifier: Apache-2.0 OR MIT

import { sync as spawn } from 'cross-spawn'

export default function getVersion(
  command: string,
  args: string[] = []
): string | null {
  try {
    const child = spawn(command, [...args, '--version'])
    if (child.status === 0) {
      const output = String(child.output[1])
      return output.replace(/\n/g, '')
    }
    return null
  } catch (err) {
    return null
  }
}
