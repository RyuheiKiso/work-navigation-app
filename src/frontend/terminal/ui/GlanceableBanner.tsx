// Glanceable バナー。現在 Step・異常アラート・次 Step を 200ms 以内に視認可能にする
import React from 'react';
import { StyleSheet, Text, View } from 'react-native';
import { tokens } from '@wnav/shared';

export interface GlanceableBannerProps {
  currentStepLabel: string;
  nextStepLabel?: string;
  alertLabel?: string | null;
  stepNumber?: number;
  totalSteps?: number;
}

export function GlanceableBanner(props: GlanceableBannerProps): JSX.Element {
  return (
    <View accessibilityRole="header" style={styles.container}>
      <View style={styles.row}>
        <Text style={styles.title} accessibilityLabel="現在のステップ">
          現在: {props.currentStepLabel}
        </Text>
        {props.stepNumber !== undefined && props.totalSteps !== undefined ? (
          <Text style={styles.progress}>
            {props.stepNumber}/{props.totalSteps}
          </Text>
        ) : null}
      </View>
      {props.alertLabel ? (
        <Text style={styles.alert} accessibilityRole="alert" accessibilityLabel="異常アラート">
          {props.alertLabel}
        </Text>
      ) : null}
      {props.nextStepLabel ? (
        <Text style={styles.next} accessibilityLabel="次のステップ">
          次へ: {props.nextStepLabel}
        </Text>
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: tokens.brand.primary.light[500],
    padding: 16,
    borderRadius: 12,
    marginBottom: 12,
  },
  row: { flexDirection: 'row', justifyContent: 'space-between', alignItems: 'center' },
  title: { color: '#FFFFFF', fontSize: 20, fontWeight: '700' },
  progress: { color: '#FFFFFF', fontSize: 16, fontWeight: '600' },
  alert: {
    color: '#FFFFFF',
    fontSize: 18,
    fontWeight: '700',
    backgroundColor: tokens.state.andon.light.base,
    padding: 8,
    marginTop: 8,
    borderRadius: 6,
  },
  next: { color: '#FFFFFF', fontSize: 16, marginTop: 8 },
});
