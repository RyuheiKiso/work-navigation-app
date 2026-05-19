// SCR-HA-016 IQC 受入。ロット QR / supplier / material / 受入数量 → IqcInspectionFlow.startInspection
import React, { useState } from 'react';
import { ScrollView, StyleSheet, Text, TextInput, View } from 'react-native';
import { useRouter } from 'expo-router';
import { WNavButton } from '../../../ui/WNavButton';
import { QrScannerOverlay } from '../../../ui/QrScannerOverlay';
import { IqcInspectionFlow } from '../../../domain/iqc/IqcInspectionFlow';

export default function IqcReceiveScreen(): JSX.Element {
  const router = useRouter();
  const [lotId, setLotId] = useState('');
  const [supplierId, setSupplierId] = useState('');
  const [materialId, setMaterialId] = useState('');
  const [qty, setQty] = useState('');
  const [scanning, setScanning] = useState(false);

  const handleScanned = (value: string): void => {
    setLotId(value);
    setScanning(false);
  };

  const handleStart = async (): Promise<void> => {
    const flow = new IqcInspectionFlow();
    const inspection = await flow.startInspection({
      lotId,
      supplierId,
      materialId,
      receivedQty: Number(qty),
      samplingPlanId: 'default-plan',
      inspectionLevel: 'II',
      severityState: 'NORMAL',
      aqlValue: 1.0,
    });
    router.push(`/(main)/iqc/measurement?id=${encodeURIComponent(inspection.id)}`);
  };

  if (scanning) {
    return <QrScannerOverlay onScanned={handleScanned} onCancel={() => setScanning(false)} />;
  }

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.title} accessibilityRole="header">
        受入検査開始
      </Text>
      <View style={styles.field}>
        <Text style={styles.label}>ロット QR</Text>
        <TextInput accessibilityLabel="ロット ID" value={lotId} onChangeText={setLotId} style={styles.input} />
        <WNavButton
          label="スキャン"
          accessibilityLabel="ロット QR をスキャン"
          variant="secondary"
          onPress={() => setScanning(true)}
        />
      </View>
      <View style={styles.field}>
        <Text style={styles.label}>仕入先 ID</Text>
        <TextInput accessibilityLabel="仕入先 ID" value={supplierId} onChangeText={setSupplierId} style={styles.input} />
      </View>
      <View style={styles.field}>
        <Text style={styles.label}>材料 ID</Text>
        <TextInput accessibilityLabel="材料 ID" value={materialId} onChangeText={setMaterialId} style={styles.input} />
      </View>
      <View style={styles.field}>
        <Text style={styles.label}>受入数量</Text>
        <TextInput
          accessibilityLabel="受入数量"
          keyboardType="decimal-pad"
          value={qty}
          onChangeText={setQty}
          style={styles.input}
        />
      </View>
      <WNavButton
        label="検査開始"
        accessibilityLabel="検査を開始"
        onPress={() => {
          void handleStart();
        }}
      />
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { padding: 16 },
  title: { fontSize: 22, fontWeight: '700', marginBottom: 12 },
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
