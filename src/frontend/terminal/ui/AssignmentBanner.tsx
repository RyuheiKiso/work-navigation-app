// CMP-HA-021 作業指示割当通知バナー
import React from 'react';
import { StyleSheet, Text, TouchableOpacity, View } from 'react-native';
import { tokens } from '@wnav/shared';
import type { WorkAssignment } from '@wnav/shared';

export interface AssignmentBannerProps {
  assignment: WorkAssignment;
  onPress: (assignment: WorkAssignment) => void;
}

export function AssignmentBanner(props: AssignmentBannerProps): JSX.Element {
  const due = props.assignment.dueAt !== null ? `期限: ${props.assignment.dueAt}` : '期限なし';
  return (
    <TouchableOpacity
      accessibilityRole="button"
      accessibilityLabel={`作業指示 ${props.assignment.sopName} を選択`}
      onPress={() => props.onPress(props.assignment)}
      style={styles.banner}
    >
      <View style={styles.row}>
        <Text style={styles.title}>{props.assignment.sopName}</Text>
        <Text style={styles.priority}>優先度 {props.assignment.priority}</Text>
      </View>
      <Text style={styles.subtitle}>ロット: {props.assignment.lotNumber ?? '-'}</Text>
      <Text style={styles.subtitle}>{due}</Text>
    </TouchableOpacity>
  );
}

const styles = StyleSheet.create({
  banner: {
    minHeight: 96,
    padding: 16,
    backgroundColor: tokens.brand.accent.light[50],
    borderLeftWidth: 6,
    borderLeftColor: tokens.brand.accent.light[500],
    borderRadius: 10,
    marginVertical: 6,
  },
  row: { flexDirection: 'row', justifyContent: 'space-between' },
  title: { fontSize: 18, fontWeight: '700', color: tokens.neutral[900] },
  priority: { fontSize: 14, fontWeight: '600', color: tokens.brand.accent.light[700] },
  subtitle: { fontSize: 14, color: tokens.neutral[700], marginTop: 4 },
});
