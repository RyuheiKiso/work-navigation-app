# 07b iOS 配布手順（App Store / EAS）

本章の責務は、`07_ハンディAPP配布手順（統合）.md` の iOS 固有サブ手順として、Apple 開発者アカウントと証明書の確認・EAS Build による iOS ビルドの実行・TestFlight または MDM による配布実施・iOS デバイスへのインストール確認を確定することである。

---

## 1 本章の責務（07_統合書の iOS 固有サブ）

### 1-1. 07_統合書との関係

本章は `07_ハンディAPP配布手順（統合）.md` の iOS 専用サブ手順書である。本章で扱う手順は iOS OS 固有の操作に限定し、3 OS 共通の手順は統合書（07 章）を参照する。

| 担当範囲 | 文書 |
|---|---|
| 3 OS 共通フロー・EAS 設定・判定表 | `07_ハンディAPP配布手順（統合）.md` |
| iOS 固有証明書確認・ビルド・配布・インストール確認 | 本章（07b） |

iOS の配布は Apple の Developer Program への有効なメンバーシップを前提とする。`99_前提制約.md` の CON-MIG-X-014 を参照すること。

---

**本節で確定した方針**
- 本章は iOS 固有手順の唯一の権威文書として管理することを確定する。
- 3 OS 共通事項は統合書（07 章）を参照することを確定する。
- 本章の手順完了をもって iOS 端末への配布完了と判定することを確定する。

---

## 2 Apple 開発者アカウントと証明書の確認

### 2-1. Distribution 証明書・Provisioning Profile の有効性確認

**MIG-X-166**: EAS Build の実行前に、Apple Developer Program の以下の要素が有効であることを Apple Developer Console（https://developer.apple.com）で確認する。

| 確認項目 | 確認内容 | 確認場所 |
|---|---|---|
| Apple Developer Program メンバーシップ | 有効期限内であること（残余 30 日以上を確認する） | Membership セクション |
| Distribution 証明書 | 有効期限内であること・ `iOS Distribution` タイプであること | Certificates セクション |
| App ID | バンドル ID（例: com.example.worknavigation）が登録済みであること | Identifiers セクション |
| Provisioning Profile | `App Store` タイプで有効期限内であること・対象 App ID と証明書に紐付いていること | Profiles セクション |

Distribution 証明書の有効期限が 30 日以内の場合は事前に更新する。EAS は証明書を自動管理（`credentials` コマンド）するが、有効期限の確認は手動で実施する。

証明書の手動確認コマンドを以下に定める。

```bash
# EAS 管理の証明書一覧確認
eas credentials --platform ios
```

---

**本節で確定した方針**
- Distribution 証明書および Provisioning Profile の有効性確認を EAS Build 実行前の必須条件とすることを確定する。
- 有効期限 30 日以内の証明書は事前更新に対応することを確定する。
- `eas credentials --platform ios` による証明書状態確認を実施することを確定する。

---

## 3 EAS Build の実行

### 3-1. `eas build --platform ios --profile production` の実行

**MIG-X-167**: iOS 本番ビルドは以下のコマンドで実行する。実行前に統合書 §2 で確認した `eas.json` の `production` プロファイルが有効であることを確認する。

```bash
# EAS Build の実行（iOS production ビルド）
eas build --platform ios --profile production

# ビルド進捗確認（Expo ダッシュボードの URL が出力される）
# 例: https://expo.dev/accounts/<account>/projects/<project>/builds/<build-id>
```

ビルド実行後、Expo ダッシュボードでビルドのステータスが `FINISHED` になるまで待機する。iOS ビルドは macOS ビルドサーバー上で実行されるため、通常 15〜40 分を要する。

### 3-2. ビルド完了確認と IPA ファイルのダウンロード

**MIG-X-168**: EAS Build 完了後、以下の確認とダウンロードを実施する。

| 確認事項 | 確認方法 |
|---|---|
| ビルドステータスが `FINISHED` であること | Expo ダッシュボードで確認 |
| ビルドアーティファクトが `.ipa` ファイルであること | Expo ダッシュボードのアーティファクト種別で確認 |
| アプリバージョンが `1.0.0`、buildNumber が `1.0.0` であること | ビルド詳細画面で確認 |
| Distribution 証明書による署名済みであること | ビルド詳細画面の「Signed」表示で確認 |

ダウンロード手順を以下に定める。

```bash
# EAS CLI によるダウンロード
eas build:download --id <build-id>

# または Expo ダッシュボードからブラウザで直接ダウンロード
```

ダウンロードした `.ipa` ファイルは `releases/ios/v1.0.0/` ディレクトリに保存し、ファイルのハッシュ値（SHA256）を記録する。

---

**本節で確定した方針**
- `eas build --platform ios --profile production` を iOS 本番ビルドの唯一の実行コマンドとすることを確定する。
- ビルドアーティファクトの SHA256 ハッシュ値を記録することを確定する。
- ビルドステータス `FINISHED` の確認をダウンロード開始条件とすることを確定する。

