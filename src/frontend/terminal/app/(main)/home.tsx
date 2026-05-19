// SCR-HA-002 ホーム。作業指示一覧 + AssignmentBanner / List
import React from 'react';
import { ScrollView, StyleSheet, Text, View } from 'react-native';
import { useRouter } from 'expo-router';
import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import Constants from 'expo-constants';
import type { WorkAssignment } from '@wnav/shared';
import { WNavButton } from '../../ui/WNavButton';
import { AssignmentList } from '../../ui/AssignmentList';
import { AssignmentBanner } from '../../ui/AssignmentBanner';
import { useAuth } from '../../contexts/AuthContext';

export default function HomeScreen(): JSX.Element {
  const router = useRouter();
  const { t } = useTranslation();
  const { state } = useAuth();

  const apiBaseUrl =
    (Constants.expoConfig?.extra?.['apiBaseUrl'] as string | undefined) ?? 'http://localhost:8080/api/v1';

  const { data, isLoading, refetch } = useQuery({
    queryKey: ['work-assignments', state.user?.userId],
    queryFn: async () => {
      const res = await fetch(`${apiBaseUrl}/work-assignments?status=pending`, {
        headers: { Authorization: `Bearer ${state.token ?? ''}` },
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = (await res.json()) as { data: WorkAssignment[] };
      return body.data;
    },
    enabled: state.isAuthenticated,
  });

  const handleSelect = (assignment: WorkAssignment): void => {
    router.push(`/(main)/sop/${assignment.sopId}`);
  };

  const assignments = data ?? [];
  // CMP-HA-021: priority=1 かつ pending/dispatched の最優先割当をバナーで固定表示する（仕様: SCR-HA-002）
  const bannerAssignment = assignments.find(
    (a) => a.priority === 1 && (a.status === 'pending' || a.status === 'dispatched'),
  ) ?? null;
  // CMP-HA-022: バナー表示中の割当を一覧から除外して重複を防ぐ
  const listAssignments = bannerAssignment
    ? assignments.filter((a) => a.id !== bannerAssignment.id)
    : assignments;

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.title} accessibilityRole="header">
        {t('home.title')}
      </Text>
      {bannerAssignment !== null && (
        <AssignmentBanner assignment={bannerAssignment} onPress={handleSelect} />
      )}
      <View style={styles.actions}>
        <WNavButton
          label="QR スキャン"
          accessibilityLabel="QR コードスキャン画面を開く"
          onPress={() => router.push('/(main)/qr-scan')}
        />
        <WNavButton
          label="再開"
          accessibilityLabel="中断中の作業を再開"
          variant="secondary"
          onPress={() => router.push('/(main)/resume')}
        />
      </View>
      <AssignmentList
        assignments={listAssignments}
        onSelect={handleSelect}
        emptyMessage={t('home.emptyAssignments')}
      />
      <WNavButton
        label="更新"
        accessibilityLabel="作業指示一覧を更新"
        variant="secondary"
        onPress={() => {
          void refetch();
        }}
        loading={isLoading}
      />
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { padding: 16 },
  title: { fontSize: 24, fontWeight: '700', marginBottom: 16 },
  actions: { flexDirection: 'row', gap: 12, marginBottom: 16 },
});
