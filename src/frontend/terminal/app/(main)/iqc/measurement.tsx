// SCR-HA-017 IQC 測定値入力。MeasurementInput + 不良判定チップ表示
import React, { useState } from 'react';
import { ScrollView, StyleSheet, Switch, Text, View } from 'react-native';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { useAuth } from '../../../contexts/AuthContext';
import { MeasurementInput } from '../../../ui/MeasurementInput';
import { WNavButton } from '../../../ui/WNavButton';
import { IqcInspectionFlow } from '../../../domain/iqc/IqcInspectionFlow';

export default function IqcMeasurementScreen(): JSX.Element {
  const router = useRouter();
  const params = useLocalSearchParams<{ id?: string }>();
  const { state: auth } = useAuth();
  const [sampleNo, setSampleNo] = useState(1);
  const [value, setValue] = useState('');
  const [defect, setDefect] = useState(false);

  const handleSave = async (): Promise<void> => {
    if (params.id === undefined) return;
    const flow = new IqcInspectionFlow();
    await flow.recordMeasurement({
      inspectionId: params.id,
      sampleNo,
      measuredValue: value === '' ? null : Number(value),
      defectFlag: defect,
      evidenceFileId: null,
      recordedBy: auth.user?.userId ?? 'unknown',
    });
    setSampleNo((prev) => prev + 1);
    setValue('');
    setDefect(false);
  };

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.title} accessibilityRole="header">
        サンプル {sampleNo} の測定
      </Text>
      <MeasurementInput
        value={value}
        onChange={setValue}
        unit="mm"
        usl={10.5}
        lsl={9.5}
        accessibilityLabel="測定値を入力"
      />
      <View style={styles.row}>
        <Text style={styles.label}>不良フラグ</Text>
        <Switch
          accessibilityLabel="不良フラグ"
          value={defect}
          onValueChange={setDefect}
        />
      </View>
      <WNavButton
        label="サンプル登録"
        accessibilityLabel="このサンプルを登録"
        onPress={() => {
          void handleSave();
        }}
      />
      <WNavButton
        label="判定へ"
        accessibilityLabel="判定画面へ進む"
        variant="secondary"
        onPress={() => router.push(`/(main)/iqc/verdict?id=${encodeURIComponent(params.id ?? '')}`)}
      />
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { padding: 16 },
  title: { fontSize: 22, fontWeight: '700', marginBottom: 12 },
  row: { flexDirection: 'row', alignItems: 'center', gap: 12, marginVertical: 8 },
  label: { fontSize: 16 },
});
