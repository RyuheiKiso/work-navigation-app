import type React from 'react';
import { ReactElement, ReactNode } from 'react';
import { render, type RenderOptions, type RenderResult } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ThemeProvider } from '@mui/material/styles';
import CssBaseline from '@mui/material/CssBaseline';
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';
import { lightTheme } from '@/theme';

interface ProviderOptions {
  route?: string;
  queryClient?: QueryClient;
}

// 各テストで Providers をまとめて適用するヘルパー（楽観的更新禁止規約も反映）。
function buildClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: { retry: false, gcTime: 0, staleTime: 0 },
      mutations: { retry: false },
    },
  });
}

export function renderWithProviders(
  ui: ReactElement,
  options: ProviderOptions & Omit<RenderOptions, 'wrapper'> = {},
): RenderResult & { queryClient: QueryClient } {
  const { route = '/', queryClient = buildClient(), ...rest } = options;
  const Wrapper = ({ children }: { children: ReactNode }): React.ReactElement => (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider theme={lightTheme}>
        <CssBaseline />
        <LocalizationProvider dateAdapter={AdapterDayjs}>
          <MemoryRouter initialEntries={[route]}>{children}</MemoryRouter>
        </LocalizationProvider>
      </ThemeProvider>
    </QueryClientProvider>
  );
  const result = render(ui, { wrapper: Wrapper, ...rest });
  return { ...result, queryClient };
}
