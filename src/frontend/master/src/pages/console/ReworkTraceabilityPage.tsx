import type React from 'react';
import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Box, Stack, TextField, Button, Paper, Typography, Alert } from '@mui/material';
import { ReactFlow, Background, Controls, type Node, type Edge, MarkerType } from 'reactflow';
import type { Rework } from '@wnav/shared/types';
import { api } from '@/api/client';
import { queryKeys } from '@/api/queryKeys';
import { PageHeader } from '@/components/PageHeader';

// SCR-MC-015 リワーク追跡（quality_admin/system_admin、FR-RW-016）。
// parent_case_id ↔ rework_case_id の双方向グラフを reactflow で可視化。
export function ReworkTraceabilityPage(): React.ReactElement {
  const [caseId, setCaseId] = useState('');
  const [searchedId, setSearchedId] = useState<string | null>(null);

  const traceQuery = useQuery({
    queryKey: queryKeys.console.reworkTrace(searchedId ?? ''),
    queryFn: async (): Promise<Rework[]> => {
      const r = await api.getList<Rework>(`/reworks?related_to=${encodeURIComponent(searchedId ?? '')}`);
      return r.data;
    },
    enabled: !!searchedId,
  });

  const reworks = traceQuery.data ?? [];
  const { nodes, edges } = buildGraph(searchedId, reworks);

  return (
    <Box>
      <PageHeader title="リワーク追跡" subtitle="parent_case ↔ rework_case の双方向グラフ表示" />
      <Stack direction="row" spacing={2} mb={2}>
        <TextField
          label="ケース ID（parent または rework）"
          value={caseId}
          onChange={(e) => setCaseId(e.target.value)}
          sx={{ minWidth: 400 }}
          inputProps={{ 'aria-label': 'ケース ID' }}
        />
        <Button
          variant="contained"
          onClick={() => setSearchedId(caseId || null)}
          disabled={!caseId}
          aria-label="追跡を実行"
        >
          追跡
        </Button>
      </Stack>
      {searchedId && reworks.length === 0 && !traceQuery.isLoading && (
        <Alert severity="info">関連するリワークケースが見つかりません</Alert>
      )}
      {nodes.length > 0 && (
        <Paper sx={{ p: 2, height: 600 }} elevation={1}>
          <Typography variant="h3" gutterBottom>
            関連ケースグラフ ({reworks.length} ノード)
          </Typography>
          <Box sx={{ height: 520 }} role="img" aria-label="リワーク追跡グラフ">
            <ReactFlow nodes={nodes} edges={edges} fitView>
              <Background />
              <Controls />
            </ReactFlow>
          </Box>
        </Paper>
      )}
    </Box>
  );
}

function buildGraph(seedId: string | null, reworks: Rework[]): { nodes: Node[]; edges: Edge[] } {
  if (!seedId) return { nodes: [], edges: [] };
  const ids = new Set<string>([seedId]);
  for (const r of reworks) {
    ids.add(r.parentCaseId);
    ids.add(r.reworkCaseId);
  }
  const sortedIds = [...ids];
  const nodes: Node[] = sortedIds.map((id, idx) => ({
    id,
    data: { label: id === seedId ? `★ ${shorten(id)}` : shorten(id) },
    position: { x: (idx % 4) * 240, y: Math.floor(idx / 4) * 120 },
    style: { background: id === seedId ? '#FEF3C7' : '#F1F5F9' },
  }));
  const edges: Edge[] = reworks.map((r) => ({
    id: `e-${r.id}`,
    source: r.parentCaseId,
    target: r.reworkCaseId,
    label: `回数 ${r.reworkCount}`,
    markerEnd: { type: MarkerType.ArrowClosed },
  }));
  return { nodes, edges };
}

function shorten(uuid: string): string {
  return uuid.length > 12 ? `${uuid.slice(0, 8)}…${uuid.slice(-4)}` : uuid;
}
