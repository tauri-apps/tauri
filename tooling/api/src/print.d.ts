// Copyright 2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Start printing without a user dialog
 *
 * @since 2.0.0
 */
export type OptionSilent = {
  Silent: boolean
}

/**
 * The printing margins in mm
 *
 * @since 2.0.0
 */
export type OptionMargins = {
  Margins: {
    top:    number,
    bottom: number,
    left:   number,
    right:  number
  }
}

/**
 * The printing margins in mm
 *
 * @since 2.0.0
 */
export type OptionGeneratePDF = {
  GeneratePDF: {
    filename: string
  }
}

export type PrintOption = OptionSilent | OptionMargins | OptionGeneratePDF

