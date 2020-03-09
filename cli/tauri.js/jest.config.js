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
    '<rootDir>/bin/**/*.js',
    '<rootDir>/helpers/**/*.js',
    '<rootDir>/api/**/*.js'
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
  testPathIgnorePatterns: [
    '(build|dev).spec.js'
  ],
  moduleFileExtensions: ['ts', 'js', 'json'],
  moduleNameMapper: {
    '^~/(.*)$': '<rootDir>/$1',
    '^bin/(.*)$': '<rootDir>/bin/$1',
    '^helpers/(.*)$': '<rootDir>/src/helpers/$1',
    '^api/(.*)$': '<rootDir>/src/api/$1',
    '^templates/(.*)$': '<rootDir>/src/templates/$1',
    '^test/(.*)$': '<rootDir>/test/$1',
    '../../package.json': '<rootDir>/package.json'
  },
  "transform": {
    "templates[\\/]tauri.js": "./test/jest/raw-loader-transformer.js",
    "\\.(js|ts)$": "babel-jest"
  }
}
