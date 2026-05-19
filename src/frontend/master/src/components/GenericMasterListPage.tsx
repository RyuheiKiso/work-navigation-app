import type React from 'react';
import { useState, type ReactNode } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Box,
  Button,
  Chip,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Stack,
  Alert,
} from '@mui/material';
import { DataGrid, type GridColDef } from '@mui/x-data-grid';
import { Add, Block, Edit } from '@mui/icons-material';
import type { QueryKey } from '@tanstack/react-query';
import { useMasterList, useDeprecateMaster, useCreateMaster, useUpdateMaster } from '@/api/useMasterCrud';
import { MasterListShell } from './MasterListShell';
import { ImpactRangePreview } from './ImpactRangePreview';

// 物理削除禁止（src/CLAUDE.md §マスタの不変ルール）の論理削除しか持たないマスタ用の汎用一覧画面。
export interface GenericMasterListPageProps<T extends { id: string; deletedAt: string | null }, TCreate> {
  title: string;
  subtitle?: string;
  endpoint: string;
  queryKeyBuilder: (asOf: string | null) => QueryKey;
  columns: GridColDef<T>[];
  // 作成フォームの初期値（呼び出し側が型を制約する）
  initialCreateForm: TCreate;
  // 既存レコードから編集フォームの初期値を生成（指定時はインライン編集ダイアログが有効になる）
  initialEditForm?: (item: T) => TCreate;
  // 作成・編集フォーム本体（state を呼び出し側で保持しない設計）
  renderCreateForm: (state: {
    value: TCreate;
    onChange: (next: TCreate) => void;
    error: string | null;
    setError: (msg: string | null) => void;
  }) => ReactNode;
  // 作成前バリデーション（OK のとき payload を返す）
  validateAndBuildPayload: (form: TCreate) => { ok: true; payload: Partial<T> } | { ok: false; message: string };
  // 廃止確認ダイアログのラベル取得
  labelOf: (item: T) => string;
  // 廃止時の影響範囲リスト取得（任意）
  fetchImpacts?: (id: string) => Promise<Array<{ type: 'work_order' | 'work_execution' | 'sop_version'; id: string; label: string }>>;
  // 行のステータス表示（任意。デフォルトは「廃止 / 有効」）
  renderStatus?: (item: T) => ReactNode;
  // 専用編集画面への遷移パス（任意。initialEditForm と排他。指定時はダイアログ編集を無効化）
  editRoute?: (id: string) => string;
}

