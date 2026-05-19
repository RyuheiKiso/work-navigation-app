// figment Provider を合成して完全な設定ツリーを構築する
// base YAML → profile YAML → 環境変数オーバーレイ → secret_ref 解決 の順でマージする

use figment::{
    Figment,
    providers::{Env, Format, Yaml},
};

use crate::{error::ConfigError, profile::Profile, secret_ref::SecretRefProvider};

/// 完全な設定ツリーを構築する figment を返す
/// WNAV_PROFILE 必須。未設定・schema_version 不一致はエラーを返す
pub fn build_figment() -> Result<Figment, ConfigError> {
    // プロファイルを最初に確定して以降の設定ファイル名解決に使用する
    let profile = Profile::from_env()?;
    let dir = config_dir();

    // base YAML から開始してプロファイル YAML → 環境変数の順でマージする
    // 後からマージするほど優先度が高くなるという figment のマージ規則を利用する
    let figment = Figment::new()
        // すべての環境に共通する既定値を最初にロードする
        .merge(Yaml::file(dir.join("config.base.yml")))
        // 環境固有の上書き値をマージする（配列は完全置換される）
        .merge(Yaml::file(dir.join(format!("config.{profile}.yml"))))
        // WNAV__SECTION__KEY 形式の環境変数で動的に上書きできるようにする
        .merge(Env::prefixed("WNAV__").split("__"));

    // schema_version を検証してから secret_ref 解決を行う
    // 不正なスキーマ上で secret_ref 解決を試みると混乱するため順序を守る
    check_schema_version(&figment)?;

    // secret_ref をすべて解決して最終的な figment を組み立てる
    let mut resolver = SecretRefProvider::new();
    resolver.resolve_from_figment(&figment)?;

    // secret_ref を解決済み値で上書きした最終 figment を返す
    let final_figment = figment.merge(resolver);

    Ok(final_figment)
}

/// schema_version が期待値（1）であることを確認する
/// 不一致は設定ファイルとクレートバージョンの乖離を示すため起動を止める
fn check_schema_version(figment: &Figment) -> Result<(), ConfigError> {
    // serde でデシリアライズして schema_version を取得する（to_i128 より確実）
    // figment の find_value は整数型の扱いが Num(Pos/Neg) で分かれるため
    // serde の u32 デシリアライズ経由が最も安全
    #[derive(serde::Deserialize)]
    struct VersionOnly {
        schema_version: Option<u32>,
    }
    let got: u32 = figment
        .extract::<VersionOnly>()
        .ok()
        .and_then(|v| v.schema_version)
        .unwrap_or(0);

    if got != 1 {
        return Err(ConfigError::SchemaVersionMismatch { expected: 1, got });
    }
    Ok(())
}

/// 設定ファイルディレクトリのパスを解決する
/// 優先度: WNAV_CONFIG_DIR > CARGO_MANIFEST_DIR からの相対パス > /etc/wnav/config
fn config_dir() -> std::path::PathBuf {
    // 明示的に指定された場合はそのパスを使用する（CI・Docker での上書きに使う）
    if let Ok(dir) = std::env::var("WNAV_CONFIG_DIR") {
        return dir.into();
    }

    // 開発環境: Cargo ワークスペースからの相対パスで infra/config を探す
    // CARGO_MANIFEST_DIR は crates/wnav_config を指すため 2 段上に遡る
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let candidate = std::path::PathBuf::from(manifest_dir)
            // crates/wnav_config → crates へ
            .parent()
            .and_then(|p| p.parent()) // crates → backend/src へ（workspace root の crates ディレクトリ）
            .map(|p| p.join("../../infra/config")) // backend → src → infra/config
            .unwrap_or_default();

        // 正規化してシンボリックリンクを解決する
        if let Ok(canonical) = candidate.canonicalize() {
            if canonical.exists() {
                return canonical;
            }
        }
    }

    // 本番環境の標準配置パス（Dockerfile での COPY 先）
    "/etc/wnav/config".into()
}
