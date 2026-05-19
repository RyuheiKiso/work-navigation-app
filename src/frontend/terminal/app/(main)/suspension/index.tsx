// SCR-HA-011 中断画面。中断理由ドロップダウン + 詳細 + 確定
import React, { useState } from 'react';
import { ScrollView, StyleSheet, Text, TextInput, TouchableOpacity, View } from 'react-native';
import { useRouter } from 'expo-router';
import { WNavButton } from '../../../ui/WNavButton';
import { SuspensionFlow, type SuspendReason } from '../../../domain/suspension/SuspensionFlow';
import { useWorkExecution } from '../../../contexts/WorkExecutionContext';

const REASONS: { code: SuspendReason; label: string }[] = [
  { code: 'equipment_breakdown', label: '設備故障' },
  { code: 'material_shortage', label: '部材不足' },
  { code: 'quality_issue', label: '品質問題' },
  { code: 'emergency', label: '緊急事態' },
  { code: 'other', label: 'その他' },
];

export default function SuspensionScreen(): JSX.Element {
  const router = useRouter();
  const { state: exec } = useWorkExecution();
  const [reason, setReason] = useState<SuspendReason>('other');
  const [detail, setDetail] = useState('');

  const handleSubmit = async (): Promise<void> => {
    if (exec.workExecutionId === null) return;
    const flow = new SuspensionFlow();
    await flow.suspend({ workExecutionId: exec.workExecutionId, reasonCode: reason, reasonDetail: detail });
    router.replace('/(main)/home');
  };

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.title} accessibilityRole="header">
        作業中断
      </Text>
      <Text style={styles.label}>中断理由</Text>
      <View style={styles.reasonGroup}>
        {REASONS.map((item) => (
          <TouchableOpacity
            key={item.code}
            accessibilityRole="radio"
            accessibilityLabel={item.label}
            accessibilityState={{ selected: reason === item.code }}
            onPress={() => setReason(item.code)}
            style={[styles.reasonChip, reason === item.code ? styles.reasonChipSelected : null]}
          >
            <Text style={[styles.reasonText, reason === item.code ? styles.reasonTextSelected : null]}>
              {item.label}
            </Text>
          </TouchableOpacity>
        ))}
      </View>
      <Text style={styles.label}>詳細コメント</Text>
      <TextInput
        accessibilityLabel="中断詳細コメント"
        value={detail}
        onChangeText={setDetail}
        multiline
        numberOfLines={4}
        style={styles.input}
      />
      <WNavButton
        label="中断を確定"
        accessibilityLabel="作業中断を確定"
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
  label: { fontSize: 14, fontWeight: '600', marginVertical: 6 },
  reasonGroup: { flexDirection: 'row', flexWrap: 'wrap', gap: 8, marginBottom: 12 },
  reasonChip: {
    minHeight: 72,
    paddingHorizontal: 18,
    justifyContent: 'center',
    borderWidth: 2,
    borderColor: '#CBD5E1',
    borderRadius: 12,
  },
  reasonChipSelected: { backgroundColor: '#1E3A5F', borderColor: '#1E3A5F' },
  reasonText: { fontSize: 16, color: '#1E293B' },
  reasonTextSelected: { color: '#FFFFFF', fontWeight: '700' },
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
