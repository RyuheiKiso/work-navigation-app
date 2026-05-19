// SCR-HA-012 作業再開。GET /work-assignments で中断中の作業を取得
import React from 'react';
import { ActivityIndicator, FlatList, StyleSheet, Text, TouchableOpacity, View } from 'react-native';
import { useRouter } from 'expo-router';
import { useQuery } from '@tanstack/react-query';
import Constants from 'expo-constants';
import type { WorkAssignment } from '@wnav/shared';
import { useAuth } from '../../../contexts/AuthContext';

export default function ResumeScreen(): JSX.Element {
  const router = useRouter();
  const { state } = useAuth();
  const apiBaseUrl =
    (Constants.expoConfig?.extra?.['apiBaseUrl'] as string | undefined) ?? 'http://localhost:8080/api/v1';

  const { data, isLoading } = useQuery({
    queryKey: ['resume-list'],
    queryFn: async () => {
      const res = await fetch(`${apiBaseUrl}/work-assignments?status=pending&suspended=true`, {
        headers: { Authorization: `Bearer ${state.token ?? ''}` },
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = (await res.json()) as { data: WorkAssignment[] };
      return body.data;
    },
  });

  if (isLoading) {
    return (
      <View style={styles.center}>
        <ActivityIndicator />
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <Text style={styles.title} accessibilityRole="header">
        再開可能な作業
      </Text>
      <FlatList
        data={data ?? []}
        keyExtractor={(item) => item.id}
        renderItem={({ item }) => (
          <TouchableOpacity
            accessibilityRole="button"
            accessibilityLabel={`${item.sopName} を再開`}
            style={styles.row}
            onPress={() => router.push(`/(main)/step/${encodeURIComponent(item.id)}`)}
          >
            <Text style={styles.rowTitle}>{item.sopName}</Text>
            <Text style={styles.rowSub}>{item.lotNumber ?? '-'}</Text>
          </TouchableOpacity>
        )}
        ListEmptyComponent={<Text style={styles.empty}>再開可能な作業はありません</Text>}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, padding: 16 },
  center: { flex: 1, justifyContent: 'center', alignItems: 'center' },
  title: { fontSize: 22, fontWeight: '700', marginBottom: 12 },
  row: {
    minHeight: 72,
    padding: 16,
    borderWidth: 1,
    borderColor: '#E2E8F0',
    borderRadius: 10,
    marginBottom: 6,
    justifyContent: 'center',
  },
  rowTitle: { fontSize: 16, fontWeight: '700' },
  rowSub: { fontSize: 14, color: '#64748B', marginTop: 2 },
  empty: { textAlign: 'center', padding: 24, color: '#64748B' },
});
