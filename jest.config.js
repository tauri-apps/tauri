module.exports = {
  globals: {
    __DEV__: true
  },
  setupFilesAfterEnv: ['<rootDir>/test/jest/jest.setup.js'],
  // noStackTrace: true,
  // bail: true,
  // cache: false,
  // verbose: true,
  // watch: true,
  collectCoverage: true,
  coverageDirectory: '<rootDir>/test/jest/coverage',
  collectCoverageFrom: [
    '<rootDir>/mode/**/*.js'
  ],
  coverageReporters: ['json-summary', 'text', 'lcov'],
  coverageThreshold: {
    global: {
      //  branches: 50,
      //  functions: 50,
      //  lines: 50,
      //  statements: 50
    }
  },
  testMatch: [
    '<rootDir>/test/jest/__tests__/**/*.spec.js',
    '<rootDir>/test/jest/__tests__/**/*.test.js'
  ],
  moduleFileExtensions: ['js', 'json'],
  moduleNameMapper: {
    '^~/(.*)$': '<rootDir>/$1',
    '^mode/(.*)$': '<rootDir>/mode/$1',
    '^test/(.*)$': '<rootDir>/test/$1'
  },
  transform: {}
}
