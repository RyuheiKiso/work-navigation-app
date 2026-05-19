// SCR-HA-009 計測値証跡。MeasurementInput で値入力 + LSL/USL チェック
import React, { useState } from 'react';
import { ScrollView, StyleSheet, Text } from 'react-native';
import { useRouter } from 'expo-router';
import { generateId } from '@wnav/shared/domain/id';
import { MeasurementInput } from '../../../ui/MeasurementInput';
import { WNavButton } from '../../../ui/WNavButton';
import { getDataSource } from '../../../db/data-source';
import { LocalMeasurement } from '../../../db/entities/LocalMeasurement';
import { useAuth } from '../../../contexts/AuthContext';
import { useWorkExecution } from '../../../contexts/WorkExecutionContext';

const DEFAULT_UNIT = 'mm';
const DEFAULT_LSL = 9.5;
const DEFAULT_USL = 10.5;

export default function MeasurementScreen(): JSX.Element {
  const router = useRouter();
  const { state: auth } = useAuth();
  const { state: exec } = useWorkExecution();
  const [value, setValue] = useState('');

  const handleSave = async (): Promise<void> => {
    const numeric = Number(value);
    if (Number.isNaN(numeric)) return;
    const entity: LocalMeasurement = {
      id: generateId(),
      workExecutionId: exec.workExecutionId ?? 'unknown',
      stepId: exec.currentStepId ?? 'unknown',
      value: numeric,
      unit: DEFAULT_UNIT,
      usl: DEFAULT_USL,
      lsl: DEFAULT_LSL,
      inSpec: numeric >= DEFAULT_LSL && numeric <= DEFAULT_USL,
      recordedBy: auth.user?.userId ?? 'unknown',
      recordedAt: new Date().toISOString(),
    };
    await getDataSource().getRepository(LocalMeasurement).save(entity);
    router.back();
  };

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.title} accessibilityRole="header">
        計測値の入力
      </Text>
      <MeasurementInput
        value={value}
        onChange={setValue}
        unit={DEFAULT_UNIT}
        usl={DEFAULT_USL}
        lsl={DEFAULT_LSL}
        accessibilityLabel="計測値を入力"
      />
      <WNavButton
        label="保存"
        accessibilityLabel="計測値を保存"
        onPress={() => {
          void handleSave();
        }}
      />
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { padding: 16 },
  title: { fontSize: 22, fontWeight: '700', marginBottom: 12 },
});
