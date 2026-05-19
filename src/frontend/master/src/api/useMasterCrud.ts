import { useQuery, useMutation, useQueryClient, type QueryKey } from '@tanstack/react-query';
import { api } from './client';

// マスタ一覧の標準フェッチ。asOfUtc を query param に展開する。
export function useMasterList<T>(
  queryKey: QueryKey,
  endpoint: string,
  options?: { asOfUtc?: string | null; search?: string },
): ReturnType<typeof useQuery<T[]>> {
  return useQuery<T[]>({
    queryKey,
    queryFn: async () => {
      const params = new URLSearchParams();
      if (options?.asOfUtc) params.set('as_of', options.asOfUtc);
      if (options?.search) params.set('q', options.search);
      const url = params.toString() ? `${endpoint}?${params.toString()}` : endpoint;
      const result = await api.getList<T>(url);
      return result.data;
    },
  });
}

// 論理削除（deleted_at 設定）。マスタ物理削除禁止規約に準拠（src/CLAUDE.md §マスタの不変ルール）。
export function useDeprecateMaster(
  endpoint: string,
  invalidateKey: QueryKey,
): ReturnType<typeof useMutation<unknown, Error, string>> {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      return api.patch(`${endpoint}/${id}`, { deleted_at: new Date().toISOString() });
    },
    // 楽観的更新禁止: サーバ確定後にキャッシュを必ず無効化する（src/frontend/master/CLAUDE.md §データフェッチ規約）
    onSettled: () => queryClient.invalidateQueries({ queryKey: invalidateKey }),
  });
}

// マスタ新規作成（汎用）。
export function useCreateMaster<TInput, TOutput = TInput>(
  endpoint: string,
  invalidateKey: QueryKey,
): ReturnType<typeof useMutation<TOutput, Error, TInput>> {
  const queryClient = useQueryClient();
  return useMutation<TOutput, Error, TInput>({
    mutationFn: async (body) => {
      const result = await api.post<TOutput>(endpoint, body);
      return result.data;
    },
    onSettled: () => queryClient.invalidateQueries({ queryKey: invalidateKey }),
  });
}

// マスタ更新（汎用）。
export function useUpdateMaster<TInput, TOutput = TInput>(
  endpoint: string,
  invalidateKey: QueryKey,
): ReturnType<typeof useMutation<TOutput, Error, { id: string; patch: TInput }>> {
  const queryClient = useQueryClient();
  return useMutation<TOutput, Error, { id: string; patch: TInput }>({
    mutationFn: async ({ id, patch }) => {
      const result = await api.patch<TOutput>(`${endpoint}/${id}`, patch);
      return result.data;
    },
    onSettled: () => queryClient.invalidateQueries({ queryKey: invalidateKey }),
  });
}
