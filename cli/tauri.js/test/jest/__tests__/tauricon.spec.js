const { tauricon } = require('helpers/tauricon')
const { tauri } = require('bin/tauri')

describe('[CLI] tauri-icon internals', () => {
  it('tells you the version', () => {
    const version = tauricon.version()
    expect(!!version).toBe(true)
  })
  it('gets you help', async () => {
    jest.spyOn(console, 'log')
    tauri(['icon', 'help'])
    expect(!!console.log.mock.calls[0][0]).toBe(true)
    jest.clearAllMocks()
  })

  it('will not validate a non-file', async () => {
    try {
      await tauricon.validate('test/jest/fixtures/doesnotexist.png', 'test/jest/fixtures/')
    } catch (e) {
      expect(e.message).toBe('[ERROR] Source image for tauricon not found')
    }
  })
  it('will not validate a non-png', async () => {
    try {
      await tauricon.validate('test/jest/fixtures/notAMeme.jpg', 'test/jest/fixtures/')
    } catch (e) {
      expect(e.message).toBe('[ERROR] Source image for tauricon is not a png')
    }
  })
  it('can validate an image as PNG', async () => {
    const valid = await tauricon.validate('test/jest/fixtures/tauri-logo.png', 'test/jest/fixtures/')
    expect(valid).toBe(true)
  })
})

/**
 * This test suite takes A LOT of time. Maybe 5 minutes...? You may blame
 * Zopfli, but don't blame us for trying to help you get the smallest
 * possible binaries!
 */
describe('[CLI] tauri-icon builder', () => {
  it('makes a set of icons with pngquant', async () => {
    const valid = await tauricon.make('test/jest/fixtures/tauri-logo.png', 'test/jest/tmp/pngquant', 'pngquant')
    expect(valid).toBe(true)
  })

  it('makes a set of icons with optipng', async () => {
    const valid = await tauricon.make('test/jest/fixtures/tauri-logo.png', 'test/jest/tmp/optipng', 'optipng')
    expect(valid).toBe(true)
  })

  it('makes a set of icons with zopfli', async () => {
    jest.setTimeout(120000)
    const valid = await tauricon.make('test/jest/fixtures/tauri-logo.png', 'test/jest/tmp/zopfli', 'zopfli')
    expect(valid).toBe(true)
  })
})
