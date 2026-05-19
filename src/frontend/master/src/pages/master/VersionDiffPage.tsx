import type React from 'react';
import { useState } from 'react';
import { useParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { Box, Stack, MenuItem, Select, FormControl, InputLabel, Alert, Typography } from '@mui/material';
import type { MasterVersion } from '@wnav/shared/types';
import { api } from '@/api/client';
import { queryKeys } from '@/api/queryKeys';
import { PageHeader } from '@/components/PageHeader';
import { VersionDiffViewer } from '@/components/VersionDiffViewer';

// SCR-MA-010 版差分（docs/05/05_WebAPP詳細設計/05_MasterVersionDiff詳細設計.md）。
// ベース版と HEAD 版を選択して項目ごとの差分を side-by-side で表示。
export function VersionDiffPage(): React.ReactElement {
  const { id } = useParams<{ id: string }>();
  const [baseId, setBaseId] = useState<string>('');
  const [headId, setHeadId] = useState<string>('');

  const versionsQuery = useQuery({
    queryKey: queryKeys.master.sopVersions(id ?? ''),
    queryFn: async (): Promise<MasterVersion[]> => {
      const r = await api.getList<MasterVersion>(`/master/sops/${id}/versions`);
      return r.data;
    },
    enabled: !!id,
  });

  const versions = versionsQuery.data ?? [];
  const base = versions.find((v) => v.id === baseId);
  const head = versions.find((v) => v.id === headId);
  const sameVersion = baseId && headId && baseId === headId;

  return (
    <Box>
      <PageHeader title="バージョン差分" subtitle="公開済みバージョン同士の項目別差分を表示" />
      <Stack direction="row" spacing={2} mb={2}>
        <FormControl fullWidth>
          <InputLabel>ベースバージョン</InputLabel>
          <Select
            value={baseId}
            label="ベースバージョン"
            onChange={(e) => setBaseId(e.target.value)}
            inputProps={{ 'aria-label': 'ベースバージョン' }}
          >
            {versions.map((v) => (
              <MenuItem key={v.id} value={v.id}>
                {v.version} ({v.status})
              </MenuItem>
            ))}
          </Select>
        </FormControl>
        <FormControl fullWidth>
          <InputLabel>HEAD バージョン</InputLabel>
          <Select
            value={headId}
            label="HEAD バージョン"
            onChange={(e) => setHeadId(e.target.value)}
            inputProps={{ 'aria-label': 'HEAD バージョン' }}
          >
            {versions.map((v) => (
              <MenuItem key={v.id} value={v.id}>
                {v.version} ({v.status})
              </MenuItem>
            ))}
          </Select>
        </FormControl>
      </Stack>

      {sameVersion && <Alert severity="warning">ベースと HEAD が同一バージョンです</Alert>}

      {base && head && !sameVersion ? (
        <VersionDiffViewer
          beforeLabel={`v${base.version}`}
          afterLabel={`v${head.version}`}
          fields={[
            { field: 'version', before: base.version, after: head.version, changed: base.version !== head.version },
            { field: 'status', before: base.status, after: head.status, changed: base.status !== head.status },
            {
              field: 'changeSummary',
              before: base.changeSummary,
              after: head.changeSummary,
              changed: base.changeSummary !== head.changeSummary,
            },
            { field: 'stepCount', before: base.stepCount, after: head.stepCount, changed: base.stepCount !== head.stepCount },
            { field: 'publishedAt', before: base.publishedAt, after: head.publishedAt, changed: base.publishedAt !== head.publishedAt },
          ]}
        />
      ) : (
        <Typography color="text.secondary">2 つのバージョンを選択してください</Typography>
      )}
    </Box>
  );
}
