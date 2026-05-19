/** @type {import('@jest/types').Config.InitialOptions} */
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  testRegex: '\\.e2e\\.ts$',
  setupFilesAfterFramework: ['./setup.ts'],
  testTimeout: 120_000,
  globals: {
    'ts-jest': { tsconfig: '../tsconfig.json' },
  },
};
