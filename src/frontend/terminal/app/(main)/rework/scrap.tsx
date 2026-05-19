// SCR-HA-021 廃却処理。立会者電子サイン取得
import React, { useState } from 'react';
import { ScrollView, StyleSheet, Text, TextInput } from 'react-native';
import { useRouter } from 'expo-router';
import { generateId } from '@wnav/shared/domain/id';
import { WNavButton } from '../../../ui/WNavButton';
import { getDataSource } from '../../../db/data-source';
import { LocalScrapRecord } from '../../../db/entities/LocalScrapRecord';
import { useAuth } from '../../../contexts/AuthContext';

export default function ScrapScreen(): JSX.Element {
  const router = useRouter();
  const { state: auth } = useAuth();
  const [nonconformityId, setNonconformityId] = useState('');
  const [witnessId, setWitnessId] = useState('');
  const [quantity, setQuantity] = useState('');
  const [note, setNote] = useState('');

  const handleSubmit = async (): Promise<void> => {
    const entity: LocalScrapRecord = {
      id: generateId(),
      nonconformityId,
      scrappedBy: auth.user?.userId ?? 'unknown',
      witnessId,
      scrappedAt: new Date().toISOString(),
      quantity: Number(quantity),
      note,
    };
    await getDataSource().getRepository(LocalScrapRecord).save(entity);
    router.push('/(main)/evidence/signature');
  };

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.title} accessibilityRole="header">
        廃却処理
      </Text>
      <Text style={styles.label}>不適合 ID</Text>
      <TextInput accessibilityLabel="不適合 ID" value={nonconformityId} onChangeText={setNonconformityId} style={styles.input} />
      <Text style={styles.label}>立会者 ID</Text>
      <TextInput accessibilityLabel="立会者 ID" value={witnessId} onChangeText={setWitnessId} style={styles.input} />
      <Text style={styles.label}>数量</Text>
      <TextInput accessibilityLabel="廃却数量" keyboardType="decimal-pad" value={quantity} onChangeText={setQuantity} style={styles.input} />
      <Text style={styles.label}>備考</Text>
      <TextInput
        accessibilityLabel="備考"
        value={note}
        onChangeText={setNote}
        multiline
        numberOfLines={4}
        style={[styles.input, styles.multi]}
      />
      <WNavButton
        label="廃却を確定（署名へ）"
        accessibilityLabel="廃却を確定し署名画面へ"
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
  label: { fontSize: 14, marginTop: 8, marginBottom: 4 },
  input: {
    minHeight: 72,
    borderWidth: 2,
    borderColor: '#CBD5E1',
    borderRadius: 12,
    paddingHorizontal: 16,
    fontSize: 18,
  },
  multi: { minHeight: 100, textAlignVertical: 'top' },
});
