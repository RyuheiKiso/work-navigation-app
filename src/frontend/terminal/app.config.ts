import type { ExpoConfig, ConfigContext } from 'expo/config';

// Expo 設定は app.json ではなく TypeScript で型安全に定義する
export default ({ config }: ConfigContext): ExpoConfig => ({
  ...config,
  name: 'WNAV Terminal',
  slug: 'wnav-terminal',
  version: '1.0.0',
  orientation: 'landscape',
  icon: './assets/icon.png',
  splash: {
    image: './assets/splash.png',
    resizeMode: 'contain',
    backgroundColor: '#1E3A5F',
  },
  // react-native-windows で Android/iOS/Windows タブレットの3OS を単一コードベースで統合する
  platforms: ['ios', 'android', 'windows'],
  ios: {
    supportsTablet: true,
    bundleIdentifier: 'local.factory.wnav.terminal',
  },
  android: {
    adaptiveIcon: {
      foregroundImage: './assets/adaptive-icon.png',
      backgroundColor: '#1E3A5F',
    },
    package: 'local.factory.wnav.terminal',
  },
  windows: {
    // Windows タブレット向け設定（react-native-windows 0.76.x）
    manifest: {
      backgroundColor: '#1E3A5F',
      displayName: 'WNAV Terminal',
    },
  } as Record<string, unknown>,
  extra: {
    apiBaseUrl: process.env['API_BASE_URL'] ?? 'http://localhost:8080/api/v1',
    eas: { projectId: process.env['EAS_PROJECT_ID'] ?? 'local' },
  },
  updates: {
    url: process.env['EXPO_UPDATES_URL'] ?? 'https://u.expo.dev/local',
    checkAutomatically: 'ON_LOAD',
    fallbackToCacheTimeout: 0,
  },
  runtimeVersion: { policy: 'appVersion' },
  plugins: [
    'expo-router',
    'expo-sqlite',
    ['expo-camera', { cameraPermission: '作業記録のためにカメラを使用します' }],
    'expo-secure-store',
    'expo-network',
    ['expo-splash-screen', { imageResizeMode: 'contain', backgroundColor: '#1E3A5F' }],
  ],
});