export function GenericMasterListPage<T extends { id: string; deletedAt: string | null }, TCreate>({
  title,
  subtitle,
  endpoint,
  queryKeyBuilder,
  columns,
  initialCreateForm,
  initialEditForm,
  renderCreateForm,
  validateAndBuildPayload,
  labelOf,
  renderStatus,
  editRoute,
}: GenericMasterListPageProps<T, TCreate>): React.ReactElement {
  const navigate = useNavigate();
  const [asOfUtc, setAsOfUtc] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [openCreate, setOpenCreate] = useState(false);
  const [editingItem, setEditingItem] = useState<T | null>(null);
  const [confirmDeprecate, setConfirmDeprecate] = useState<T | null>(null);
  const [createForm, setCreateForm] = useState<TCreate>(initialCreateForm);
  const [editForm, setEditForm] = useState<TCreate>(initialCreateForm);
  const [createError, setCreateError] = useState<string | null>(null);
  const [editError, setEditError] = useState<string | null>(null);

  const queryKey = queryKeyBuilder(asOfUtc);
  const { data, isLoading, error } = useMasterList<T>(queryKey, endpoint, { asOfUtc, search });
  const deprecateMutation = useDeprecateMaster(endpoint, queryKey);
  const create = useCreateMaster<Partial<T>, T>(endpoint, queryKey);
  const update = useUpdateMaster<Partial<T>, T>(endpoint, queryKey);

  const openEditDialog = (item: T): void => {
    if (!initialEditForm) return;
    setEditingItem(item);
    setEditForm(initialEditForm(item));
    setEditError(null);
  };

  const allColumns: GridColDef<T>[] = [
    ...columns,
    {
      field: '__status',
      headerName: '状態',
      width: 120,
      sortable: false,
      renderCell: ({ row }) =>
        renderStatus ? (
          renderStatus(row)
        ) : row.deletedAt ? (
          <Chip label="廃止" color="error" size="small" />
        ) : (
          <Chip label="有効" color="success" size="small" />
        ),
    },
    {
      field: '__actions',
      headerName: '操作',
      width: 220,
      sortable: false,
      renderCell: ({ row }) => (
        <Stack direction="row" spacing={1}>
          {editRoute && (
            <Button
              size="small"
              startIcon={<Edit />}
              onClick={() => navigate(editRoute(row.id))}
              disabled={!!row.deletedAt}
              aria-label={`${labelOf(row)} を編集`}
            >
              編集
            </Button>
          )}
          {!editRoute && initialEditForm && (
            <Button
              size="small"
              startIcon={<Edit />}
              onClick={() => openEditDialog(row)}
              disabled={!!row.deletedAt}
              aria-label={`${labelOf(row)} を編集`}
            >
              編集
            </Button>
          )}
          {!row.deletedAt && (
            <Button
              size="small"
              color="error"
              startIcon={<Block />}
              onClick={() => setConfirmDeprecate(row)}
              aria-label={`${labelOf(row)} を廃止`}
            >
              廃止
            </Button>
          )}
        </Stack>
      ),
    },
  ];

  const submitCreate = (): void => {
    const validation = validateAndBuildPayload(createForm);
    if (!validation.ok) { setCreateError(validation.message); return; }
    create.mutate(validation.payload, {
      onSuccess: () => { setOpenCreate(false); setCreateForm(initialCreateForm); setCreateError(null); },
      onError: (e: unknown) => setCreateError(e instanceof Error ? e.message : '作成に失敗しました'),
    });
  };

  const submitEdit = (): void => {
    if (!editingItem) return;
    const validation = validateAndBuildPayload(editForm);
    if (!validation.ok) { setEditError(validation.message); return; }
    update.mutate({ id: editingItem.id, patch: validation.payload }, {
      onSuccess: () => { setEditingItem(null); setEditError(null); },
      onError: (e: unknown) => setEditError(e instanceof Error ? e.message : '更新に失敗しました'),
    });
  };

  return (
    <MasterListShell
      title={title}
      {...(subtitle !== undefined ? { subtitle } : {})}
      search={search}
      onSearchChange={setSearch}
      onAsOfChange={setAsOfUtc}
      actions={
        <Button variant="contained" startIcon={<Add />} onClick={() => setOpenCreate(true)} aria-label={`${title} を新規作成`}>
          新規作成
        </Button>
      }
    >
      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error instanceof Error ? error.message : 'データ取得に失敗しました'}
        </Alert>
      )}
      <Box sx={{ width: '100%' }}>
        <DataGrid
          rows={data ?? []}
          columns={allColumns}
          loading={isLoading}
          getRowId={(row) => row.id}
          autoHeight
          pageSizeOptions={[10, 25, 50]}
          initialState={{ pagination: { paginationModel: { pageSize: 25, page: 0 } } }}
          aria-label={`${title} テーブル`}
        />
      </Box>

      {/* 新規作成ダイアログ */}
      <Dialog open={openCreate} onClose={() => setOpenCreate(false)} fullWidth maxWidth="sm">
        <DialogTitle>{title} を新規作成</DialogTitle>
        <DialogContent>
          <Stack spacing={2} mt={1}>
            {createError && <Alert severity="error">{createError}</Alert>}
            {renderCreateForm({ value: createForm, onChange: setCreateForm, error: createError, setError: setCreateError })}
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setOpenCreate(false)} aria-label="作成をキャンセル">キャンセル</Button>
          <Button onClick={submitCreate} variant="contained" disabled={create.isPending} aria-label="保存">保存</Button>
        </DialogActions>
      </Dialog>

      {/* インライン編集ダイアログ（initialEditForm が指定されている場合のみ） */}
      <Dialog open={editingItem !== null} onClose={() => setEditingItem(null)} fullWidth maxWidth="sm">
        <DialogTitle>{title} を編集</DialogTitle>
        <DialogContent>
          <Stack spacing={2} mt={1}>
            {editError && <Alert severity="error">{editError}</Alert>}
            {renderCreateForm({ value: editForm, onChange: setEditForm, error: editError, setError: setEditError })}
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setEditingItem(null)} aria-label="編集をキャンセル">キャンセル</Button>
          <Button onClick={submitEdit} variant="contained" disabled={update.isPending} aria-label="更新">更新</Button>
        </DialogActions>
      </Dialog>

      {/* 廃止確認ダイアログ */}
      <Dialog open={confirmDeprecate !== null} onClose={() => setConfirmDeprecate(null)} fullWidth maxWidth="sm">
        <DialogTitle>{title} を廃止</DialogTitle>
        <DialogContent>
          {confirmDeprecate && (
            <Stack spacing={2}>
              <Alert severity="warning">
                {labelOf(confirmDeprecate)} を論理削除します。物理削除はされません。
              </Alert>
              <ImpactRangePreview items={[]} emptyMessage="影響範囲はありません" />
            </Stack>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setConfirmDeprecate(null)} aria-label="廃止をキャンセル">キャンセル</Button>
          <Button
            color="error"
            variant="contained"
            onClick={() => {
              if (confirmDeprecate) {
                deprecateMutation.mutate(confirmDeprecate.id);
                setConfirmDeprecate(null);
              }
            }}
            aria-label="廃止を確定"
          >
            廃止する
          </Button>
        </DialogActions>
      </Dialog>
    </MasterListShell>
  );
}
