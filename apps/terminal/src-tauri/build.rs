// 対応 §: ロードマップ §7.1
// Tauri ビルドスクリプト。`tauri-build` がリソース統合を行う。

fn main() {
    // tauri-build が tauri.conf.json を読み込んで native コードを生成する
    tauri_build::build();
}
