jest.setTimeout(50000)

global.Promise = require('promise')

setTimeout(() => {
  // do nothing
}, 1)

require('dotenv').config({ path: '.env.jest' })
