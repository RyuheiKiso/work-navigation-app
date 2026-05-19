// SCR-HA-018 IQC 合否判定。AQL 判定実行 + ディスポジションへの遷移
import React, { useState } from 'react';
import { ScrollView, StyleSheet, Text, View } from 'react-native';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { useAuth } from '../../../contexts/AuthContext';
import { WNavButton } from '../../../ui/WNavButton';
import { IqcInspectionFlow } from '../../../domain/iqc/IqcInspectionFlow';
import type { AqlVerdict } from '@wnav/shared/domain/aql';

export default function IqcVerdictScreen(): JSX.Element {
  const router = useRouter();
  const params = useLocalSearchParams<{ id?: string }>();
  const { state: auth } = useAuth();
  const [verdict, setVerdict] = useState<AqlVerdict | null>(null);

  const handleJudge = async (): Promise<void> => {
    if (params.id === undefined) return;
    const flow = new IqcInspectionFlow();
    const v = await flow.judge(params.id, auth.user?.userId ?? 'unknown');
    setVerdict(v);
  };

  const banner =
    verdict === 'PASSED' ? '合格' : verdict === 'REJECTED' ? '不合格' : verdict === 'INSPECTING' ? '判定中' : '未判定';
  const bannerStyle =
    verdict === 'PASSED' ? styles.bannerPass : verdict === 'REJECTED' ? styles.bannerFail : styles.bannerNeutral;

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.title} accessibilityRole="header">
        AQL 判定
      </Text>
      <View style={[styles.banner, bannerStyle]} accessibilityLabel={`判定結果: ${banner}`}>
        <Text style={styles.bannerText}>{banner}</Text>
      </View>
      <WNavButton
        label="判定実行"
        accessibilityLabel="AQL 判定を実行"
        onPress={() => {
          void handleJudge();
        }}
      />
      <WNavButton
        label="ディスポジションへ"
        accessibilityLabel="ディスポジション画面へ"
        variant="secondary"
        onPress={() => router.push('/(main)/rework/execute')}
      />
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { padding: 16 },
  title: { fontSize: 22, fontWeight: '700', marginBottom: 12 },
  banner: {
    padding: 24,
    borderRadius: 12,
    alignItems: 'center',
    marginBottom: 16,
  },
  bannerPass: { backgroundColor: '#059669' },
  bannerFail: { backgroundColor: '#DC2626' },
  bannerNeutral: { backgroundColor: '#64748B' },
  bannerText: { color: '#FFFFFF', fontSize: 28, fontWeight: '700' },
});
