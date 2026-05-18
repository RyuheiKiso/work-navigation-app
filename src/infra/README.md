# src/infra — インフラ設定・構成管理

本ディレクトリは WNAV システムのインフラ層（デプロイ設定・DB マイグレーション・各種設定ファイル）を管理する。

---

## ディレクトリ構成

```
src/infra/
  config/           # 接続先情報・非機密設定の YAML 一元管理（SSoT）
  database/         # DB マイグレーションファイル（sqlx migrate）
```

---

## config/

バックエンド・インフラ・フロントエンド配信設定の単一情報源。  
`WNAV_PROFILE={local,dev,staging,prod}` で環境を切り替え、`config.base.yml` + `config.{profile}.yml` をマージして読み込む。  
機密はすべて `secret_ref: "<scheme>:<id>"` で間接参照し、YAML ファイルには機密を直書きしない。

詳細は `config/README.md` を参照。

---

## database/

`sqlx migrate` 形式のマイグレーションファイルを管理する（未実装・今後追加予定）。

マイグレーション手順は `docs/08_移行/導入手順/03_PostgreSQL初期化とマイグレーション手順.md` を参照。
