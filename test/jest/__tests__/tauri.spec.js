const { tauri } = require('mode/bin/tauri')

describe('[CLI] tauri.js', () => {
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

  it('will not run an unavailable command', async () => {
    jest.spyOn(console, 'log')
    tauri('foo')
    expect(console.log.mock.calls[0][0].split('.')[0]).toBe('Invalid command foo')
    jest.clearAllMocks()
  })

  it('will pass on an available command', async () => {
    jest.spyOn(console, 'log')
    tauri('init')
    expect(console.log.mock.calls[0][0].split('.')[0]).toBe('[tauri]: running init')
    jest.clearAllMocks()
  })
})
