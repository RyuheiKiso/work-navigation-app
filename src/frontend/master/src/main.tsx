import React from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import { ThemeProvider } from '@mui/material/styles';
import CssBaseline from '@mui/material/CssBaseline';
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';
import dayjs from 'dayjs';
import 'dayjs/locale/ja';
import { lightTheme } from '@/theme';
import { ErrorBoundary } from '@/components/ErrorBoundary';
import { App } from '@/App';
import '@/i18n';
import 'reactflow/dist/style.css';
import { enableMocking } from '@/mocks/browser';

dayjs.locale('ja');

// TanStack Query: 監査対象データの楽観的更新を全面禁止するため、mutation 後は invalidateQueries で再フェッチを強制する。
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 2,
      staleTime: 30_000,
      refetchOnWindowFocus: false,
    },
    mutations: {
      onSettled: () => {
        // 個別 mutation の onSettled で invalidateQueries を行う規約のためここでは noop
      },
    },
  },
});

async function bootstrap(): Promise<void> {
  if (import.meta.env.DEV) {
    await enableMocking();
  }
  const rootEl = document.getElementById('root');
  if (!rootEl) throw new Error('root element not found');

  ReactDOM.createRoot(rootEl).render(
    <React.StrictMode>
      <ErrorBoundary>
        <QueryClientProvider client={queryClient}>
          <ThemeProvider theme={lightTheme}>
            <CssBaseline />
            <LocalizationProvider dateAdapter={AdapterDayjs} adapterLocale="ja">
              <BrowserRouter future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>
                <App />
              </BrowserRouter>
            </LocalizationProvider>
          </ThemeProvider>
          {import.meta.env.DEV && <ReactQueryDevtools initialIsOpen={false} />}
        </QueryClientProvider>
      </ErrorBoundary>
    </React.StrictMode>,
  );
}

void bootstrap();
