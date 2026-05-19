// SCR-HA-003 QR スキャン。スキャン結果で作業特定 → WorkExecution を解決して step 画面へ
import React, { useState } from 'react';
import { Alert } from 'react-native';
import { useRouter } from 'expo-router';
import { QrScannerOverlay } from '../../ui/QrScannerOverlay';
import { useWorkExecution } from '../../contexts/WorkExecutionContext';
import { WorkExecutionRepository } from '../../db/repositories/WorkExecutionRepository';

export default function QrScanScreen(): JSX.Element {
  const router = useRouter();
  const { dispatch } = useWorkExecution();
  const [processing, setProcessing] = useState(false);

  const handleScanned = async (value: string): Promise<void> => {
    if (processing) return;
    setProcessing(true);
    try {
      if (!value.startsWith('case:')) {
        router.replace('/(main)/home');
        return;
      }
      const caseId = value.slice('case:'.length);

      // ローカル DB から作業実行を取得して WorkExecutionContext を初期化する
      const execRepo = new WorkExecutionRepository();
      const execution = await execRepo.findById(caseId);
      if (execution == null) {
        Alert.alert('スキャンエラー', 'この QR コードに対応する作業が見つかりません');
        return;
      }

      // sopVersionSnapshot から totalStepCount と sopVersionId を取得する
      const snapshot = JSON.parse(execution.sopVersionSnapshot) as { sopId: string; version: string; snapshotHash: string };
      dispatch({
        type: 'START_EXECUTION',
        payload: {
          caseId,
          workExecutionId: execution.id,
          sopVersionId: snapshot.sopId,
          totalSteps: execution.totalStepCount,
        },
      });
      router.replace(`/(main)/step/${encodeURIComponent(caseId)}`);
    } catch (err) {
      Alert.alert('エラー', err instanceof Error ? err.message : 'QR スキャン処理に失敗しました');
    } finally {
      setProcessing(false);
    }
  };

  return (
    <QrScannerOverlay
      onScanned={(v) => void handleScanned(v)}
      onCancel={() => router.back()}
    />
  );
}
