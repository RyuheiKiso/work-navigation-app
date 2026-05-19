import type React from 'react';
import { Alert, AlertTitle, List, ListItem, ListItemText, Box } from '@mui/material';

interface ImpactedItem {
  type: 'work_order' | 'work_execution' | 'sop_version';
  id: string;
  label: string;
  description?: string;
}

// 変更対象マスタを参照中の作業指示・進行中作業を事前表示（src/frontend/master/CLAUDE.md §マスタ編集 UX）。
export function ImpactRangePreview({
  items,
  emptyMessage = '影響範囲はありません',
}: {
  items: ImpactedItem[];
  emptyMessage?: string;
}): React.ReactElement {
  if (items.length === 0) {
    return (
      <Alert severity="success" sx={{ mb: 2 }} role="status">
        <AlertTitle>影響範囲</AlertTitle>
        {emptyMessage}
      </Alert>
    );
  }
  return (
    <Box mb={2}>
      <Alert severity="warning" role="alert">
        <AlertTitle>影響範囲 ({items.length} 件)</AlertTitle>
        この変更は以下を参照中です。承認前に確認してください。
      </Alert>
      <List dense aria-label="影響対象リスト">
        {items.map((item) => (
          <ListItem key={`${item.type}-${item.id}`} divider>
            <ListItemText
              primary={`[${labelFor(item.type)}] ${item.label}`}
              secondary={item.description}
            />
          </ListItem>
        ))}
      </List>
    </Box>
  );
}

function labelFor(type: ImpactedItem['type']): string {
  switch (type) {
    case 'work_order':
      return '作業指示';
    case 'work_execution':
      return '進行中作業';
    case 'sop_version':
      return '参照 SOP 版';
  }
}
