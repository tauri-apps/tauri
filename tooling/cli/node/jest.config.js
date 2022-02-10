module.exports = {
  setupFilesAfterEnv: ['<rootDir>/test/jest/jest.setup.js'],
  testMatch: [
    '<rootDir>/test/jest/__tests__/**/*.spec.js',
    '<rootDir>/test/jest/__tests__/**/*.test.js'
  ],
  moduleFileExtensions: ['ts', 'js', 'json'],
  moduleNameMapper: {
    '^~/(.*)$': '<rootDir>/$1'
  },
  transform: {
    '\\.toml$': 'jest-transform-toml'
  }
}
