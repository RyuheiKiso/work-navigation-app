// secret_ref: "<scheme>:<id>" を平文値に解決する figment Provider を実装する
// YAML ツリー内の { secret_ref: "env:VAR" } 形式のマップを検出して実値に置換する

use figment::{
    Metadata, Profile, Provider, map,
    value::{Dict, Map, Value},
};
use std::collections::HashMap;

// secret_ref 解決時に発生する個別エラー（ConfigError とは別に定義して詳細を保持する）
#[derive(Debug, thiserror::Error)]
pub enum SecretResolveError {
    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),

    #[error("Failed to read file '{path}': {source}")]
    FileReadFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Docker secret not found: /run/secrets/{0}")]
    DockerSecretNotFound(String),

    #[error("Unknown secret_ref scheme: '{0}'. Supported: env, file, docker_secret")]
    UnknownScheme(String),
}

/// secret_ref スキームを解決するトレイト
/// 新しいスキーム（DPAPI 等）を追加するときはこのトレイトを実装する
pub trait SecretResolver: Send + Sync {
    // スキーム識別子（"env" / "file" / "docker_secret" 等）を返す
    fn scheme(&self) -> &'static str;
    // 与えられた id から機密の実値を解決する
    fn resolve(&self, id: &str) -> Result<String, SecretResolveError>;
}

/// 環境変数からシークレットを解決するリゾルバ
/// 開発環境・CI 環境での使用を想定する
pub struct EnvSecretResolver;

impl SecretResolver for EnvSecretResolver {
    fn scheme(&self) -> &'static str {
        "env"
    }

    // 環境変数が存在しない場合はエラーにして起動を失敗させる
    fn resolve(&self, id: &str) -> Result<String, SecretResolveError> {
        std::env::var(id).map_err(|_| SecretResolveError::EnvVarNotFound(id.to_string()))
    }
}

/// ファイルからシークレットを解決するリゾルバ
/// 末尾の改行を除去して純粋な値のみを返す
pub struct FileSecretResolver;

impl SecretResolver for FileSecretResolver {
    fn scheme(&self) -> &'static str {
        "file"
    }

    // ファイル読み込み失敗はパスを含めてエラーを返し原因特定を容易にする
    fn resolve(&self, path: &str) -> Result<String, SecretResolveError> {
        std::fs::read_to_string(path)
            .map(|s| s.trim().to_string())
            .map_err(|e| SecretResolveError::FileReadFailed {
                path: path.to_string(),
                source: e,
            })
    }
}

/// Docker secrets（/run/secrets/）からシークレットを解決するリゾルバ
/// Docker Swarm / Docker Compose secrets での本番運用を想定する
pub struct DockerSecretResolver;

impl SecretResolver for DockerSecretResolver {
    fn scheme(&self) -> &'static str {
        "docker_secret"
    }

    // Docker secrets の標準パスから読み込む。失敗時は明示的なエラーを返す
    fn resolve(&self, name: &str) -> Result<String, SecretResolveError> {
        let path = format!("/run/secrets/{name}");
        std::fs::read_to_string(&path)
            .map(|s| s.trim().to_string())
            .map_err(|_| SecretResolveError::DockerSecretNotFound(name.to_string()))
    }
}

// Windows 環境では DPAPI によるシークレット解決を提供する
// Windows Server 2022 上の IIS デプロイで暗号化された認証情報を扱う
#[cfg(target_os = "windows")]
pub struct DpapiSecretResolver;

#[cfg(target_os = "windows")]
impl SecretResolver for DpapiSecretResolver {
    fn scheme(&self) -> &'static str {
        "dpapi"
    }

    // DPAPI はシステムユーザーの資格情報で保護された値を復号する
    // 現時点では実装プレースホルダ。Windows API ラッパークレートが確定次第実装する
    fn resolve(&self, _id: &str) -> Result<String, SecretResolveError> {
        // DPAPI 実装は Windows 専用クレート確定後に追加する
        Err(SecretResolveError::UnknownScheme(
            "dpapi (not yet implemented)".to_string(),
        ))
    }
}

/// YAML ツリーを走査して secret_ref マップノードを平文値に置換する figment Provider
/// figment のマージ最終段に配置することで他 Provider の値を上書きしない
pub struct SecretRefProvider {
    resolvers: Vec<Box<dyn SecretResolver>>,
    // 解決済みのフラットな key-value マップ（走査結果を保持する）
    resolved: HashMap<String, String>,
}

