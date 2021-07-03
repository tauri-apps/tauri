const isPrime = (number) => {
  if (number % 2 === 0 && number > 2) {
    return false
  }

  let start = 2
  const limit = Math.sqrt(number)
  while (start <= limit) {
    if (number % start++ < 1) {
      return false
    }
  }
  return number > 1
}

addEventListener('message', (e) => {
  const { startTime } = e.data

  let n = 0
  let total = 0
  const THRESHOLD = e.data.value
  const primes = []

  let previous = startTime

  while (++n <= THRESHOLD) {
    if (isPrime(n)) {
      primes.push(n)
      total++

      const now = Date.now()

      if (now - previous > 250) {
        previous = now
        postMessage({
          status: 'calculating',
          count: total,
          time: Date.now() - startTime
        })
      }
    }
  }

  postMessage({ status: 'done', count: total, time: Date.now() - startTime })
})
