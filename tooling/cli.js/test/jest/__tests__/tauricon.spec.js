import * as appTestSetup from '../fixtures/app-test-setup.js'
appTestSetup.initJest('app')

describe('[CLI] tauri-icon internals', () => {
  it('tells you the version', async () => {
    const tauricon = (await import('api/tauricon')).default
    const version = tauricon.version()
    expect(!!version).toBe(true)
  })
})

describe('[CLI] tauri-icon builder', () => {
  it('will still use default compression if missing compression chosen', async () => {
    const tauricon = (await import('api/tauricon')).default
    const valid = await tauricon.make(
      'test/jest/fixtures/tauri-logo.png',
      'test/jest/tmp/missing',
      'missing'
    )
    expect(valid).toBe(true)
  })

  it('will not validate a non-file', async () => {
    try {
      const tauricon = (await import('api/tauricon')).default
      await tauricon.make(
        'test/jest/fixtures/tauri-foo-not-found.png',
        'test/jest/tmp/optipng',
        'optipng'
      )
    } catch (e) {
      expect(e.message).toBe('Input file is missing')
    }
  })

  it('makes a set of icons with optipng', async () => {
    const tauricon = (await import('api/tauricon')).default
    const valid = await tauricon.make(
      'test/jest/fixtures/tauri-logo.png',
      'test/jest/tmp/optipng',
      'optipng'
    )
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
