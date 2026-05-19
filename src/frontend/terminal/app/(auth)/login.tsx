// SCR-HA-001 ログイン画面。ユーザー名・パスワード入力 → AuthContext へ dispatch
import React, { useState } from 'react';
import { ScrollView, StyleSheet, Text, TextInput, View } from 'react-native';
import { useRouter } from 'expo-router';
import { useTranslation } from 'react-i18next';
import Constants from 'expo-constants';
import * as Application from 'expo-application';
import { useAuth } from '../../contexts/AuthContext';
import { WNavButton } from '../../ui/WNavButton';
import { JwtService } from '../../auth/JwtService';
import type { UserRole, Locale } from '@wnav/shared';

export default function LoginScreen(): JSX.Element {
  const { state, dispatch } = useAuth();
  const router = useRouter();
  const { t } = useTranslation();
  const [loginId, setLoginId] = useState('');
  const [password, setPassword] = useState('');

  const apiBaseUrl =
    (Constants.expoConfig?.extra?.['apiBaseUrl'] as string | undefined) ?? 'http://localhost:8080/api/v1';
  const jwt = new JwtService({ baseApiUrl: apiBaseUrl });

  const handleLogin = async (): Promise<void> => {
    dispatch({ type: 'LOGIN_START' });
    try {
      const deviceId = (await Application.getIosIdForVendorAsync()) ?? Application.getAndroidId() ?? 'unknown';
      const data = await jwt.login({ loginId, password, deviceId, factoryId: 'default' });
      const role: UserRole = (data.roles[0] as UserRole | undefined) ?? 'operator';
      dispatch({
        type: 'LOGIN_SUCCESS',
        payload: {
          token: data.accessToken,
          refreshToken: data.refreshToken,
          user: {
            userId: data.userId,
            displayName: loginId,
            role,
            roles: data.roles,
            locale: 'ja' satisfies Locale,
            factoryId: data.factoryId,
          },
        },
      });
      router.replace('/(main)/home');
    } catch (err) {
      const message = err instanceof Error ? err.message : t('auth.loginFailed');
      dispatch({ type: 'LOGIN_FAILURE', payload: { errCode: 'ERR-AUTH-001', message } });
    }
  };

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.title} accessibilityRole="header">
        {t('auth.loginTitle')}
      </Text>
      <View style={styles.field}>
        <Text style={styles.label}>{t('auth.loginId')}</Text>
        <TextInput
          accessibilityLabel={t('auth.loginId')}
          value={loginId}
          onChangeText={setLoginId}
          autoCapitalize="none"
          style={styles.input}
        />
      </View>
      <View style={styles.field}>
        <Text style={styles.label}>{t('auth.password')}</Text>
        <TextInput
          accessibilityLabel={t('auth.password')}
          value={password}
          onChangeText={setPassword}
          secureTextEntry
          style={styles.input}
        />
      </View>
      {state.error !== null ? (
        <Text style={styles.error} accessibilityRole="alert">
          {state.error}
        </Text>
      ) : null}
      <WNavButton
        label={t('auth.login')}
        accessibilityLabel={t('auth.login')}
        onPress={() => {
          void handleLogin();
        }}
        loading={state.isLoading}
      />
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { padding: 24, gap: 16 },
  title: { fontSize: 28, fontWeight: '700', marginBottom: 24 },
  field: { marginBottom: 12 },
  label: { fontSize: 14, marginBottom: 4 },
  input: {
    minHeight: 72,
    borderWidth: 2,
    borderColor: '#CBD5E1',
    borderRadius: 12,
    paddingHorizontal: 16,
    fontSize: 20,
  },
  error: { color: '#DC2626', marginBottom: 12, fontWeight: '600' },
});
