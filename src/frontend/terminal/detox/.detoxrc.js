/** @type {import('detox').DetoxConfig} */
module.exports = {
  testRunner: {
    args: { $0: 'jest', config: 'detox/jest.config.js' },
    jest: { setupTimeout: 120000 },
  },
  apps: {
    'android.debug': {
      type: 'android.apk',
      binaryPath: 'android/app/build/outputs/apk/debug/app-debug.apk',
      build: 'cd android && ./gradlew assembleDebug assembleAndroidTest',
    },
    'ios.debug': {
      type: 'ios.app',
      binaryPath: 'ios/build/Build/Products/Debug-iphonesimulator/WnavTerminal.app',
      build: 'xcodebuild -workspace ios/WnavTerminal.xcworkspace -scheme WnavTerminal -configuration Debug -sdk iphonesimulator -derivedDataPath ios/build',
    },
  },
  devices: {
    'android.emulator': {
      type: 'android.emulator',
      device: { avdName: 'Pixel_7_API_34' },
    },
    'ios.simulator': {
      type: 'ios.simulator',
      device: { type: 'iPhone 15', os: 'iOS 17.4' },
    },
  },
  configurations: {
    'android.debug': {
      device: 'android.emulator',
      app: 'android.debug',
    },
    'ios.debug': {
      device: 'ios.simulator',
      app: 'ios.debug',
    },
  },
};
