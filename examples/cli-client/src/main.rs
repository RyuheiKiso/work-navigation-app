//! work-navigation-app サンプル CLI クライアント
//!
//! 対応 §: ロードマップ §10.3.5 §10.3.1 §14.2

// 結果型
use anyhow::Result;
// 標準引数
use std::env;

/// CLI エントリポイント
#[tokio::main]
async fn main() -> Result<()> {
    // コマンドライン引数を取得
    let args: Vec<String> = env::args().collect();
    // ベース URL を環境変数または引数から取得（既定 localhost:8080）
    let base = env::var("WNA_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    // サブコマンド名を取得（無ければヘルプ）
    let cmd = args.get(1).map(String::as_str).unwrap_or("help");

    // ディスパッチ
    match cmd {
        // ヘルスチェック
        "healthz" => healthz(&base).await,
        // Task 取得
        "get-task" => {
            // 第 2 引数として Task ID を要求
            let task_id = args
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("get-task <task_id>"))?;
            // 取得処理
            get_task(&base, task_id).await
        }
        // それ以外はヘルプ
        _ => {
            // 使い方を表示
            println!("使い方: wna-cli <healthz|get-task <id>>");
            // 異常終了ではなく 0 を返す
            Ok(())
        }
    }
}

// =====================================================================
// サブコマンド実装
// =====================================================================

/// /healthz を呼び出す
async fn healthz(base: &str) -> Result<()> {
    // URL を組み立てる
    let url = format!("{base}/healthz");
    // GET リクエスト
    let res = reqwest::get(&url).await?;
    // ステータスとボディを表示
    println!("status: {}", res.status());
    let body = res.text().await?;
    println!("body: {body}");
    // 正常終了
    Ok(())
}

/// /tasks/:id を呼び出す
async fn get_task(base: &str, id: &str) -> Result<()> {
    // URL を組み立てる
    let url = format!("{base}/tasks/{id}");
    // GET リクエスト
    let res = reqwest::get(&url).await?;
    // ステータス
    println!("status: {}", res.status());
    // ボディ
    let body = res.text().await?;
    println!("body: {body}");
    // 正常終了
    Ok(())
}
