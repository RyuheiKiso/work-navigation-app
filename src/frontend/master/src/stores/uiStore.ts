import { create } from 'zustand';

// サイドナビ開閉・選択メニュー・テーマモードのみ。サーバー状態は TanStack Query 側で管理する。
interface UiState {
  sidebarOpen: boolean;
  selectedMenu: string;
  themeMode: 'light' | 'dark';
  setSidebarOpen: (open: boolean) => void;
  setSelectedMenu: (key: string) => void;
  setThemeMode: (mode: 'light' | 'dark') => void;
  toggleSidebar: () => void;
}

export const useUiStore = create<UiState>((set) => ({
  sidebarOpen: true,
  selectedMenu: '',
  themeMode: 'light',
  setSidebarOpen: (open) => set({ sidebarOpen: open }),
  setSelectedMenu: (key) => set({ selectedMenu: key }),
  setThemeMode: (mode) => set({ themeMode: mode }),
  toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
}));
