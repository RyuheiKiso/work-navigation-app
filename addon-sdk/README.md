# addon-sdk

> 対応 §: ロードマップ §17 §17.2 §17.3 §17.4 §17.5

work-navigation-app のアドオン SDK。WASM／Wasmtime 上で動作する拡張機能を、Rust（一次サポート）と AssemblyScript（二次サポート）で開発できる。

## ディレクトリ

```
addon-sdk/
├── README.md
├── rust/                  # Rust 一次 SDK（§17.2）
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── api.rs         # API surface v1（§17.3 11 領域）
├── assemblyscript/        # AssemblyScript 二次 SDK
│   ├── package.json
│   ├── asconfig.json
│   └── assembly/
│       └── index.ts
└── examples/
    └── hello-step/         # サンプル（§17.7 受入観点）
```

## API surface v1（§17.3 再掲）

| 領域 | 提供 API（抜粋） | 必須 capability |
| --- | --- | --- |
| 作業情報読取 | `getCurrentTask()`／`listSteps()`／`getContext()` | `task.read` |
| 作業実績書込 | `appendRecord()`／`markStep(id, evidence)` | `task.write` |
| メディア | `attachMedia(taskId, mediaRef)`／`readMedia(ref)` | `media.write`／`media.read` |
| HTTP アウトバウンド | `fetch(url, options)` ホワイトリスト URL のみ | `net.outbound:<host>` |
| ストレージ KV | `kv.get(key)`／`kv.set(key, value)` 名前空間別 | `storage:<namespace>` |
| 通知 | `notify(channel, message)` | `notify:<channel>` |
| UI 拡張 | `registerPanel(slot, component)` | `ui.extend:<slot>` |
| ロギング | `log(level, message)` | 既定許可 |
| 設定 | `getConfig(key)` 公開設定のみ | `config.read` |
| 暗号 | `crypto.sign(payload)`／`crypto.verify(...)` | `crypto.sign` |
| 時刻 | `now()`（サーバ時刻同期済み） | 既定許可 |

破壊的変更は MAJOR バンプ＋ 6 ヶ月の移行猶予（§17.3 ／§19.3）。

## 配布形式

`*.wnaddon`（zip 形式）。マニフェスト＋WASM＋アセット（§17.6）。
リリース署名（OIDC）必須（§19.3）。
