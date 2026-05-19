// SCR-HA-022 仕入先返却。追跡番号入力 + 返却伝票確認
import React, { useState } from 'react';
import { ScrollView, StyleSheet, Text, TextInput } from 'react-native';
import { useRouter } from 'expo-router';
import { generateId } from '@wnav/shared/domain/id';
import { WNavButton } from '../../../ui/WNavButton';
import { getDataSource } from '../../../db/data-source';
import { LocalReturnToVendorRecord } from '../../../db/entities/LocalReturnToVendorRecord';
import { useAuth } from '../../../contexts/AuthContext';

export default function ReturnScreen(): JSX.Element {
  const router = useRouter();
  const { state: auth } = useAuth();
  const [nonconformityId, setNonconformityId] = useState('');
  const [supplierId, setSupplierId] = useState('');
  const [trackingNo, setTrackingNo] = useState('');
  const [quantity, setQuantity] = useState('');

  const handleSubmit = async (): Promise<void> => {
    const entity: LocalReturnToVendorRecord = {
      id: generateId(),
      nonconformityId,
      supplierId,
      trackingNo,
      returnedBy: auth.user?.userId ?? 'unknown',
      returnedAt: new Date().toISOString(),
      quantity: Number(quantity),
    };
    await getDataSource().getRepository(LocalReturnToVendorRecord).save(entity);
    router.push('/(main)/evidence/signature');
  };

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.title} accessibilityRole="header">
        仕入先返却
      </Text>
      <Text style={styles.label}>不適合 ID</Text>
      <TextInput accessibilityLabel="不適合 ID" value={nonconformityId} onChangeText={setNonconformityId} style={styles.input} />
      <Text style={styles.label}>仕入先 ID</Text>
      <TextInput accessibilityLabel="仕入先 ID" value={supplierId} onChangeText={setSupplierId} style={styles.input} />
      <Text style={styles.label}>追跡番号</Text>
      <TextInput accessibilityLabel="追跡番号" value={trackingNo} onChangeText={setTrackingNo} style={styles.input} />
      <Text style={styles.label}>返却数量</Text>
      <TextInput accessibilityLabel="返却数量" keyboardType="decimal-pad" value={quantity} onChangeText={setQuantity} style={styles.input} />
      <WNavButton
        label="返却を確定（署名へ）"
        accessibilityLabel="返却を確定し署名画面へ"
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
});
