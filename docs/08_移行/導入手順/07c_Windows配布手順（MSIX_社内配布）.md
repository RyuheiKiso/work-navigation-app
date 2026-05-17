# 07c Windows 配布手順（MSIX / 社内配布）

本章の責務は、`07_ハンディAPP配布手順（統合）.md` の Windows 固有サブ手順として、React Native Windows のビルド実行・MSIX パッケージへのコード署名・社内配布とインストール確認を確定することである。

---

## 1 本章の責務（07_統合書の Windows 固有サブ）

### 1-1. 07_統合書との関係

本章は `07_ハンディAPP配布手順（統合）.md` の Windows 専用サブ手順書である。本章で扱う手順は Windows OS 固有の操作に限定し、3 OS 共通の手順は統合書（07 章）を参照する。

| 担当範囲 | 文書 |
|---|---|
| 3 OS 共通フロー・EAS 設定・判定表 | `07_ハンディAPP配布手順（統合）.md` |
| Windows 固有ビルド・MSIX・署名・配布・インストール確認 | 本章（07c） |

Windows 向け React Native（React Native Windows）のビルドは、Visual Studio 2022 および Windows App SDK がインストール済みの Windows 開発環境上で実行する。

---

**本節で確定した方針**
- 本章は Windows 固有手順の唯一の権威文書として管理することを確定する。
- 3 OS 共通事項は統合書（07 章）を参照することを確定する。
- 本章の手順完了をもって Windows 端末への配布完了と判定することを確定する。

---

## 2 React Native Windows のビルド

### 2-1. ビルドコマンドの実行

**MIG-X-171**: Windows 本番ビルドは以下のコマンドで実行する。実行前に Visual Studio 2022・Windows App SDK・Node.js の各バージョンが `versions.lock` に記録された要件を満たすことを確認する。

```powershell
# React Native Windows リリースビルドの実行
npx react-native run-windows --release --arch x64

# または、プロジェクト固有のビルドスクリプトを使用する場合
npm run build:windows:release
```

ビルドコマンドの実行オプションを以下に定める。

| オプション | 説明 |
|---|---|
| `--release` | リリースビルド（デバッグ情報なし・最適化済み）の指定 |
| `--arch x64` | x64 アーキテクチャ（64bit Windows）のビルド指定 |

ビルドログは `windows\build_release.log` に出力する。エラーが発生した場合はビルドログを確認し、依存関係の不足・パス設定の誤り・証明書設定の問題がないか順に確認する。

### 2-2. MSIX パッケージの生成確認

**MIG-X-172**: ビルド完了後、以下の手順で MSIX パッケージの生成を確認する。

```powershell
# MSIX パッケージの生成（Visual Studio の MSBuild を使用）
msbuild windows\<AppName>.sln /p:Configuration=Release /p:Platform=x64 /p:AppxPackageDir="output\msix\" /t:Publish

# 生成された MSIX ファイルの確認
dir output\msix\
```

MSIX パッケージの生成確認事項を以下に定める。

| 確認事項 | 確認方法 |
|---|---|
| `output\msix\` ディレクトリに `.msix` または `.msixbundle` ファイルが生成されていること | `dir output\msix\` で確認 |
| パッケージ名・バージョン（`1.0.0`）が `Package.appxmanifest` の定義と一致すること | パッケージ詳細プロパティで確認 |
| ファイルサイズが異常に小さくないこと（通常 50 MB 以上） | ファイルサイズを確認 |

生成された MSIX ファイルは `releases/windows/v1.0.0/` ディレクトリにコピーし、SHA256 ハッシュ値を記録する。

---

**本節で確定した方針**
- `npx react-native run-windows --release --arch x64` を Windows 本番ビルドの標準コマンドとすることを確定する。
- MSIX パッケージのバージョン・ファイルサイズの確認をビルド完了条件とすることを確定する。
- ビルドアーティファクトの SHA256 ハッシュ値を記録することを確定する。

---

## 3 コード署名

### 3-1. 証明書の準備と署名の実施

**MIG-X-173**: MSIX パッケージへのコード署名は、以下のいずれかの証明書を使用して実施する。Windows デバイスへのインストール時に「信頼された署名者」として認識されるために署名は必須である。

**自己署名証明書による署名（開発・社内配布向け）:**

```powershell
# 自己署名証明書の生成
$cert = New-SelfSignedCertificate -Type Custom -Subject "CN=<AppPublisher>" `
  -KeyUsage DigitalSignature -FriendlyName "<AppName>" `
  -CertStoreLocation "Cert:\CurrentUser\My"

# 証明書のエクスポート（PFX 形式）
Export-PfxCertificate -cert $cert -FilePath "<AppName>.pfx" -Password (ConvertTo-SecureString -String "<password>" -Force -AsPlainText)

