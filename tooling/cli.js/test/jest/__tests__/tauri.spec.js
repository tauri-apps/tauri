const { tauri } = require('bin/tauri')

describe('[CLI] cli.js', () => {
  it('displays a help message', async () => {
    jest.spyOn(console, 'log')
    jest.spyOn(process, 'exit').mockImplementation(() => true)
    tauri('help')
    console.log(process.exit.mock.calls[0][0])
    expect(process.exit.mock.calls[0][0]).toBe(0)
    expect(!!console.log.mock.calls[0][0]).toBe(true)
    tauri('--help')
    expect(!!console.log.mock.calls[2][0]).toBe(true)
    tauri('-h')
    expect(!!console.log.mock.calls[3][0]).toBe(true)
    tauri(['help'])
    expect(!!console.log.mock.calls[4][0]).toBe(true)
    jest.clearAllMocks()
  })

  it('gets you help', async () => {
    jest.spyOn(console, 'log')
    const tests = ['--help', '-h']
    for (const test of tests) {
      tauri([test])
      expect(!!console.log.mock.calls[0][0]).toBe(true)
      jest.clearAllMocks()
    }
  })
  it('gets you version', async () => {
    jest.spyOn(console, 'log')
    const tests = ['--version', '-v']
    const version = require('../../../package.json').version
    for (const test of tests) {
      tauri([test])
      expect(console.log.mock.calls[0][0]).toBe(version)
      jest.clearAllMocks()
    }
  })
})
