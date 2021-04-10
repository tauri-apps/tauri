jest.setTimeout(1200000)

global.Promise = require('promise')

setTimeout(() => {
  // do nothing
}, 1)

require('dotenv').config({ path: '.env.jest' })
