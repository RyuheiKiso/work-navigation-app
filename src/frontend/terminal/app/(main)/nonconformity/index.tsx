// SCR-HA-014 不適合登録。カテゴリ + 4M（Man/Machine/Material/Method）選択
import React, { useState } from 'react';
import { ScrollView, StyleSheet, Text, TextInput, TouchableOpacity, View } from 'react-native';
import { useRouter } from 'expo-router';
import { WNavButton } from '../../../ui/WNavButton';
import { AndonKaizenFlow } from '../../../domain/andon/AndonKaizenFlow';
import { useAuth } from '../../../contexts/AuthContext';
import { useWorkExecution } from '../../../contexts/WorkExecutionContext';

const NC_TYPES = ['process_deviation', 'material_defect', 'measurement_out_of_spec', 'document_error'] as const;
const FOUR_M = ['MAN', 'MACHINE', 'MATERIAL', 'METHOD'] as const;

export default function NonconformityScreen(): JSX.Element {
  const router = useRouter();
  const { state: auth } = useAuth();
  const { state: exec } = useWorkExecution();
  const [ncType, setNcType] = useState<(typeof NC_TYPES)[number]>('process_deviation');
  const [cause, setCause] = useState<(typeof FOUR_M)[number]>('MAN');
  const [description, setDescription] = useState('');

  const handleSubmit = async (): Promise<void> => {
    const flow = new AndonKaizenFlow();
    await flow.registerNonconformity({
      ncType,
      description: `[${cause}] ${description}`,
      discoveredBy: auth.user?.userId ?? 'unknown',
      ...(exec.workExecutionId !== null ? { workExecutionId: exec.workExecutionId } : {}),
      evidenceIds: [],
    });
    router.replace('/(main)/home');
  };

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.title} accessibilityRole="header">
        不適合登録
      </Text>

      <Text style={styles.label}>カテゴリ</Text>
      <View style={styles.row}>
        {NC_TYPES.map((t) => (
          <TouchableOpacity
            key={t}
            accessibilityRole="radio"
            accessibilityState={{ selected: ncType === t }}
            onPress={() => setNcType(t)}
            style={[styles.chip, ncType === t ? styles.chipSelected : null]}
          >
            <Text style={[styles.chipText, ncType === t ? styles.chipTextSelected : null]}>{t}</Text>
          </TouchableOpacity>
        ))}
      </View>

      <Text style={styles.label}>4M 原因</Text>
      <View style={styles.row}>
        {FOUR_M.map((c) => (
          <TouchableOpacity
            key={c}
            accessibilityRole="radio"
            accessibilityState={{ selected: cause === c }}
            onPress={() => setCause(c)}
            style={[styles.chip, cause === c ? styles.chipSelected : null]}
          >
            <Text style={[styles.chipText, cause === c ? styles.chipTextSelected : null]}>{c}</Text>
          </TouchableOpacity>
        ))}
      </View>

      <Text style={styles.label}>説明</Text>
      <TextInput
        accessibilityLabel="不適合の説明"
        value={description}
        onChangeText={setDescription}
        multiline
        numberOfLines={4}
        style={styles.input}
      />
      <WNavButton
        label="登録"
        accessibilityLabel="不適合を登録"
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
  chipSelected: { backgroundColor: '#1E3A5F', borderColor: '#1E3A5F' },
  chipText: { fontSize: 16, color: '#1E293B' },
  chipTextSelected: { color: '#FFFFFF', fontWeight: '700' },
  input: {
    borderWidth: 2,
    borderColor: '#CBD5E1',
    borderRadius: 12,
    padding: 12,
    minHeight: 100,
    fontSize: 16,
    textAlignVertical: 'top',
  },
});
