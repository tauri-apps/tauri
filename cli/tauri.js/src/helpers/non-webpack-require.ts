// SPDX-License-Identifier: Apache-2.0 OR MIT

// this function has been moved to a module so we can mock it
export default (path: string): any => {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-return
  return __non_webpack_require__(path)
}
