// SCR-HA-007 カスタム入力 Step。CustomStepPayload.fields を動的レンダリングして completeStep を呼ぶ
import React, { useEffect, useState } from 'react';
import { Alert, ScrollView, StyleSheet, Text, TextInput, View } from 'react-native';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { generateId } from '@wnav/shared/domain/id';
import type { CustomStepPayload } from '@wnav/shared/domain/step-engine';
import { GlanceableBanner } from '../../../../ui/GlanceableBanner';
import { WNavButton } from '../../../../ui/WNavButton';
import { useAuth } from '../../../../contexts/AuthContext';
import { useWorkExecution } from '../../../../contexts/WorkExecutionContext';
import { StepEngine } from '../../../../domain/step-engine/StepEngine';
import { WorkEventRepository } from '../../../../db/repositories/WorkEventRepository';
import { OutboxRepository } from '../../../../db/repositories/OutboxRepository';
import { SopRepository } from '../../../../db/repositories/SopRepository';

interface CustomField {
  key: string;
  label: string;
  type: 'text' | 'numeric';
}

const FIELDS: CustomField[] = [
  { key: 'note', label: 'メモ', type: 'text' },
  { key: 'count', label: '数量', type: 'numeric' },
];

export default function CustomStepScreen(): JSX.Element {
  const params = useLocalSearchParams<{ caseId: string }>();
  const router = useRouter();
  const { state: auth } = useAuth();
  const { state: exec, dispatch } = useWorkExecution();
  const [values, setValues] = useState<Record<string, string>>({});
  const [busy, setBusy] = useState(false);
  const [stepIdLoading, setStepIdLoading] = useState(true);
  const [resolvedStepId, setResolvedStepId] = useState<string | null>(null);

  useEffect(() => {
    const sopVersionId = exec.sopVersionId ?? '';
    const stepIndex = exec.currentStepIndex;
    setResolvedStepId(null);
    setStepIdLoading(true);
    void (async () => {
      try {
        const allSteps = await new SopRepository().findStepsBySopVersionId(sopVersionId);
        const step = allSteps[stepIndex];
        if (step != null) {
          setResolvedStepId(step.id);
          dispatch({ type: 'SET_CURRENT_STEP', payload: { index: stepIndex, stepId: step.id } });
        }
      } finally {
        setStepIdLoading(false);
      }
    })();
  }, [exec.currentStepIndex, exec.sopVersionId, dispatch]);

  const handleChange = (key: string, value: string): void => {
    setValues((prev) => ({ ...prev, [key]: value }));
  };

  const handleComplete = async (): Promise<void> => {
    setBusy(true);
    try {
      const sopRepo = new SopRepository();
      const engine = new StepEngine(new WorkEventRepository(), new OutboxRepository(), sopRepo);
      const caseId = params.caseId ?? '';
      const sopVersionId = exec.sopVersionId ?? '';
      const stepIndex = exec.currentStepIndex;
      const stepId = resolvedStepId ?? generateId();

      const gate = await engine.canAdvanceToStep(caseId, stepIndex, sopVersionId);
      if (!gate.canAdvance) {
        const messages: Record<string, string> = {
          PREVIOUS_STEP_NOT_COMPLETED: '前のステップが完了していません',
          EVIDENCE_REQUIRED: '証拠（写真・QR）を記録してください',
          SIGN_REQUIRED: '電子署名が必要です',
          WRONG_TOOL_SCAN: '使用器具のQRスキャン照合が未完了です',
        };
        Alert.alert('ステップ完了不可', messages[gate.blockedReason ?? ''] ?? '要件を満たしていません');
        return;
      }

      const payload: CustomStepPayload = {
        inputType: 'custom',
        stepId,
        stepNumber: stepIndex + 1,
        value: values,
        rendererKey: 'generic_form',
      };
      await engine.completeStep({
        caseId,
        stepId,
        stepIndex,
        sopVersionId,
        workerId: auth.user?.userId ?? 'unknown',
        terminalId: 'terminal-1',
        payload,
        inputData: values,
        activity: 'step_completed',
      });
      dispatch({ type: 'ADVANCE_STEP' });
      router.replace('/(main)/home');
    } finally {
      setBusy(false);
    }
  };

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <GlanceableBanner currentStepLabel="カスタム Step" />
      <Text style={styles.instruction}>必要事項を入力して完了してください</Text>
      {FIELDS.map((field) => (
        <View key={field.key} style={styles.field}>
          <Text style={styles.label}>{field.label}</Text>
          <TextInput
            accessibilityLabel={field.label}
            value={values[field.key] ?? ''}
            onChangeText={(v) => handleChange(field.key, v)}
            keyboardType={field.type === 'numeric' ? 'decimal-pad' : 'default'}
            style={styles.input}
          />
        </View>
      ))}
      <WNavButton
        label="完了"
        accessibilityLabel="カスタム Step を完了"
        onPress={() => { void handleComplete(); }}
        loading={busy || stepIdLoading}
        disabled={stepIdLoading}
      />
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { padding: 16 },
  instruction: { fontSize: 16, marginBottom: 12 },
  field: { marginVertical: 6 },
  label: { fontSize: 14, marginBottom: 4 },
  input: {
    minHeight: 72,
    borderWidth: 2,
    borderColor: '#CBD5E1',
    borderRadius: 12,
    paddingHorizontal: 16,
    fontSize: 20,
  },
});
