// 対応 §: ロードマップ §7.1
// Tauri アプリのエントリポイント（バイナリ）。

// Windows ビルド時のコンソールウィンドウを抑止する
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// アプリ実行関数を呼び出す
fn main() {
    // ライブラリ側の run 関数を呼び出してアプリを起動する
    wna_terminal_tauri::run();
}
