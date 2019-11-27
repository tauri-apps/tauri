'use strict'

/**
 * This is a module that takes an original image and resizes
 * it to common icon sizes and will put them in a folder.
 * It will retain transparency and can make special file
 * types. You can control the settings.
 *
 * @module tauricon
 * @exports tauricon
 * @author Daniel Thompson-Yvetot
 * @license MIT
 */

const path = require('path')
const sharp = require('sharp')
const imagemin = require('imagemin')
const pngquant = require('imagemin-pngquant')
const optipng = require('imagemin-optipng')
const zopfli = require('imagemin-zopfli')
const png2icons = require('png2icons')
const readChunk = require('read-chunk')
const isPng = require('is-png')

const settings = require('./tauricon.config')
let image = false

const {
  access,
  writeFileSync,
  ensureDir,
  ensureFileSync
} = require('fs-extra')

const exists = async function (file) {
  try {
    await access(file)
    return true
  } catch (err) {
    return false
  }
}

/**
 * This is the first call that attempts to memoize the sharp(src).
 * If the source image cannot be found or if it is not a png, it
 * is a failsafe that will exit or throw.
 *
 * @param {string} src - a folder to target
 * @exits {error} if not a png, if not an image
 */
const checkSrc = async function (src) {
  if (image !== false) {
    return image
  } else {
    const srcExists = await exists(src)
    if (!srcExists) {
      image = false
      throw new Error('[ERROR] Source image for tauricon not found')
    } else {
      const buffer = await readChunk(src, 0, 8)
      if (isPng(buffer) === true) {
        return (image = sharp(src))
      } else {
        image = false
        throw new Error('[ERROR] Source image for tauricon is not a png')
        // exit because this is BAD!
        // Developers should catch () { } this as it is
        // the last chance to stop bad things happening.
      }
    }
  }
}

/**
 * Sort the folders in the current job for unique folders.
 *
 * @param {object} options - a subset of the settings
 * @returns {array} folders
 */
const uniqueFolders = function (options) {
  let folders = []
  for (const type in options) {
    if (options[type].folder) {
      folders.push(options[type].folder)
    }
  }
  folders = folders.sort().filter((x, i, a) => !i || x !== a[i - 1])
  return folders
}

/**
 * Turn a hex color (like #212342) into r,g,b values
 *
 * @param {string} hex - hex colour
 * @returns {array} r,g,b
 */
const hexToRgb = function (hex) {
  // https://stackoverflow.com/questions/5623838/rgb-to-hex-and-hex-to-rgb
  // Expand shorthand form (e.g. "03F") to full form (e.g. "0033FF")
  const shorthandRegex = /^#?([a-f\d])([a-f\d])([a-f\d])$/i
  hex = hex.replace(shorthandRegex, function (m, r, g, b) {
    return r + r + g + g + b + b
  })

  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex)
  return result
    ? {
      r: parseInt(result[1], 16),
      g: parseInt(result[2], 16),
      b: parseInt(result[3], 16)
    }
    : null
}

/**
 * validate image and directory
 * @param {string} src
 * @param {string} target
 * @returns {Promise<void>}
 */
const validate = async function (src, target) {
  if (target !== undefined) {
    await ensureDir(target)
  }
  return checkSrc(src)
}

/**
 * Log progress in the command line
 *
 * @param {string} msg
 * @param {boolean} end
 */
const progress = function (msg) {
  console.log(msg)
  // process.stdout.write(`  ${msg}                       \r`)
}

/**
 * Create a spinner on the command line
 *
 * @example
 *
 *     const spinnerInterval = spinner()
 *     // later
 *     clearInterval(spinnerInterval)
 * @returns {function} - the interval object
 */
const spinner = function () {
  return setInterval(() => {
    process.stdout.write('/ \r')
    setTimeout(() => {
      process.stdout.write('- \r')
      setTimeout(() => {
        process.stdout.write('\\ \r')
        setTimeout(() => {
          process.stdout.write('| \r')
        }, 100)
      }, 100)
    }, 100)
  }, 500)
}

