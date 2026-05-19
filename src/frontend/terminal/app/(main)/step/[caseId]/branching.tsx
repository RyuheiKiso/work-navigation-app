// SCR-HA-006 条件分岐 Step。StepEngine.resolveBranch() で次 Step を決定
import React, { useState } from 'react';
import { ScrollView, StyleSheet, Text, TextInput, View } from 'react-native';
import { useLocalSearchParams, useRouter } from 'expo-router';
import type { BranchingStepPayload } from '@wnav/shared/domain/step-engine';
import { GlanceableBanner } from '../../../../ui/GlanceableBanner';
import { WNavButton } from '../../../../ui/WNavButton';
import { StepEngine } from '../../../../domain/step-engine/StepEngine';
import { WorkEventRepository } from '../../../../db/repositories/WorkEventRepository';
import { OutboxRepository } from '../../../../db/repositories/OutboxRepository';

export default function BranchingStepScreen(): JSX.Element {
  const params = useLocalSearchParams<{ caseId: string }>();
  const router = useRouter();
  const [measuredValue, setMeasuredValue] = useState('');
  const [result, setResult] = useState<string | null>(null);

  const handleEvaluate = (): void => {
    const engine = new StepEngine(new WorkEventRepository(), new OutboxRepository());
    const numeric = Number(measuredValue);
    const payload: BranchingStepPayload = {
      inputType: 'condition_branch',
      stepId: 'branch-step',
      stepNumber: 1,
      branchResult: !Number.isNaN(numeric),
      judgmentCondition: {
        rule: { '>': [{ var: 'measuredValue' }, 10] },
        passStepId: 'pass-step',
        failStepId: 'fail-step',
      },
    };
    const r = engine.resolveBranch(payload, { measuredValue: numeric });
    setResult(`次のステップ: ${r.nextStepId ?? '完了'} (passed=${r.passed})`);
  };

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <GlanceableBanner currentStepLabel="条件分岐 Step" />
      <Text style={styles.instruction}>計測値を入力すると JSON Logic 条件を評価します</Text>
      <View style={styles.field}>
        <Text style={styles.label}>計測値</Text>
        <TextInput
          accessibilityLabel="計測値入力"
          keyboardType="decimal-pad"
          value={measuredValue}
          onChangeText={setMeasuredValue}
          style={styles.input}
        />
      </View>
      <WNavButton label="評価" accessibilityLabel="条件評価" onPress={handleEvaluate} />
      {result !== null ? <Text style={styles.result}>{result}</Text> : null}
      <WNavButton
        label="完了"
        accessibilityLabel="分岐 Step を完了"
        variant="secondary"
        onPress={() => router.replace(`/(main)/step/${encodeURIComponent(params.caseId ?? '')}`)}
      />
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { padding: 16, gap: 12 },
  instruction: { fontSize: 16 },
  field: { marginVertical: 8 },
  label: { fontSize: 14, marginBottom: 4 },
  input: {
    minHeight: 72,
    borderWidth: 2,
    borderColor: '#CBD5E1',
    borderRadius: 12,
    paddingHorizontal: 16,
    fontSize: 20,
  },
  result: { fontSize: 16, color: '#1E3A5F', fontWeight: '600' },
});