---

## 4 配布方法の選択と実施

### 4-1. TestFlight（社内テスト配布）の手順

**MIG-X-169**: TestFlight を使用した社内テスト配布の場合、以下の手順を App Store Connect（https://appstoreconnect.apple.com）で実施する。

| ステップ | 手順 | 担当者 |
|---|---|---|
| 1 | App Store Connect にログインし、対象アプリを選択する | system_admin |
| 2 | 「TestFlight」タブを選択する | system_admin |
| 3 | 「新しいビルドをアップロード」または `altool` / `Transporter` でビルドをアップロードする | system_admin |
| 4 | アップロード完了後、ビルドの処理完了（通常 5〜30 分）を待つ | system_admin |
| 5 | 内部テストグループ（社内テスター）を選択し、ビルドを追加する | system_admin |
| 6 | テスター全員にテスト招待メールが送信されることを確認する | system_admin |
| 7 | テスターは TestFlight アプリからビルドをインストールする | 各 iOS デバイス担当者 |

TestFlight アップロード後、Appleによる簡易審査（ベータ審査）が実施される。承認まで通常 24 時間以内を要する。

MDM による社内配布（組織管理デバイス向け）の手順を以下に定める。組織が Apple Business Manager を導入済みで、MDM サーバー（例: Jamf, Intune）を運用している場合は MDM 配布を選択する。

| ステップ | 手順 | 担当者 |
|---|---|---|
| 1 | MDM サーバーの管理コンソールにログインする | system_admin |
| 2 | `.ipa` ファイルをアプリカタログにアップロードする | system_admin |
| 3 | 対象デバイスグループを選択し、アプリの配布ポリシーを設定する | system_admin |
| 4 | デバイスへのプッシュ配布を実行する | system_admin |
| 5 | 各デバイスでのインストール完了を MDM コンソールで確認する | system_admin |

---

**本節で確定した方針**
- TestFlight 配布は Apple ベータ審査（最大 24 時間）のスケジュール余裕を確保した上で実施することを確定する。
- MDM 配布は Apple Business Manager 導入済みの組織のみを対象とすることを確定する。
- 配布方法は統合書 §3 の判定表に基づいて選択することを確定する。

---

## 5 iOS デバイスへのインストール確認

### 5-1. インストール完了確認と本番 URL 接続確認

**MIG-X-170**: 全 iOS デバイスへのインストール完了後、以下のチェックリストで確認を実施する。全項目の確認完了をもって iOS 配布完了と判定する。

| チェック番号 | 確認項目 | 確認方法 | 担当者 |
|---|---|---|---|
| MIG-CK-iOS-01 | ハンディ APP がデバイスにインストール済みであること | ホーム画面でアプリアイコン確認 | system_admin |
| MIG-CK-iOS-02 | アプリバージョンが `1.0.0` であること | アプリの「設定」→「バージョン情報」で確認 | system_admin |
| MIG-CK-iOS-03 | 本番サーバー URL が正しく設定されていること | アプリの設定画面で URL を確認 | system_admin |
| MIG-CK-iOS-04 | 本番 API エンドポイントへの接続が成功すること | アプリ内「接続確認」機能を実行 | system_admin |
| MIG-CK-iOS-05 | テストアカウントでのログインが成功すること | worker_test アカウントでログイン実行 | system_admin |

疎通確認の結果は統合書 §5 のデバイス登録チェックリストに記録する。全 iOS デバイスで MIG-CK-iOS-01〜05 の全項目が合格になった時点で iOS 配布手順を完了する。

---

**本節で確定した方針**
- MIG-CK-iOS-01〜05 の全確認を iOS 配布完了条件とすることを確定する。
- 疎通確認結果の統合書 §5 チェックリストへの記録を必須とすることを確定する。
- 全デバイス確認完了をもって iOS 固有手順の完了と判定することを確定する。

---

### 参照業界分析

#### 必須

- [07_スマートファクトリーと作業のデジタル化.md](../../90_業界分析/07_スマートファクトリーと作業のデジタル化.md) — iOS ハンディ端末による製造現場デジタル化の根拠

#### 関連

- [06_品質管理とトレーサビリティ.md](../../90_業界分析/06_品質管理とトレーサビリティ.md) — iOS デバイス配布後の証跡整合性確保の根拠
- [19_電子チェックリストと手順遵守の科学.md](../../90_業界分析/19_電子チェックリストと手順遵守の科学.md) — iOS デバイス確認チェックリスト設計の根拠
- [22_規制別トレーサビリティ要件詳論.md](../../90_業界分析/22_規制別トレーサビリティ要件詳論.md) — Apple 開発者アカウント管理と規制要件の根拠

---

| バージョン | 日付 | 変更内容 | 作成者 |
|---|---|---|---|
| 0.1.0 | 2026-05-18 | 初版 | RyuheiKiso |
