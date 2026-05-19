import type React from 'react';
import { useMemo } from 'react';
import { NavLink, Outlet, useLocation } from 'react-router-dom';
import {
  AppBar,
  Box,
  Drawer,
  IconButton,
  List,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Toolbar,
  Typography,
  Divider,
  Collapse,
} from '@mui/material';
import MenuIcon from '@mui/icons-material/Menu';
import DashboardIcon from '@mui/icons-material/Dashboard';
import AccountTreeIcon from '@mui/icons-material/AccountTree';
import EngineeringIcon from '@mui/icons-material/Engineering';
import InventoryIcon from '@mui/icons-material/Inventory';
import DescriptionIcon from '@mui/icons-material/Description';
import PeopleIcon from '@mui/icons-material/People';
import VerifiedIcon from '@mui/icons-material/Verified';
import HistoryIcon from '@mui/icons-material/History';
import ScienceIcon from '@mui/icons-material/Science';
import BuildIcon from '@mui/icons-material/Build';
import AssessmentIcon from '@mui/icons-material/Assessment';
import { ExpandLess, ExpandMore } from '@mui/icons-material';
import type { UserRole } from '@wnav/shared/types';
import { useUiStore } from '@/stores/uiStore';
import { useAuth } from '@/auth/useAuth';
import { RBACBadge } from './RBACBadge';

const DRAWER_WIDTH = 260;

interface MenuItem {
  key: string;
  label: string;
  icon: React.ReactNode;
  path: string;
  roles: readonly UserRole[];
}

interface MenuGroup {
  key: string;
  label: string;
  items: MenuItem[];
}

const MENU_GROUPS: MenuGroup[] = [
  {
    key: 'master',
    label: 'マスタメンテ',
    items: [
      { key: 'processes', label: 'プロセス', icon: <AccountTreeIcon />, path: '/master/processes', roles: ['master_admin'] },
      { key: 'operations', label: 'オペレーション', icon: <EngineeringIcon />, path: '/master/operations', roles: ['master_admin'] },
      { key: 'products', label: '製品', icon: <InventoryIcon />, path: '/master/products', roles: ['master_admin'] },
      { key: 'sops', label: 'SOP 一覧', icon: <DescriptionIcon />, path: '/master/sops', roles: ['master_admin', 'quality_admin'] },
      { key: 'materials', label: '材料', icon: <InventoryIcon />, path: '/master/materials', roles: ['master_admin'] },
      { key: 'suppliers', label: '仕入先', icon: <PeopleIcon />, path: '/master/suppliers', roles: ['master_admin'] },
      { key: 'sampling', label: 'サンプリング計画', icon: <ScienceIcon />, path: '/master/sampling-plans', roles: ['quality_admin'] },
      { key: 'rework-sops', label: 'リワーク SOP', icon: <BuildIcon />, path: '/master/rework-sops', roles: ['master_admin'] },
      { key: 'rework-mappings', label: 'リワーク対応表', icon: <BuildIcon />, path: '/master/rework-sop-mappings', roles: ['master_admin', 'quality_admin'] },
      { key: 'report-templates', label: '帳票テンプレ', icon: <DescriptionIcon />, path: '/master/report-templates', roles: ['system_admin'] },
    ],
  },
  {
    key: 'console',
    label: '管理コンソール',
    items: [
      { key: 'dashboard', label: 'ダッシュボード', icon: <DashboardIcon />, path: '/console/dashboard', roles: ['system_admin', 'executive'] },
      { key: 'users', label: 'ユーザー', icon: <PeopleIcon />, path: '/console/users', roles: ['system_admin'] },
      { key: 'roles', label: 'ロール / スキル', icon: <PeopleIcon />, path: '/console/roles', roles: ['system_admin'] },
      { key: 'audit', label: '監査ログ', icon: <HistoryIcon />, path: '/console/audit-logs', roles: ['quality_admin', 'system_admin'] },
      { key: 'xes', label: 'XES エクスポート', icon: <AssessmentIcon />, path: '/console/xes-export', roles: ['quality_admin', 'system_admin'] },
      { key: 'backup', label: 'バックアップ', icon: <VerifiedIcon />, path: '/console/backup', roles: ['system_admin'] },
      { key: 'outbox', label: 'Outbox 監視', icon: <HistoryIcon />, path: '/console/outbox', roles: ['system_admin'] },
      { key: 'hash', label: 'ハッシュチェーン検証', icon: <VerifiedIcon />, path: '/console/hash-chain', roles: ['quality_admin', 'system_admin'] },
      { key: 'reports', label: '帳票生成', icon: <AssessmentIcon />, path: '/console/reports', roles: ['quality_admin', 'system_admin'] },
      { key: 'concessions', label: '特採承認', icon: <VerifiedIcon />, path: '/console/concession-approvals', roles: ['quality_admin'] },
      { key: 'iqc-dashboard', label: '受入検査ダッシュ', icon: <DashboardIcon />, path: '/console/iqc-dashboard', roles: ['quality_admin', 'executive'] },
      { key: 'iqc-lots', label: '受入ロット一覧', icon: <InventoryIcon />, path: '/console/iqc-lots', roles: ['quality_admin', 'system_admin'] },
      { key: 'disposition', label: 'ディスポジション', icon: <VerifiedIcon />, path: '/console/dispositions', roles: ['quality_admin', 'supervisor'] },
      { key: 'rework-list', label: 'リワーク一覧', icon: <BuildIcon />, path: '/console/rework-list', roles: ['quality_admin', 'supervisor'] },
      { key: 'rework-trace', label: 'リワーク追跡', icon: <BuildIcon />, path: '/console/rework-traceability', roles: ['quality_admin', 'system_admin'] },
    ],
  },
];

