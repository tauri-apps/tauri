const tauricon = require('~/dist/tauricon.js')

describe('[CLI] tauri-icon internals', () => {
  it('tells you the version', () => {
    const version = tauricon.version()
    expect(!!version).toBe(true)
  })

  it('will not validate a non-file', async () => {
    jest.spyOn(process, 'exit').mockImplementation(() => true)
    await tauricon.validate('test/jest/fixtures/doesnotexist.png', 'test/jest/fixtures/')
    expect(process.exit.mock.calls[0][0]).toBe(1)
    jest.clearAllMocks()
  })
  it('will not validate a non-png', async () => {
    jest.spyOn(process, 'exit').mockImplementation(() => true)
    await tauricon.validate('test/jest/fixtures/notAMeme.jpg', 'test/jest/fixtures/')
    expect(process.exit.mock.calls[0][0]).toBe(1)
    jest.clearAllMocks()
  })
  it('can validate an image as PNG', async () => {
    const valid = await tauricon.validate('test/jest/fixtures/tauri-logo.png', 'test/jest/fixtures/')
    expect(valid).toBe(true)
  })
})

describe('[CLI] tauri-icon builder', () => {
  it('will still use default compression if missing compression chosen', async () => {
    const valid = await tauricon.make('test/jest/fixtures/tauri-logo.png', 'test/jest/tmp/missing', 'missing')
    expect(valid).toBe(true)
  })
})

describe('[CLI] tauri-icon builder', () => {
  it('will not validate a non-file', async () => {
    try {
      await tauricon.make('test/jest/fixtures/tauri-foo-not-found.png', 'test/jest/tmp/pngquant', 'pngquant')
    } catch (e) {
      expect(e.message).toBe('Input file is missing')
    }
  })
})

describe('[CLI] tauri-icon builder', () => {
  it('makes a set of icons with pngquant', async () => {
    const valid = await tauricon.make('test/jest/fixtures/tauri-logo.png', 'test/jest/tmp/pngquant', 'pngquant')
    expect(valid).toBe(true)
  })

  it('makes a set of icons with optipng', async () => {
    const valid = await tauricon.make('test/jest/fixtures/tauri-logo.png', 'test/jest/tmp/optipng', 'optipng')
    expect(valid).toBe(true)
  })

  /*
  TURNED OFF BECAUSE IT TAKES FOREVER
  it('makes a set of icons with zopfli', async () => {
    jest.setTimeout(120000)
    const valid = await tauricon.make('test/jest/fixtures/tauri-logo.png', 'test/jest/tmp/zopfli', 'zopfli')
    expect(valid).toBe(true)
  })
  */
})
