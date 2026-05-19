// SCR-HA-015 設定画面。言語切替・ネットワーク状態・縮退モード
import React from 'react';
import { ScrollView, StyleSheet, Text, TouchableOpacity, View } from 'react-native';
import { useTranslation } from 'react-i18next';
import { useLocale } from '../../../contexts/LocaleContext';
import { useNetwork } from '../../../contexts/NetworkContext';
import type { Locale } from '@wnav/shared';
import { WNavButton } from '../../../ui/WNavButton';
import { useAuth } from '../../../contexts/AuthContext';

const LOCALES: { code: Locale; label: string }[] = [
  { code: 'ja', label: '日本語' },
  { code: 'en', label: 'English' },
  { code: 'zh', label: '中文' },
];

export default function SettingsScreen(): JSX.Element {
  const { t } = useTranslation();
  const { locale, setLocale } = useLocale();
  const { state: net } = useNetwork();
  const { logout } = useAuth();

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.title} accessibilityRole="header">
        {t('settings.title')}
      </Text>

      <Text style={styles.label}>{t('settings.language')}</Text>
      <View style={styles.row}>
        {LOCALES.map((item) => (
          <TouchableOpacity
            key={item.code}
            accessibilityRole="radio"
            accessibilityLabel={`${item.label} に切替`}
            accessibilityState={{ selected: locale === item.code }}
            onPress={() => setLocale(item.code)}
            style={[styles.chip, locale === item.code ? styles.chipSelected : null]}
          >
            <Text style={[styles.chipText, locale === item.code ? styles.chipTextSelected : null]}>
              {item.label}
            </Text>
          </TouchableOpacity>
        ))}
      </View>

      <Text style={styles.label}>{t('settings.network')}</Text>
      <Text style={styles.body}>品質: {net.quality}</Text>
      <Text style={styles.body}>緊急モード: {net.isEmergencyMode ? 'ON' : 'OFF'}</Text>
      <Text style={styles.body}>保留件数: {net.pendingSyncCount}</Text>

      <WNavButton
        label="ログアウト"
        accessibilityLabel="ログアウト"
        variant="danger"
        onPress={logout}
      />
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { padding: 16 },
  title: { fontSize: 22, fontWeight: '700', marginBottom: 12 },
  label: { fontSize: 14, fontWeight: '600', marginTop: 8, marginBottom: 6 },
  row: { flexDirection: 'row', gap: 8 },
  chip: {
    minHeight: 72,
    paddingHorizontal: 18,
    justifyContent: 'center',
    borderWidth: 2,
    borderColor: '#CBD5E1',
    borderRadius: 12,
  },
  chipSelected: { backgroundColor: '#1E3A5F', borderColor: '#1E3A5F' },
  chipText: { fontSize: 16, color: '#1E293B' },
  chipTextSelected: { color: '#FFFFFF', fontWeight: '700' },
  body: { fontSize: 16, marginVertical: 4 },
});
