// jest-expo プリセットを使用しつつ shared パッケージへのパスエイリアスを解決する
module.exports = {
  preset: 'jest-expo',
  setupFilesAfterFramework: ['@testing-library/jest-native/extend-expect'],
  transformIgnorePatterns: [
    'node_modules/(?!((jest-)?react-native|@react-native(-community)?)|expo(nent)?|@expo(nent)?/.*|react-navigation|@react-navigation/.*|@unimodules/.*|unimodules|native-base|react-native-svg)',
  ],
  moduleNameMapper: {
    '^@wnav/shared$': '<rootDir>/../shared/src/index.ts',
    '^@wnav/shared/(.*)$': '<rootDir>/../shared/src/$1',
  },
};
