// SCR-HA-019 修正作業実行。新 case_id で開始、前後写真必須
import React, { useState } from 'react';
import { ScrollView, StyleSheet, Text, TextInput, View } from 'react-native';
import { useRouter } from 'expo-router';
import { WNavButton } from '../../../ui/WNavButton';
import { ReworkFlow } from '../../../domain/rework/ReworkFlow';
import { useAuth } from '../../../contexts/AuthContext';

export default function ReworkExecuteScreen(): JSX.Element {
  const router = useRouter();
  const { state: auth } = useAuth();
  const [parentCaseId, setParentCaseId] = useState('');
  const [nonconformityId, setNonconformityId] = useState('');
  const [category, setCategory] = useState('process_deviation');

  const handleStart = async (): Promise<void> => {
    const flow = new ReworkFlow();
    await flow.startRework({
      parentCaseId,
      nonconformityId,
      ncCategory: category,
      assignedTo: auth.user?.userId ?? null,
      deadline: null,
    });
    router.push('/(main)/rework/re-inspection');
  };

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.title} accessibilityRole="header">
        修正作業の開始
      </Text>
      <Text style={styles.label}>親 case_id</Text>
      <TextInput accessibilityLabel="親 case_id" value={parentCaseId} onChangeText={setParentCaseId} style={styles.input} />
      <Text style={styles.label}>不適合 ID</Text>
      <TextInput accessibilityLabel="不適合 ID" value={nonconformityId} onChangeText={setNonconformityId} style={styles.input} />
      <Text style={styles.label}>不適合カテゴリ</Text>
      <TextInput accessibilityLabel="不適合カテゴリ" value={category} onChangeText={setCategory} style={styles.input} />
      <View style={styles.actions}>
        <WNavButton
          label="前写真撮影"
          accessibilityLabel="修正前写真を撮影"
          variant="secondary"
          onPress={() => router.push('/(main)/evidence/photo')}
        />
        <WNavButton
          label="後写真撮影"
          accessibilityLabel="修正後写真を撮影"
          variant="secondary"
          onPress={() => router.push('/(main)/evidence/photo')}
        />
      </View>
      <WNavButton
        label="修正作業開始"
        accessibilityLabel="修正作業を開始"
        onPress={() => {
          void handleStart();
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
  actions: { flexDirection: 'row', gap: 12, marginVertical: 12 },
});
