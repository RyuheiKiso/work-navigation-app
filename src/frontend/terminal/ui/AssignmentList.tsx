// CMP-HA-022 割当一覧コンポーネント
import React from 'react';
import { FlatList, StyleSheet, Text, View } from 'react-native';
import type { WorkAssignment } from '@wnav/shared';
import { AssignmentBanner } from './AssignmentBanner';

export interface AssignmentListProps {
  assignments: WorkAssignment[];
  onSelect: (assignment: WorkAssignment) => void;
  emptyMessage?: string;
}

export function AssignmentList(props: AssignmentListProps): JSX.Element {
  if (props.assignments.length === 0) {
    return (
      <View style={styles.empty} accessibilityLabel="作業指示なし">
        <Text style={styles.emptyText}>{props.emptyMessage ?? '作業指示がありません'}</Text>
      </View>
    );
  }
  return (
    <FlatList
      data={props.assignments}
      keyExtractor={(item) => item.id}
      renderItem={({ item }) => <AssignmentBanner assignment={item} onPress={props.onSelect} />}
      contentContainerStyle={styles.list}
    />
  );
}

const styles = StyleSheet.create({
  list: { paddingVertical: 8 },
  empty: { padding: 24, alignItems: 'center' },
  emptyText: { fontSize: 16, color: '#475569' },
});
