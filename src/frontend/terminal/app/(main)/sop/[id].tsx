// SCR-HA-004 SOP 詳細。GET /master/sops/{id} と steps を取得して一覧表示
import React from 'react';
import { ActivityIndicator, FlatList, ScrollView, StyleSheet, Text, View } from 'react-native';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { useQuery } from '@tanstack/react-query';
import Constants from 'expo-constants';
import type { Sop, Step } from '@wnav/shared';
import { useAuth } from '../../../contexts/AuthContext';
import { WNavButton } from '../../../ui/WNavButton';

interface SopWithSteps {
  sop: Sop;
  steps: Step[];
}

export default function SopDetailScreen(): JSX.Element {
  const params = useLocalSearchParams<{ id: string }>();
  const router = useRouter();
  const { state } = useAuth();

  const apiBaseUrl =
    (Constants.expoConfig?.extra?.['apiBaseUrl'] as string | undefined) ?? 'http://localhost:8080/api/v1';

  const { data, isLoading } = useQuery<SopWithSteps>({
    queryKey: ['sop', params.id],
    queryFn: async () => {
      const res = await fetch(`${apiBaseUrl}/master/sops/${encodeURIComponent(params.id ?? '')}`, {
        headers: { Authorization: `Bearer ${state.token ?? ''}` },
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = (await res.json()) as { data: SopWithSteps };
      return body.data;
    },
  });

  if (isLoading || data === undefined) {
    return (
      <View style={styles.center}>
        <ActivityIndicator />
      </View>
    );
  }

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.title} accessibilityRole="header">
        {data.sop.nameJson.ja}
      </Text>
      <Text style={styles.code}>{data.sop.sopCode}</Text>
      <FlatList
        data={data.steps}
        keyExtractor={(item) => item.id}
        scrollEnabled={false}
        renderItem={({ item }) => (
          <View style={styles.stepRow}>
            <Text style={styles.stepNumber}>#{item.stepNumber}</Text>
            <Text style={styles.stepTitle}>{item.titleJson.ja}</Text>
          </View>
        )}
      />
      <WNavButton
        label="作業開始"
        accessibilityLabel="この SOP で作業を開始"
        onPress={() => router.push(`/(main)/step/${encodeURIComponent(params.id ?? '')}`)}
      />
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { padding: 16 },
  center: { flex: 1, justifyContent: 'center', alignItems: 'center' },
  title: { fontSize: 24, fontWeight: '700', marginBottom: 4 },
  code: { fontSize: 14, color: '#475569', marginBottom: 16 },
  stepRow: {
    flexDirection: 'row',
    padding: 12,
    borderWidth: 1,
    borderColor: '#E2E8F0',
    borderRadius: 8,
    marginBottom: 6,
    alignItems: 'center',
  },
  stepNumber: { fontSize: 16, fontWeight: '700', marginRight: 12, width: 40 },
  stepTitle: { fontSize: 16, flex: 1 },
});
