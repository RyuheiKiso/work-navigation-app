import type React from 'react';
import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Box, Chip, Stack } from '@mui/material';
import { DataGrid, type GridColDef } from '@mui/x-data-grid';
import type { Sop } from '@wnav/shared/types';
import { resolveLocale } from '@wnav/shared/i18n';
import { api } from '@/api/client';
import { queryKeys } from '@/api/queryKeys';
import { MasterListShell } from '@/components/MasterListShell';

// SCR-MA-015 リワーク SOP。sop_type=REWORK の SOP を絞り込み表示・版管理（docs/05/05_WebAPP詳細設計/17_ReworkSopEditor詳細設計.md）。
export function ReworkSopPage(): React.ReactElement {
  const [search, setSearch] = useState('');
  const sopsQuery = useQuery({
    queryKey: queryKeys.master.reworkSops(),
    queryFn: async (): Promise<Sop[]> => {
      const r = await api.getList<Sop>('/master/sops');
      return r.data.filter((s) => s.sopType === 'REWORK' && s.deletedAt === null);
    },
  });

  const filtered = sopsQuery.data?.filter(
    (s) =>
      !search ||
      s.sopCode.toLowerCase().includes(search.toLowerCase()) ||
      resolveLocale(s.nameJson, 'ja').toLowerCase().includes(search.toLowerCase()),
  ) ?? [];

  const columns: GridColDef<Sop>[] = [
    { field: 'sopCode', headerName: 'SOP コード', width: 200 },
    { field: 'nameJson', headerName: '名称', flex: 1, valueGetter: (_, row) => resolveLocale(row.nameJson, 'ja') },
    {
      field: 'currentVersionId',
      headerName: '状態',
      width: 120,
      renderCell: ({ row }) =>
        row.currentVersionId ? (
          <Chip label="公開済" color="success" size="small" />
        ) : (
          <Chip label="未公開" color="default" size="small" />
        ),
    },
  ];

  return (
    <MasterListShell
      title="リワーク SOP"
      subtitle="sop_type=REWORK の SOP 一覧（FR-RW-010）"
      search={search}
      onSearchChange={setSearch}
    >
      <Stack>
        <Box sx={{ width: '100%' }}>
          <DataGrid
            rows={filtered}
            columns={columns}
            loading={sopsQuery.isLoading}
            getRowId={(r) => r.id}
            autoHeight
            pageSizeOptions={[10, 25, 50]}
            initialState={{ pagination: { paginationModel: { pageSize: 25, page: 0 } } }}
            aria-label="リワーク SOP 一覧"
          />
        </Box>
      </Stack>
    </MasterListShell>
  );
}
