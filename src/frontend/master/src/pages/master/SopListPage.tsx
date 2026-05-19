import type React from 'react';
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Box, Button, Chip, Stack } from '@mui/material';
import { DataGrid, type GridColDef } from '@mui/x-data-grid';
import { Add, Edit, Visibility, AccountTree } from '@mui/icons-material';
import type { Sop } from '@wnav/shared/types';
import { resolveLocale } from '@wnav/shared/i18n';
import { useMasterList } from '@/api/useMasterCrud';
import { queryKeys } from '@/api/queryKeys';
import { MasterListShell } from '@/components/MasterListShell';

// SCR-MA-004 SOP 一覧。編集・プレビュー・バージョン管理への起点画面（FR-MA-004）。
export function SopListPage(): React.ReactElement {
  const navigate = useNavigate();
  const [search, setSearch] = useState('');

  const queryKey = queryKeys.master.sops({ search: search || undefined });
  const { data, isLoading, error } = useMasterList<Sop>(queryKey, '/master/sops', { search });

  const columns: GridColDef<Sop>[] = [
    { field: 'sopCode', headerName: 'SOP コード', width: 160 },
    {
      field: 'nameJson',
      headerName: '名称',
      flex: 1,
      valueGetter: (_, row) => resolveLocale(row.nameJson, 'ja'),
    },
    {
      field: 'sopType',
      headerName: '種別',
      width: 120,
      renderCell: ({ row }) => (
        <Chip
          label={row.sopType === 'REWORK' ? 'リワーク' : '標準'}
          color={row.sopType === 'REWORK' ? 'warning' : 'default'}
          size="small"
        />
      ),
    },
    {
      field: 'currentVersionId',
      headerName: '公開版',
      width: 100,
      renderCell: ({ row }) =>
        row.currentVersionId ? (
          <Chip label="公開中" color="success" size="small" />
        ) : (
          <Chip label="未公開" color="default" size="small" />
        ),
    },
    {
      field: 'deletedAt',
      headerName: '状態',
      width: 100,
      renderCell: ({ row }) =>
        row.deletedAt ? (
          <Chip label="廃止" color="error" size="small" />
        ) : (
          <Chip label="有効" color="success" size="small" />
        ),
    },
    {
      field: '_actions',
      headerName: '操作',
      width: 240,
      sortable: false,
      renderCell: ({ row }) => (
        <Stack direction="row" spacing={0.5}>
          <Button
            size="small"
            startIcon={<Edit />}
            onClick={() => navigate(`/master/sops/${row.id}/edit`)}
            disabled={!!row.deletedAt}
            aria-label={`${resolveLocale(row.nameJson, 'ja')} を編集`}
          >
            編集
          </Button>
          <Button
            size="small"
            startIcon={<Visibility />}
            onClick={() => navigate(`/master/sops/${row.id}/preview`)}
            aria-label={`${resolveLocale(row.nameJson, 'ja')} をプレビュー`}
          >
            確認
          </Button>
          <Button
            size="small"
            startIcon={<AccountTree />}
            onClick={() => navigate(`/master/sops/${row.id}/versions`)}
            aria-label={`${resolveLocale(row.nameJson, 'ja')} のバージョン一覧`}
          >
            版管理
          </Button>
        </Stack>
      ),
    },
  ];

  return (
    <MasterListShell
      title="SOP 一覧"
      subtitle="標準作業手順書の一覧・編集・版管理"
      search={search}
      onSearchChange={setSearch}
      actions={
        <Button
          variant="contained"
          startIcon={<Add />}
          onClick={() => navigate('/master/sops/new')}
          aria-label="SOP を新規作成"
        >
          新規作成
        </Button>
      }
    >
      <DataGrid
        rows={data ?? []}
        columns={columns}
        loading={isLoading}
        autoHeight
        disableRowSelectionOnClick
        pageSizeOptions={[25, 50, 100]}
        initialState={{ pagination: { paginationModel: { pageSize: 25 } } }}
        aria-label="SOP 一覧テーブル"
        getRowClassName={({ row }) => (row.deletedAt ? 'row-deprecated' : '')}
        sx={{ '& .row-deprecated': { opacity: 0.5 } }}
      />
      {error && (
        <Box mt={2} color="error.main" role="alert">
          データの取得に失敗しました
        </Box>
      )}
    </MasterListShell>
  );
}
