// SCR-HA-005 標準 Step 画面。StepEngine.completeStep() で進行イベントを生成
import React, { useState } from 'react';
import { ScrollView, StyleSheet, Text, View } from 'react-native';
import { useLocalSearchParams, useRouter } from 'expo-router';
import type { StandardStepPayload } from '@wnav/shared/domain/step-engine';
import { generateId } from '@wnav/shared/domain/id';
import { GlanceableBanner } from '../../../../ui/GlanceableBanner';
import { WNavButton } from '../../../../ui/WNavButton';
import { useAuth } from '../../../../contexts/AuthContext';
import { useWorkExecution } from '../../../../contexts/WorkExecutionContext';
import { StepEngine } from '../../../../domain/step-engine/StepEngine';
import { WorkEventRepository } from '../../../../db/repositories/WorkEventRepository';
import { OutboxRepository } from '../../../../db/repositories/OutboxRepository';

export default function StandardStepScreen(): JSX.Element {
  const params = useLocalSearchParams<{ caseId: string }>();
  const router = useRouter();
  const { state: auth } = useAuth();
  const { state: exec, dispatch } = useWorkExecution();
  const [busy, setBusy] = useState(false);

  const handleComplete = async (): Promise<void> => {
    setBusy(true);
    try {
      const engine = new StepEngine(new WorkEventRepository(), new OutboxRepository());
      const stepId = exec.workExecutionId ?? generateId();
      const payload: StandardStepPayload = {
        inputType: 'boolean_check',
        stepId,
        stepNumber: exec.currentStepIndex + 1,
        value: true,
      };
      await engine.completeStep({
        caseId: params.caseId ?? '',
        stepId,
        sopVersionId: exec.sopVersionId ?? '',
        workerId: auth.user?.userId ?? 'unknown',
        terminalId: 'terminal-1',
        payload,
        inputData: { confirmed: true },
      });
      dispatch({ type: 'ADVANCE_STEP' });
      router.replace('/(main)/home');
    } finally {
      setBusy(false);
    }
  };

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <GlanceableBanner
        currentStepLabel={`Step ${exec.currentStepIndex + 1}`}
        nextStepLabel={`Step ${exec.currentStepIndex + 2}`}
        stepNumber={exec.currentStepIndex + 1}
        totalSteps={exec.totalSteps}
      />
      <Text style={styles.instruction} accessibilityRole="text">
        作業内容を確認し、完了したら下のボタンを押してください
      </Text>
      <View style={styles.actions}>
        <WNavButton
          label="証跡撮影"
          accessibilityLabel="証跡写真を撮影"
          variant="secondary"
          onPress={() => router.push('/(main)/evidence/photo')}
        />
        <WNavButton
          label="計測値入力"
          accessibilityLabel="計測値を入力"
          variant="secondary"
          onPress={() => router.push('/(main)/evidence/measurement')}
        />
      </View>
      <WNavButton
        label="ステップ完了"
        accessibilityLabel="このステップを完了"
        onPress={() => {
          void handleComplete();
        }}
        loading={busy}
      />
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { padding: 16 },
  instruction: { fontSize: 16, marginVertical: 16 },
  actions: { flexDirection: 'row', gap: 12, marginBottom: 16 },
});
