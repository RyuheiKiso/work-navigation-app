// SCR-HA-001 ログイン画面の基本レンダリング
import React from 'react';
import { render } from '@testing-library/react-native';
import LoginScreen from '../../app/(auth)/login';
import { AuthProvider } from '../../contexts/AuthContext';

// expo-router の useRouter を Stub する
jest.mock('expo-router', () => ({
  useRouter: () => ({ replace: jest.fn(), push: jest.fn(), back: jest.fn() }),
}));

// expo-constants の extra を Stub する
jest.mock('expo-constants', () => ({
  default: { expoConfig: { extra: { apiBaseUrl: 'http://test.local/api/v1' } } },
}));

// expo-application の deviceId 取得を Stub する
jest.mock('expo-application', () => ({
  getIosIdForVendorAsync: () => Promise.resolve('test-vendor-id'),
  getAndroidId: () => 'test-android-id',
}));

// react-i18next の useTranslation を Stub する
jest.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe('LoginScreen', () => {
  it('renders login form with accessibility labels', () => {
    const { getByLabelText } = render(
      <AuthProvider>
        <LoginScreen />
      </AuthProvider>,
    );
    expect(getByLabelText('auth.loginId')).toBeTruthy();
    expect(getByLabelText('auth.password')).toBeTruthy();
    expect(getByLabelText('auth.login')).toBeTruthy();
  });
});
