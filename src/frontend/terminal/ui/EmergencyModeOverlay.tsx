// Emergency Mode フルスクリーンオーバーレイ。NetworkContext を監視して表示する
import React from 'react';
import { StyleSheet, Text, View } from 'react-native';
import { tokens } from '@wnav/shared';
import { useNetwork } from '../contexts/NetworkContext';

export function EmergencyModeOverlay(): JSX.Element | null {
  const { state } = useNetwork();
  if (!state.isEmergencyMode) return null;

  const lastSyncDisplay = state.lastSyncedAt ?? state.lastOnlineAt ?? '未取得';

  return (
    <View accessibilityRole="alert" style={styles.overlay}>
      <Text style={styles.title} accessibilityLabel="緊急モード">
        緊急モード（オフライン）
      </Text>
      <Text style={styles.line}>サーバーへ 5 分以上接続できていません</Text>
      <Text style={styles.line}>最終同期: {lastSyncDisplay} (UTC)</Text>
      <Text style={styles.line}>キャッシュ済み SOP のみ操作可能です</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  overlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    backgroundColor: tokens.state.danger.light[700],
    padding: 16,
    zIndex: 9999,
  },
  title: { color: '#FFFFFF', fontSize: 22, fontWeight: '700', marginBottom: 8 },
  line: { color: '#FFFFFF', fontSize: 14, marginTop: 2 },
});
