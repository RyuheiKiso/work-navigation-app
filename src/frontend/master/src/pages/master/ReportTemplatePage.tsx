import type React from 'react';
import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Box, Alert, Chip } from '@mui/material';
import { DataGrid, type GridColDef } from '@mui/x-data-grid';
import { api } from '@/api/client';
import { MasterListShell } from '@/components/MasterListShell';

interface ReportTemplate {
  id: string;
  templateCode: string;
  name: string;
  category: 'RP-007' | 'RP-008' | 'RP-009' | 'RP-010';
  format: 'PDF' | 'XLSX' | 'CSV';
  updatedAt: string;
}

// SCR-MA-017 帳票テンプレ（system_admin、FR-MA-017）。RP-007〜010 のテンプレート設定。
export function ReportTemplatePage(): React.ReactElement {
  const [search, setSearch] = useState('');

  const query = useQuery({
    queryKey: ['master', 'report-templates'],
    queryFn: async (): Promise<ReportTemplate[]> => {
      try {
        const r = await api.getList<ReportTemplate>('/master/report-templates');
        return r.data;
      } catch {
        return [];
      }
    },
  });

  const filtered = (query.data ?? []).filter(
    (t) =>
      !search ||
      t.templateCode.toLowerCase().includes(search.toLowerCase()) ||
      t.name.toLowerCase().includes(search.toLowerCase()),
  );

  const columns: GridColDef<ReportTemplate>[] = [
    { field: 'templateCode', headerName: 'テンプレートコード', width: 200 },
    { field: 'name', headerName: '名称', flex: 1 },
    {
      field: 'category',
      headerName: 'カテゴリ',
      width: 120,
      renderCell: ({ value }) => <Chip label={String(value)} size="small" color="primary" />,
    },
    { field: 'format', headerName: 'フォーマット', width: 120 },
    { field: 'updatedAt', headerName: '更新日時', width: 200 },
  ];

  return (
    <MasterListShell
      title="帳票テンプレ"
      subtitle="RP-007〜010 の帳票テンプレート設定（system_admin 限定）"
      search={search}
      onSearchChange={setSearch}
    >
      {filtered.length === 0 && !query.isLoading && (
        <Alert severity="info" sx={{ mb: 2 }}>
          帳票テンプレートは未登録です
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
          aria-label="帳票テンプレート一覧"
        />
      </Box>
    </MasterListShell>
  );
}
