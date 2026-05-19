// SCR-HA-007 カスタム入力 Step。CustomStepPayload.fields を動的レンダリング
import React, { useState } from 'react';
import { ScrollView, StyleSheet, Text, TextInput, View } from 'react-native';
import { useRouter } from 'expo-router';
import { GlanceableBanner } from '../../../../ui/GlanceableBanner';
import { WNavButton } from '../../../../ui/WNavButton';

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
  const router = useRouter();
  const [values, setValues] = useState<Record<string, string>>({});

  const handleChange = (key: string, value: string): void => {
    setValues((prev) => ({ ...prev, [key]: value }));
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
        onPress={() => router.back()}
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
