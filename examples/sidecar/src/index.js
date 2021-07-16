const { write, onMessage } = require('./communication')

onMessage((line) => {
  write(`read ${line}`)
})

setInterval(() => {
  write(`[${new Date().toLocaleTimeString()}] new message`)
}, 500)