const tauricon = exports.tauricon = {
  validate: async function (src, target) {
    await validate(src, target)
    return typeof image === 'object'
  },
  version: function () {
    return require('../../package.json').version
  },
  /**
   *
   * @param {string} src
   * @param {string} target
   * @param {string} strategy
   * @param {object} options
   */
  make: async function (src, target, strategy, options) {
    const spinnerInterval = spinner()
    options = options || settings.options.tauri
    await this.validate(src, target)
    progress('Building Tauri icns and ico')
    await this.icns(src, target, options, strategy)
    progress('Building Tauri png icons')
    await this.build(src, target, options)
    if (strategy) {
      progress(`Minifying assets with ${strategy}`)
      await this.minify(target, options, strategy, 'batch')
    } else {
      console.log('no minify strategy')
    }
    progress('Tauricon Finished')
    clearInterval(spinnerInterval)
    return true
  },

  /**
   * Creates a set of images according to the subset of options it knows about.
   *
   * @param {string} src - image location
   * @param {string} target - where to drop the images
   * @param {object} options - js object that defines path and sizes
   */
  build: async function (src, target, options) {
    await this.validate(src, target) // creates the image object
    const buildify2 = async function (pvar) {
      try {
        const pngImage = image.resize(pvar[1], pvar[1])
        if (pvar[2]) {
          const rgb = hexToRgb(options.background_color)
          pngImage.flatten({
            background: { r: rgb.r, g: rgb.g, b: rgb.b, alpha: 1 }
          })
        }
        pngImage.png()
        await pngImage.toFile(pvar[0])
      } catch (err) {
        console.log(err)
      }
    }

    let output
    const folders = uniqueFolders(options)
    for (const n in folders) {
      // make the folders first
      ensureDir(`${target}${path.sep}${folders[n]}`)
    }
    for (const optionKey in options) {
      const option = options[optionKey]
      // chain up the transforms
      for (const sizeKey in option.sizes) {
        const size = option.sizes[sizeKey]
        if (!option.splash) {
          const dest = `${target}/${option.folder}`
          if (option.infix === true) {
            output = `${dest}${path.sep}${option.prefix}${size}x${size}${option.suffix}`
          } else {
            output = `${dest}${path.sep}${option.prefix}${option.suffix}`
          }
          const pvar = [output, size, option.background]
          await buildify2(pvar)
        }
      }
    }
  },
  /**
   * Creates a set of splash images (COMING SOON!!!)
   *
   * @param {string} src - icon location
   * @param {string} splashSrc - splashscreen location
   * @param {string} target - where to drop the images
   * @param {object} options - js object that defines path and sizes
   */
  splash: async function (src, splashSrc, target, options) {
    let output
    let block = false
    const rgb = hexToRgb(options.background_color)

    // three options
    // options: splashscreen_type [generate | overlay | pure]
    //          - generate (icon + background color) DEFAULT
    //          - overlay (icon + splashscreen)
    //          - pure (only splashscreen)

    let sharpSrc
    if (splashSrc === src) {
      // prevent overlay or pure
      block = true
    }
    if (block === true || options.splashscreen_type === 'generate') {
      await this.validate(src, target)
      if (!image) {
        process.exit(1)
      }
      sharpSrc = sharp(src)
      sharpSrc.extend({
        top: 726,
        bottom: 726,
        left: 726,
        right: 726,
        background: {
          r: rgb.r,
          g: rgb.g,
          b: rgb.b,
          alpha: 1
        }
      })
        .flatten({ background: { r: rgb.r, g: rgb.g, b: rgb.b, alpha: 1 } })
    } else if (options.splashscreen_type === 'overlay') {
      sharpSrc = sharp(splashSrc)
        .flatten({ background: { r: rgb.r, g: rgb.g, b: rgb.b, alpha: 1 } })
        .composite([{
          input: src
          // blend: 'multiply' <= future work, maybe just a gag
        }])
    } else if (options.splashscreen_type === 'pure') {
      sharpSrc = sharp(splashSrc)
        .flatten({ background: { r: rgb.r, g: rgb.g, b: rgb.b, alpha: 1 } })
    }

    const data = await sharpSrc.toBuffer()

    for (const optionKey in options) {
      const option = options[optionKey]
      for (const sizeKey in option.sizes) {
        const size = option.sizes[sizeKey]
        if (option.splash) {
          const dest = `${target}${path.sep}${option.folder}`
          await ensureDir(dest)

          if (option.infix === true) {
            output = `${dest}${path.sep}${option.prefix}${size}x${size}${option.suffix}`
          } else {
            output = `${dest}${path.sep}${option.prefix}${option.suffix}`
          }
          // console.log('p1', output, size)
          const pvar = [output, size]
          let sharpData = sharp(data)
          sharpData = sharpData.resize(pvar[1][0], pvar[1][1])
          await sharpData.toFile(pvar[0])
        }
      }
    }
  },

  /**
   * Minifies a set of images
   *
   * @param {string} target - image location
   * @param {object} options - where to drop the images
   * @param {string} strategy - which minify strategy to use
   * @param {string} mode - singlefile or batch
   */
  minify: async function (target, options, strategy, mode) {
    let cmd
    const minify = settings.options.minify
    if (!minify.available.find(x => x === strategy)) {
      strategy = minify.type
    }
    switch (strategy) {
      case 'pngquant':
        cmd = pngquant(minify.pngquantOptions)
        break
      case 'optipng':
        cmd = optipng(minify.optipngOptions)
        break
      case 'zopfli':
        cmd = zopfli(minify.zopfliOptions)
        break
    }

    const __minifier = async (pvar) => {
      await imagemin([pvar[0]], {
        destination: pvar[1],
        plugins: [cmd]
      }).catch(err => {
        console.log(err)
      })
    }
    switch (mode) {
      case 'singlefile':
        await __minifier([target, path.dirname(target)], cmd)
        break
      case 'batch':
        // eslint-disable-next-line no-case-declarations
        const folders = uniqueFolders(options)
        for (const n in folders) {
          console.log('batch minify:', folders[n])
          await __minifier([
            `${target}${path.sep}${folders[n]}${path.sep}*.png`,
            `${target}${path.sep}${folders[n]}`
          ], cmd)
        }
        break
      default:
        console.error('* [ERROR] Minify mode must be one of [ singlefile | batch]')
        process.exit(1)
    }
    return 'minified'
  },

  /**
   * Creates special icns and ico filetypes
   *
   * @param {string} src - image location
   * @param {string} target - where to drop the images
   * @param {object} options
   * @param {string} strategy
   */
  icns: async function (src, target, options, strategy) {
    try {
      if (!image) {
        process.exit(1)
      }
      await this.validate(src, target)

      const sharpSrc = sharp(src)
      const buf = await sharpSrc.toBuffer()

      const out = await png2icons.createICNS(buf, png2icons.BICUBIC, 0)
      ensureFileSync(path.join(target, '/icon.icns'))
      writeFileSync(path.join(target, '/icon.icns'), out)

      const out2 = await png2icons.createICO(buf, png2icons.BICUBIC, 0, true)
      ensureFileSync(path.join(target, '/icon.ico'))
      writeFileSync(path.join(target, '/icon.ico'), out2)
    } catch (err) {
      console.error(err)
    }
  }
}

if (typeof exports !== 'undefined') {
  if (typeof module !== 'undefined' && module.exports) {
    exports = module.exports = tauricon
  }
  exports.tauricon = tauricon
}
