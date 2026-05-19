// ページネーションの共通型定義
// 全リスト API で使用する汎用ページネーション型。
// デフォルトは page=1, per_page=20 とする。

use serde::{Deserialize, Serialize};

/// ページネーション済みレスポンス型。
/// 全リスト API のレスポンスボディとして使用する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    /// 取得したアイテム一覧
    pub items: Vec<T>,
    /// 総件数
    pub total: u64,
    /// 現在のページ番号（1 基準）
    pub page: u32,
    /// 1 ページあたりの件数
    pub per_page: u32,
}

/// ページネーションリクエストパラメータ。
/// クエリパラメータから変換して使用する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// ページ番号（1 基準）
    pub page: u32,
    /// 1 ページあたりの件数
    pub per_page: u32,
}

impl Default for Pagination {
    /// デフォルト値: page=1, per_page=20
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
        }
    }
}
