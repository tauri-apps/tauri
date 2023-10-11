const json = process.argv[2]
const field = process.argv[3]

const output = JSON.parse(json)
console.log(output[field])
