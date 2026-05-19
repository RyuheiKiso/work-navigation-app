// 計測値入力。USL/LSL 逸脱時は警告色で視覚的に通知する
import React, { useMemo } from 'react';
import { StyleSheet, Text, TextInput, View } from 'react-native';
import { tokens } from '@wnav/shared';

export interface MeasurementInputProps {
  value: string;
  onChange: (value: string) => void;
  unit: string;
  usl?: number | null;
  lsl?: number | null;
  accessibilityLabel: string;
}

export function MeasurementInput(props: MeasurementInputProps): JSX.Element {
  const numeric = Number(props.value);
  const outOfSpec = useMemo(() => {
    if (Number.isNaN(numeric)) return false;
    if (props.lsl !== null && props.lsl !== undefined && numeric < props.lsl) return true;
    if (props.usl !== null && props.usl !== undefined && numeric > props.usl) return true;
    return false;
  }, [numeric, props.lsl, props.usl]);

  return (
    <View style={styles.container}>
      <View style={styles.row}>
        <TextInput
          accessibilityLabel={props.accessibilityLabel}
          keyboardType="decimal-pad"
          value={props.value}
          onChangeText={props.onChange}
          style={[styles.input, outOfSpec ? styles.outOfSpecBorder : null]}
        />
        <Text style={styles.unit}>{props.unit}</Text>
      </View>
      {outOfSpec ? (
        <Text style={styles.warning} accessibilityRole="alert">
          範囲外: LSL {props.lsl ?? '-'} / USL {props.usl ?? '-'}
        </Text>
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  container: { marginVertical: 8 },
  row: { flexDirection: 'row', alignItems: 'center' },
  input: {
    flex: 1,
    minHeight: 72,
    borderWidth: 2,
    borderColor: tokens.neutral[300],
    borderRadius: 12,
    paddingHorizontal: 16,
    fontSize: 24,
  },
  outOfSpecBorder: { borderColor: tokens.state.danger.light[500] },
  unit: { marginLeft: 12, fontSize: 18, color: tokens.neutral[700] },
  warning: {
    marginTop: 6,
    color: tokens.state.danger.light[700],
    fontSize: 14,
    fontWeight: '600',
  },
});
