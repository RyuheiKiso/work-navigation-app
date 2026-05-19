// SCR-HA-020 修正品 再検査。別 worker_id チェック（自分自身禁止）
import React, { useState } from 'react';
import { ScrollView, StyleSheet, Switch, Text, TextInput, View } from 'react-native';
import { useRouter } from 'expo-router';
import { WNavButton } from '../../../ui/WNavButton';
import { QrScannerOverlay } from '../../../ui/QrScannerOverlay';
import { ReworkFlow } from '../../../domain/rework/ReworkFlow';
import { useAuth } from '../../../contexts/AuthContext';

export default function ReworkReInspectionScreen(): JSX.Element {
  const router = useRouter();
  const { state: auth } = useAuth();
  const [reworkId, setReworkId] = useState('');
  const [note, setNote] = useState('');
  const [passed, setPassed] = useState(false);
  const [scanning, setScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleScanned = (value: string): void => {
    setReworkId(value);
    setScanning(false);
  };

  const handleSubmit = async (): Promise<void> => {
    try {
      const flow = new ReworkFlow();
      await flow.verifyRework({
        reworkId,
        verifierId: auth.user?.userId ?? 'unknown',
        passed,
        note,
        evidenceIds: [],
      });
      router.replace('/(main)/home');
    } catch (err) {
      setError(err instanceof Error ? err.message : '失敗');
    }
  };

  if (scanning) {
    return <QrScannerOverlay onScanned={handleScanned} onCancel={() => setScanning(false)} />;
  }

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.title} accessibilityRole="header">
        修正品の再検査
      </Text>
      <Text style={styles.label}>修正 ID</Text>
      <TextInput accessibilityLabel="修正 ID" value={reworkId} onChangeText={setReworkId} style={styles.input} />
      <WNavButton
        label="QR スキャン"
        accessibilityLabel="修正 ID を QR でスキャン"
        variant="secondary"
        onPress={() => setScanning(true)}
      />
      <View style={styles.row}>
        <Text style={styles.label}>合格</Text>
        <Switch accessibilityLabel="合格判定" value={passed} onValueChange={setPassed} />
      </View>
      <Text style={styles.label}>判定メモ</Text>
      <TextInput
        accessibilityLabel="判定メモ"
        value={note}
        onChangeText={setNote}
        multiline
        numberOfLines={3}
        style={[styles.input, styles.multi]}
      />
      {error !== null ? (
        <Text style={styles.error} accessibilityRole="alert">
          {error}
        </Text>
      ) : null}
      <WNavButton
        label="検査結果を登録"
        accessibilityLabel="再検査の結果を登録"
        onPress={() => {
          void handleSubmit();
        }}
      />
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { padding: 16 },
  title: { fontSize: 22, fontWeight: '700', marginBottom: 12 },
  label: { fontSize: 14, marginTop: 8, marginBottom: 4 },
  row: { flexDirection: 'row', alignItems: 'center', gap: 12, marginVertical: 8 },
  input: {
    minHeight: 72,
    borderWidth: 2,
    borderColor: '#CBD5E1',
    borderRadius: 12,
    paddingHorizontal: 16,
    fontSize: 18,
  },
  multi: { minHeight: 100, textAlignVertical: 'top' },
  error: { color: '#DC2626', fontWeight: '600', marginVertical: 8 },
});