# MSIX への署名
signtool sign /fd sha256 /a /f "<AppName>.pfx" /p "<password>" "output\msix\<AppName>.msix"
```

**社内 CA 証明書による署名（組織 PKI 導入済みの場合）:**

```powershell
# 社内 CA から発行されたコード署名証明書を使用
signtool sign /fd sha256 /n "<発行された証明書の Subject CN>" "output\msix\<AppName>.msix"
```

署名後の確認コマンドを以下に定める。

```powershell
# 署名の確認
signtool verify /pa "output\msix\<AppName>.msix"
```

Windows デバイスへの自己署名証明書のインストール手順を以下に定める。

| ステップ | 手順 | 対象デバイス |
|---|---|---|
| 1 | `.pfx` ファイルをデバイスの任意のフォルダにコピーする | 全 Windows デバイス |
| 2 | `.pfx` ファイルをダブルクリックし「証明書のインポート」を起動する | 全 Windows デバイス |
| 3 | 「ローカルコンピューター」を選択し「信頼されたルート証明機関」ストアにインポートする | 全 Windows デバイス |
| 4 | インポート完了後、証明書管理コンソール（`certlm.msc`）で証明書が存在することを確認する | 全 Windows デバイス |

---

**本節で確定した方針**
- MSIX パッケージへのコード署名を Windows デバイスへのインストールの必須前提とすることを確定する。
- 自己署名証明書を使用する場合は、インストール対象の全 Windows デバイスの「信頼されたルート証明機関」ストアに事前インポートすることを確定する。
- 署名完了は `signtool verify /pa` による確認をもって判定することを確定する。

---

## 4 社内配布と Windows デバイスへのインストール

### 4-1. ファイルサーバー共有または USB による配布

**MIG-X-174**: 署名済み MSIX パッケージの社内配布は以下の方法で実施する。

**ファイルサーバー共有による配布:**

| ステップ | 手順 | 担当者 |
|---|---|---|
| 1 | 社内ファイルサーバーの配布用共有フォルダに MSIX ファイルをコピーする | system_admin |
| 2 | 共有フォルダのアクセス権限を確認する（工場内 LAN のみアクセス可能） | system_admin |
| 3 | 共有パス（例: `\\server\apps\worknavigation\v1.0.0\`）を全 Windows デバイス担当者に通知する | system_admin |
| 4 | 各 Windows デバイスから共有パスにアクセスし MSIX ファイルをコピーする | 各デバイス担当者 |

**USB による配布（ネットワーク非接続デバイス向け）:**

| ステップ | 手順 | 担当者 |
|---|---|---|
| 1 | USB ストレージデバイスに MSIX ファイルと自己署名証明書ファイルをコピーする | system_admin |
| 2 | 対象 Windows デバイスに USB を接続し、ファイルをローカルにコピーする | system_admin |
| 3 | 証明書のインストール（MIG-X-173 参照）を先に実施する | system_admin |

### 4-2. インストール完了確認と本番 URL 接続確認

**MIG-X-175**: 全 Windows デバイスへのインストール完了後、以下のチェックリストで確認を実施する。全項目の確認完了をもって Windows 配布完了と判定する。

| チェック番号 | 確認項目 | 確認方法 | 担当者 |
|---|---|---|---|
| MIG-CK-Win-01 | ハンディ APP がデバイスにインストール済みであること | スタートメニューでアプリ確認 | system_admin |
| MIG-CK-Win-02 | アプリバージョンが `1.0.0` であること | アプリの「設定」→「バージョン情報」で確認 | system_admin |
| MIG-CK-Win-03 | 本番サーバー URL が正しく設定されていること | アプリの設定画面で URL を確認 | system_admin |
| MIG-CK-Win-04 | 本番 API エンドポイントへの接続が成功すること | アプリ内「接続確認」機能を実行 | system_admin |
| MIG-CK-Win-05 | テストアカウントでのログインが成功すること | worker_test アカウントでログイン実行 | system_admin |

疎通確認の結果は統合書 §5 のデバイス登録チェックリストに記録する。全 Windows デバイスで MIG-CK-Win-01〜05 の全項目が合格になった時点で Windows 配布手順を完了する。

---

**本節で確定した方針**
- MIG-CK-Win-01〜05 の全確認を Windows 配布完了条件とすることを確定する。
- 疎通確認結果の統合書 §5 チェックリストへの記録を必須とすることを確定する。
- 全デバイス確認完了をもって Windows 固有手順の完了と判定することを確定する。

---

### 参照業界分析

#### 必須

- [07_スマートファクトリーと作業のデジタル化.md](../../90_業界分析/07_スマートファクトリーと作業のデジタル化.md) — Windows ハンディ端末による製造現場デジタル化の根拠

#### 関連

- [06_品質管理とトレーサビリティ.md](../../90_業界分析/06_品質管理とトレーサビリティ.md) — Windows デバイス配布後の証跡整合性確保の根拠
- [19_電子チェックリストと手順遵守の科学.md](../../90_業界分析/19_電子チェックリストと手順遵守の科学.md) — Windows デバイス確認チェックリスト設計の根拠
- [23_作業訓練設計とインストラクショナルデザイン.md](../../90_業界分析/23_作業訓練設計とインストラクショナルデザイン.md) — Windows デバイス操作訓練と配布手順の設計根拠

---

| バージョン | 日付 | 変更内容 | 作成者 |
|---|---|---|---|
| 0.1.0 | 2026-05-18 | 初版 | RyuheiKiso |
