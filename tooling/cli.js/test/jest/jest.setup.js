import { jest } from '@jest/globals'

// 30 minute timeout: 20 minutes wasn't always enough for compilation in GitHub Actions.
jest.setTimeout(1800000)

setTimeout(() => {
  // do nothing
}, 1)