impl SecretRefProvider {
    /// 利用可能なリゾルバを全て登録して初期化する
    /// リゾルバの登録順序は解決の優先度には影響しない（スキーム名で一意に決まる）
    pub fn new() -> Self {
        // 非 Windows 環境では変数への再代入が発生しないため mut 不要だが Windows では push が必要
        #[cfg(not(target_os = "windows"))]
        let resolvers: Vec<Box<dyn SecretResolver>> = vec![
            Box::new(EnvSecretResolver),
            Box::new(FileSecretResolver),
            Box::new(DockerSecretResolver),
        ];
        // Windows 環境では DPAPI リゾルバを追加するため mut が必要
        #[cfg(target_os = "windows")]
        let mut resolvers: Vec<Box<dyn SecretResolver>> = vec![
            Box::new(EnvSecretResolver),
            Box::new(FileSecretResolver),
            Box::new(DockerSecretResolver),
        ];
        #[cfg(target_os = "windows")]
        resolvers.push(Box::new(DpapiSecretResolver));

        Self {
            resolvers,
            resolved: HashMap::new(),
        }
    }

    /// 既マージ済みの figment からシークレット参照を解決する
    /// YAML ツリーに含まれる { secret_ref: "scheme:id" } を走査して平文に置換する
    pub fn resolve_from_figment(
        &mut self,
        figment: &figment::Figment,
    ) -> Result<(), crate::error::ConfigError> {
        // figment の現在値を取得してシークレット参照を抽出する
        if let Ok(value) = figment.find_value("") {
            self.walk_and_resolve("", &value)?;
        }
        Ok(())
    }

    /// figment Value ツリーを再帰的に走査して secret_ref マップを検出・解決する
    fn walk_and_resolve(
        &mut self,
        path: &str,
        value: &figment::value::Value,
    ) -> Result<(), crate::error::ConfigError> {
        match value {
            figment::value::Value::Dict(_, dict) => {
                // { secret_ref: "scheme:id" } 形式のマップを検出する
                if let Some(secret_ref_val) = dict.get("secret_ref") {
                    if let Some(ref_str) = secret_ref_val.as_str() {
                        // スキームと ID を ":" で分割して対応するリゾルバを選択する
                        let resolved = self.resolve_secret_ref(ref_str).map_err(|reason| {
                            crate::error::ConfigError::SecretRefResolution {
                                secret_ref: ref_str.to_string(),
                                reason,
                            }
                        })?;
                        self.resolved.insert(path.to_string(), resolved);
                        return Ok(());
                    }
                }
                // secret_ref でなければ各キーを再帰的に走査する
                for (key, child) in dict {
                    let child_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{path}.{key}")
                    };
                    self.walk_and_resolve(&child_path, child)?;
                }
            }
            figment::value::Value::Array(_, arr) => {
                // 配列要素もインデックス付きパスで再帰走査する
                for (i, child) in arr.iter().enumerate() {
                    let child_path = format!("{path}[{i}]");
                    self.walk_and_resolve(&child_path, child)?;
                }
            }
            // スカラー値は secret_ref を含まないのでスキップする
            _ => {}
        }
        Ok(())
    }

    /// "scheme:id" 文字列を解析して対応するリゾルバで解決する
    fn resolve_secret_ref(&self, ref_str: &str) -> Result<String, String> {
        // ":" で最初の区切りのみ分割してスキームと ID を取り出す
        let Some((scheme, id)) = ref_str.split_once(':') else {
            return Err(format!(
                "invalid format (expected 'scheme:id'): '{ref_str}'"
            ));
        };

        // 登録済みリゾルバからスキームに一致するものを探す
        let resolver = self
            .resolvers
            .iter()
            .find(|r| r.scheme() == scheme)
            .ok_or_else(|| {
                format!(
                    "unknown scheme '{scheme}'. Supported: {}",
                    self.resolvers
                        .iter()
                        .map(|r| r.scheme())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;

        resolver.resolve(id).map_err(|e| e.to_string())
    }
}

// SecretRefProvider はリゾルバリストの初期化が必要なため Default = new() とする
impl Default for SecretRefProvider {
    fn default() -> Self {
        Self::new()
    }
}

// figment の Provider トレイトを実装して Figment のマージチェーンに組み込む
impl Provider for SecretRefProvider {
    fn metadata(&self) -> Metadata {
        Metadata::named("secret_ref resolver")
    }

    // 解決済みシークレットを figment の Dict として返す
    // 既マージ済みの値をドット記法のパスで上書きすることで secret_ref を置換する
    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        let mut dict = Dict::new();

        // 解決済み値をネストされた Dict に変換する（"a.b.c" → ネスト構造）
        for (path, value) in &self.resolved {
            insert_nested(&mut dict, path, Value::from(value.clone()));
        }

        // デフォルトプロファイルとして返す
        Ok(map![Profile::Default => dict])
    }
}

/// ドット区切りパス（"database.write.password"）を再帰的な Dict に変換して挿入する
fn insert_nested(dict: &mut Dict, path: &str, value: Value) {
    if let Some((head, tail)) = path.split_once('.') {
        // 中間キーの Dict を取得または新規作成する
        let child = dict
            .entry(head.to_string())
            .or_insert_with(|| Value::from(Dict::new()));

        if let Value::Dict(_, child_dict) = child {
            insert_nested(child_dict, tail, value);
        }
    } else {
        // 末端キーに値を直接挿入する
        dict.insert(path.to_string(), value);
    }
}
