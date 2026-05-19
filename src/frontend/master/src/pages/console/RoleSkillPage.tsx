import type React from 'react';
import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Box, Stack } from '@mui/material';
import { DataGrid, type GridColDef } from '@mui/x-data-grid';
import type { Skill } from '@wnav/shared/types';
import { resolveLocale } from '@wnav/shared/i18n';
import { api } from '@/api/client';
import { MasterListShell } from '@/components/MasterListShell';

// SCR-MC-003 ロール / スキル管理（system_admin、FR-MA-015）。
export function RoleSkillPage(): React.ReactElement {
  const [search, setSearch] = useState('');

  const skillsQuery = useQuery({
    queryKey: ['console', 'skills'],
    queryFn: async (): Promise<Skill[]> => {
      const r = await api.getList<Skill>('/master/skills');
      return r.data;
    },
  });

  const filtered = (skillsQuery.data ?? []).filter(
    (s) =>
      !search ||
      s.skillCode.toLowerCase().includes(search.toLowerCase()) ||
      resolveLocale(s.nameJson, 'ja').toLowerCase().includes(search.toLowerCase()),
  );

  const columns: GridColDef<Skill>[] = [
    { field: 'skillCode', headerName: 'スキルコード', width: 200 },
    { field: 'nameJson', headerName: 'スキル名', flex: 1, valueGetter: (_, row) => resolveLocale(row.nameJson, 'ja') },
    { field: 'level', headerName: 'レベル', width: 120 },
  ];

  return (
    <MasterListShell
      title="ロール / スキル管理"
      subtitle="スキル割当・ロール設定"
      search={search}
      onSearchChange={setSearch}
    >
      <Stack>
        <Box sx={{ width: '100%' }}>
          <DataGrid
            rows={filtered}
            columns={columns}
            loading={skillsQuery.isLoading}
            getRowId={(r) => r.id}
            autoHeight
            pageSizeOptions={[10, 25, 50]}
            aria-label="スキルマスタ一覧"
          />
        </Box>
      </Stack>
    </MasterListShell>
  );
}
