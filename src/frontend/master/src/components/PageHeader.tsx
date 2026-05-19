import type React from 'react';
import { Box, Breadcrumbs, Link as MuiLink, Typography } from '@mui/material';
import { Link as RouterLink, useLocation } from 'react-router-dom';

interface Breadcrumb {
  label: string;
  to?: string;
}

// ページタイトル + パンくず。RBAC バッジは MainLayout 側で AppBar に表示するため重複させない。
export function PageHeader({
  title,
  subtitle,
  breadcrumbs,
  actions,
}: {
  title: string;
  subtitle?: string;
  breadcrumbs?: Breadcrumb[];
  actions?: React.ReactNode;
}): React.ReactElement {
  const { pathname } = useLocation();
  const crumbs: Breadcrumb[] = breadcrumbs ?? [{ label: 'ホーム', to: '/' }, { label: title }];
  void pathname;
  return (
    <Box mb={3}>
      <Breadcrumbs aria-label="パンくずリスト" sx={{ mb: 1 }}>
        {crumbs.map((c, i) =>
          c.to && i < crumbs.length - 1 ? (
            <MuiLink key={c.label} component={RouterLink} to={c.to} color="inherit" underline="hover">
              {c.label}
            </MuiLink>
          ) : (
            <Typography key={c.label} color="text.primary">
              {c.label}
            </Typography>
          ),
        )}
      </Breadcrumbs>
      <Box display="flex" alignItems="center" justifyContent="space-between" gap={2}>
        <Box>
          <Typography variant="h1" component="h1">
            {title}
          </Typography>
          {subtitle && (
            <Typography variant="body2" color="text.secondary">
              {subtitle}
            </Typography>
          )}
        </Box>
        {actions && <Box>{actions}</Box>}
      </Box>
    </Box>
  );
}
