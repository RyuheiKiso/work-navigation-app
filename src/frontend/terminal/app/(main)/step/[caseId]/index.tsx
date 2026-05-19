// SCR-HA-005 標準 Step 画面。StepEngine.completeStep() で進行イベントを生成
import React, { useState } from 'react';
import { Alert, ScrollView, StyleSheet, Text, View } from 'react-native';
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
import { SopRepository } from '../../../../db/repositories/SopRepository';

export default function StandardStepScreen(): JSX.Element {
  const params = useLocalSearchParams<{ caseId: string }>();
  const router = useRouter();
  const { state: auth } = useAuth();
  const { state: exec, dispatch } = useWorkExecution();
  const [busy, setBusy] = useState(false);

  const handleComplete = async (): Promise<void> => {
    setBusy(true);
    try {
      const sopRepo = new SopRepository();
      const engine = new StepEngine(new WorkEventRepository(), new OutboxRepository(), sopRepo);
      const caseId = params.caseId ?? '';
      const sopVersionId = exec.sopVersionId ?? '';
      const stepIndex = exec.currentStepIndex;

      // SOP 定義から現在ステップの ID を取得する（ランダム UUID は不変性を壊すため禁止）
      const allSteps = await sopRepo.findStepsBySopVersionId(sopVersionId);
      const currentStep = allSteps[stepIndex];
      const stepId = currentStep?.id ?? generateId();

      // ゲート検証（BR-BUS-001/002, FR-AU-001, FR-EV-013）を UI 層でも事前チェックしてユーザーに通知する
      const gate = await engine.canAdvanceToStep(caseId, stepIndex, sopVersionId);
      if (!gate.canAdvance) {
        const messages: Record<string, string> = {
          PREVIOUS_STEP_NOT_COMPLETED: '前のステップが完了していません',
          EVIDENCE_REQUIRED: '証拠（写真・QR）を記録してください',
          SIGN_REQUIRED: '電子署名が必要です',
          WRONG_TOOL_SCAN: '使用器具のQRスキャン照合が未完了です',
          SKILL_LEVEL_INSUFFICIENT: 'スキルレベルが不足しています',
          CONDITION_BRANCH_UNRESOLVED: '分岐条件が未解決です',
          OUT_OF_SPEC: '測定値が規格外です',
        };
        Alert.alert('ステップ完了不可', messages[gate.blockedReason ?? ''] ?? '要件を満たしていません');
        return;
      }
      const payload: StandardStepPayload = {
        inputType: 'boolean_check',
        stepId,
        stepNumber: stepIndex + 1,
        value: true,
      };
      await engine.completeStep({
        caseId,
        stepId,
        stepIndex,
        sopVersionId,
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
