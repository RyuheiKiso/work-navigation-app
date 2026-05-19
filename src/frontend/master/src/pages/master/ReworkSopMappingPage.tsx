import type React from 'react';
import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Box, Alert } from '@mui/material';
import { DataGrid, type GridColDef } from '@mui/x-data-grid';
import { api } from '@/api/client';
import { MasterListShell } from '@/components/MasterListShell';

interface ReworkSopMapping {
  id: string;
  ncCategory: string;
  reworkType: string;
  targetSopId: string;
  targetSopName: string;
  createdAt: string;
}

// SCR-MA-016 リワーク対応表。不適合カテゴリ × リワーク種別 × 対象 SOP の対応表（FR-RW-012）。
export function ReworkSopMappingPage(): React.ReactElement {
  const [search, setSearch] = useState('');

  const query = useQuery({
    queryKey: ['master', 'rework-sop-mappings'],
    queryFn: async (): Promise<ReworkSopMapping[]> => {
      try {
        const r = await api.getList<ReworkSopMapping>('/master/rework-sop-mappings');
        return r.data;
      } catch {
        // 未実装エンドポイントへの暫定対応
        return [];
      }
    },
  });

  const filtered = (query.data ?? []).filter(
    (m) =>
      !search ||
      m.ncCategory.toLowerCase().includes(search.toLowerCase()) ||
      m.reworkType.toLowerCase().includes(search.toLowerCase()) ||
      m.targetSopName.toLowerCase().includes(search.toLowerCase()),
  );

  const columns: GridColDef<ReworkSopMapping>[] = [
    { field: 'ncCategory', headerName: '不適合カテゴリ', width: 200 },
    { field: 'reworkType', headerName: 'リワーク種別', width: 200 },
    { field: 'targetSopName', headerName: '対象 SOP', flex: 1 },
    { field: 'createdAt', headerName: '作成日時', width: 200 },
  ];

  return (
    <MasterListShell
      title="リワーク対応表"
      subtitle="不適合カテゴリと対応する SOP のマッピング管理"
      search={search}
      onSearchChange={setSearch}
    >
      {filtered.length === 0 && !query.isLoading && (
        <Alert severity="info" sx={{ mb: 2 }}>
          リワーク対応表は未登録です
        </Alert>
      )}
      <Box sx={{ width: '100%' }}>
        <DataGrid
          rows={filtered}
          columns={columns}
          loading={query.isLoading}
          getRowId={(r) => r.id}
          autoHeight
          pageSizeOptions={[10, 25]}
          initialState={{ pagination: { paginationModel: { pageSize: 10, page: 0 } } }}
          aria-label="リワーク対応表"
        />
      </Box>
    </MasterListShell>
  );
}
