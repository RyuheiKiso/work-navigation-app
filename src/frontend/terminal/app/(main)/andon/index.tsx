// SCR-HA-013 アンドン発報。発報種別・詳細入力 → POST /alerts
import React, { useState } from 'react';
import { ScrollView, StyleSheet, Text, TextInput, TouchableOpacity, View } from 'react-native';
import { useRouter } from 'expo-router';
import { WNavButton } from '../../../ui/WNavButton';
import { AndonKaizenFlow } from '../../../domain/andon/AndonKaizenFlow';
import { useAuth } from '../../../contexts/AuthContext';
import { useWorkExecution } from '../../../contexts/WorkExecutionContext';

const TYPES = ['quality', 'safety', 'equipment', 'process'] as const;
const SEVERITIES = ['low', 'medium', 'high', 'critical'] as const;

export default function AndonScreen(): JSX.Element {
  const router = useRouter();
  const { state: auth } = useAuth();
  const { state: exec } = useWorkExecution();
  const [alertType, setAlertType] = useState<(typeof TYPES)[number]>('quality');
  const [severity, setSeverity] = useState<(typeof SEVERITIES)[number]>('medium');
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');

  const handleSubmit = async (): Promise<void> => {
    const flow = new AndonKaizenFlow();
    await flow.raiseAndon({
      alertType,
      severity,
      raisedBy: auth.user?.userId ?? 'unknown',
      title,
      description,
      ...(exec.workExecutionId !== null ? { workExecutionId: exec.workExecutionId } : {}),
    });
    router.replace('/(main)/home');
  };

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.title} accessibilityRole="header">
        アンドン発報
      </Text>

      <Text style={styles.label}>発報種別</Text>
      <View style={styles.row}>
        {TYPES.map((t) => (
          <TouchableOpacity
            key={t}
            accessibilityRole="radio"
            accessibilityLabel={`${t} を選択`}
            accessibilityState={{ selected: alertType === t }}
            onPress={() => setAlertType(t)}
            style={[styles.chip, alertType === t ? styles.chipSelected : null]}
          >
            <Text style={[styles.chipText, alertType === t ? styles.chipTextSelected : null]}>{t}</Text>
          </TouchableOpacity>
        ))}
      </View>

      <Text style={styles.label}>重大度</Text>
      <View style={styles.row}>
        {SEVERITIES.map((s) => (
          <TouchableOpacity
            key={s}
            accessibilityRole="radio"
            accessibilityLabel={`${s} を選択`}
            accessibilityState={{ selected: severity === s }}
            onPress={() => setSeverity(s)}
            style={[styles.chip, severity === s ? styles.chipSelected : null]}
          >
            <Text style={[styles.chipText, severity === s ? styles.chipTextSelected : null]}>{s}</Text>
          </TouchableOpacity>
        ))}
      </View>

      <Text style={styles.label}>タイトル</Text>
      <TextInput accessibilityLabel="タイトル" value={title} onChangeText={setTitle} style={styles.input} />
      <Text style={styles.label}>説明</Text>
      <TextInput
        accessibilityLabel="説明"
        value={description}
        onChangeText={setDescription}
        multiline
        numberOfLines={4}
        style={[styles.input, styles.multi]}
      />
      <WNavButton
        label="発報"
        accessibilityLabel="アンドン発報を確定"
        variant="danger"
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
  label: { fontSize: 14, fontWeight: '600', marginTop: 8, marginBottom: 6 },
  row: { flexDirection: 'row', flexWrap: 'wrap', gap: 8 },
  chip: {
    minHeight: 72,
    paddingHorizontal: 18,
    justifyContent: 'center',
    borderWidth: 2,
    borderColor: '#CBD5E1',
    borderRadius: 12,
  },
  chipSelected: { backgroundColor: '#DC2626', borderColor: '#DC2626' },
  chipText: { fontSize: 16, color: '#1E293B' },
  chipTextSelected: { color: '#FFFFFF', fontWeight: '700' },
  input: {
    borderWidth: 2,
    borderColor: '#CBD5E1',
    borderRadius: 12,
    padding: 12,
    minHeight: 72,
    fontSize: 16,
  },
  multi: { minHeight: 100, textAlignVertical: 'top' },
});