// AppBar + Drawer の左ナビ。ロールに応じてメニュー項目を表示/非表示する（src/frontend/master/CLAUDE.md §認証・認可）。
export function MainLayout(): React.ReactElement {
  const { sidebarOpen, toggleSidebar } = useUiStore();
  const { user } = useAuth();
  const location = useLocation();

  const visibleGroups = useMemo(() => {
    if (!user) return [];
    return MENU_GROUPS.map((group) => ({
      ...group,
      items: group.items.filter((item) => item.roles.some((r) => user.roles.includes(r))),
    })).filter((g) => g.items.length > 0);
  }, [user]);

  return (
    <Box sx={{ display: 'flex', minHeight: '100vh' }}>
      <AppBar position="fixed" sx={{ zIndex: (t) => t.zIndex.drawer + 1 }}>
        <Toolbar>
          <IconButton color="inherit" edge="start" onClick={toggleSidebar} aria-label="ナビゲーションを切替">
            <MenuIcon />
          </IconButton>
          <Typography variant="h1" component="div" sx={{ flexGrow: 1, fontSize: 20, ml: 2 }}>
            WNAV マスタ管理
          </Typography>
          {user && <RBACBadge roles={user.roles} />}
        </Toolbar>
      </AppBar>
      <Drawer
        variant="persistent"
        open={sidebarOpen}
        sx={{
          width: sidebarOpen ? DRAWER_WIDTH : 0,
          flexShrink: 0,
          '& .MuiDrawer-paper': { width: DRAWER_WIDTH, boxSizing: 'border-box' },
        }}
      >
        <Toolbar />
        <Box component="nav" aria-label="メインナビゲーション">
          {visibleGroups.map((group) => (
            <Box key={group.key}>
              <Typography variant="overline" sx={{ px: 2, pt: 2, display: 'block', color: 'text.secondary' }}>
                {group.label}
              </Typography>
              <List dense>
                {group.items.map((item) => {
                  const selected = location.pathname.startsWith(item.path);
                  return (
                    <ListItemButton
                      key={item.key}
                      component={NavLink}
                      to={item.path}
                      selected={selected}
                      aria-current={selected ? 'page' : undefined}
                    >
                      <ListItemIcon>{item.icon}</ListItemIcon>
                      <ListItemText primary={item.label} />
                    </ListItemButton>
                  );
                })}
              </List>
              <Divider />
            </Box>
          ))}
        </Box>
      </Drawer>
      <Box component="main" sx={{ flexGrow: 1, p: 3, mt: 8 }}>
        <Outlet />
      </Box>
    </Box>
  );
}

// eslint Collapse 未使用警告を抑止する目的の参照（将来サブメニュー化時に使用）
void Collapse;
void ExpandLess;
void ExpandMore;
