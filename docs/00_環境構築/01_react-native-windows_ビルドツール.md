# react-native-windows ビルドツール導入手順

`react-native-windows` (RNW) のネイティブビルドに必要な MSBuild と VCTools を、**Visual Studio 2022 IDE をインストールせずに** 「Build Tools for Visual Studio 2022」スタンドアロン版で揃える手順をまとめる。

## 1. 目的と背景

`npx react-native run-windows` は内部で `vswhere.exe` を起動し、以下を満たす Visual Studio インストールを探す。

- バージョン 17.11.0 以上 (= VS 2022 17.11+)
- ワークロード／コンポーネント:
  - `Microsoft.Component.MSBuild`
  - `Microsoft.VisualStudio.Component.VC.Tools.x86.x64`
  - UWP C++ v143 ツール
  - Windows 11 SDK (10.0.22621.0)

該当環境がない場合、次のエラーで停止する。

```
× Could not find MSBuild with VCTools for Visual Studio 17.11.0 or later.
  Make sure all required components have been installed
```

公式ドキュメントは Visual Studio 2022 Community/Pro/Enterprise の IDE 導入を推奨するが、**ビルド要件は BuildTools SKU でもすべて満たせる**。IDE を入れない分、ディスク使用量・インストール時間ともに大幅に削減できる。

## 2. 前提条件

| 項目 | 要件 |
|---|---|
| OS | Windows 10 21H2 以降 / Windows 11 (CLAUDE.md に準ずる) |
| パッケージマネージャ | winget (Windows 11 標準搭載、Windows 10 は App Installer から導入) |
| 権限 | 管理者権限の PowerShell |
| ディスク空き | 10 GB 以上を推奨 |
| ネットワーク | Microsoft 配布サーバへの HTTPS 接続 |

`winget` の存在は以下で確認する。

```powershell
winget --version
```

## 3. インストールコマンド

管理者権限の PowerShell で **1 コマンド** を実行する。改行せず 1 行で投入すること。

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--quiet --wait --norestart --add Microsoft.VisualStudio.Workload.VCTools --add Microsoft.VisualStudio.Workload.MSBuildTools --add Microsoft.VisualStudio.Workload.UniversalBuildTools --add Microsoft.VisualStudio.Workload.ManagedDesktopBuildTools --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.Windows11SDK.22621 --add Microsoft.VisualStudio.ComponentGroup.UWP.VC.BuildTools --includeRecommended"
```

`--override` 文字列は `vs_buildtools.exe` にそのまま渡される。`--quiet` で UI を抑止し、`--wait` でインストール完了まで winget をブロックする。

### 3.1 投入される workloads / components の意図

| ID | 種別 | 必要性 |
|---|---|---|
| `Microsoft.VisualStudio.Workload.MSBuildTools` | Workload | RNW CLI が呼び出す MSBuild 本体 |
| `Microsoft.VisualStudio.Workload.VCTools` | Workload | C++ ネイティブビルド (MSVC v143 ツールチェーン) |
| `Microsoft.VisualStudio.Workload.UniversalBuildTools` | Workload | UWP 系プロジェクト (RNW の Windows ターゲット) |
| `Microsoft.VisualStudio.Workload.ManagedDesktopBuildTools` | Workload | .NET デスクトップ系のビルド (RNW の C# 部分) |
| `Microsoft.VisualStudio.Component.VC.Tools.x86.x64` | Component | x86/x64 向け MSVC コンパイラ |
| `Microsoft.VisualStudio.Component.Windows11SDK.22621` | Component | Win11 SDK 10.0.22621.0 (RNW 公式要件) |
| `Microsoft.VisualStudio.ComponentGroup.UWP.VC.BuildTools` | ComponentGroup | UWP の C++ ビルドサポート |
| `--includeRecommended` | フラグ | 各 workload の「推奨コンポーネント」も同時導入 |

ARM64 端末向けにビルドする場合は、追加で次を含める。

```text
--add Microsoft.VisualStudio.Component.VC.Tools.ARM64
```

## 4. インストール後の検証

インストール完了後、新しい PowerShell セッションを開き以下を実行する。

```powershell
& "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" `
  -products * `
  -requires Microsoft.Component.MSBuild Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
  -version "[17.11,18.0)" `
  -property installationPath
```

`C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools` のようなパスが返れば成功。空応答の場合は、ワークロードの追加が不完全か、17.11 未満のバージョンが入っている可能性が高い。

MSBuild 単体の所在確認も合わせて実施する。

```powershell
& "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" `
  -products * -find MSBuild\**\Bin\MSBuild.exe
```

## 5. RNW CLI のパッチ適用

VS 2022 BuildTools 単体で `react-native-windows` v0.82 系を動かすには、CLI に **3 か所** のパッチが必要となる。いずれも上流に未修正のバグであり、本プロジェクトでは [`patch-package`](https://www.npmjs.com/package/patch-package) で `patches/@react-native-windows+cli+<version>.patch` として保持している。`npm install` 後の `postinstall` フックで自動適用されるため、開発者側で追加作業は不要である。

### 5.1 パッチの目的 (3 か所)

| # | 修正対象 | 症状 | 原因 |
|---|---|---|---|
| 1 | `vsInstalls.js` | `Could not find MSBuild with VCTools for Visual Studio 17.11.0 or later` | RNW CLI が `vswhere` 呼び出し時に `-products *` を渡さないため、BuildTools SKU が検索対象から除外される ([#10262](https://github.com/microsoft/react-native-windows/issues/10262)) |
| 2 | `deploy.js` (DeployAppRecipe.exe 経路) | `ReflectionTypeLoadException` / `FileLoadException: NuGet.VisualStudio.Contracts, Version=17.14.3.0` | BuildTools 同梱の `DeployAppRecipe.exe.config` は `NuGet.VisualStudio.Contracts` 17.14.3.0 を要求するが、配布されている DLL は 17.14.0.0 で、`NuGet.VisualStudio.dll` 自体は欠落している (BuildTools パッケージング不整合) |
| 3 | `deploy.js` (MSBuild Deploy 経路) | `You must first build the project before deploying it [app.vcxproj] / Microsoft.ReactNative.Uwp.CppApp.targets(45,5)` | `sln` 全体に対する `MSBuild /t:deploy /p:DeployLayout=true` が vcxproj 側の Deploy ターゲットも発火させ、vcxproj が出力しない `app.Build.appxrecipe` を要求して失敗する |

修正方針:

1. `vsInstalls.js` の `enumerateVsInstalls` で `args.push('-products *')` を追加し BuildTools を検出可能にする。
2. `deploy.js` で `DeployAppRecipe.exe` 経路を強制無効化する (`useDeployAppRecipeExe = false`)。
3. `deploy.js` のフォールバック経路を MSBuild 起動ではなく **`Add-AppxPackage -Register` 直接呼び出し** に置き換える。`wapproj` が出力した `AppxManifest.xml` を Dev Mode で登録するため、追加のビルド成果物を必要としない。

### 5.2 パッチの中身

`patches/@react-native-windows+cli+<version>.patch` の核心部分:

```diff
--- a/node_modules/@react-native-windows/cli/lib-commonjs/utils/vsInstalls.js
+++ b/node_modules/@react-native-windows/cli/lib-commonjs/utils/vsInstalls.js
@@ -74,6 +74,9 @@ function enumerateVsInstalls(opts) {
     if (opts.requires) {
         args.push(`-requires ${opts.requires.join(' ')}`);
     }
+    // Patch: include BuildTools / TestAgent / TeamExplorer SKUs which vswhere
+    // excludes by default. Required to detect standalone Build Tools for VS 2022.
+    args.push('-products *');
     if (opts.latest) {
         args.push('-latest');
     }
```

```diff
--- a/node_modules/@react-native-windows/cli/lib-commonjs/utils/deploy.js
+++ b/node_modules/@react-native-windows/cli/lib-commonjs/utils/deploy.js
@@ -292,7 +292,13 @@ async function deployToDesktop(...) {
         const appxRecipe = getAppxRecipePath(options, projectName);
         const ideFolder = `${buildTools.installationPath}\\Common7\\IDE`;
         const deployAppxRecipeExePath = `${ideFolder}\\DeployAppRecipe.exe`;
-        if (vsVersion.gte(version_1.default.fromString('16.8.30906.45')) &&
+        // Patch: VS 2022 BuildTools の DeployAppRecipe.exe は NuGet.VisualStudio.Contracts の
+        // bindingRedirect が 17.14.3.0 を要求するのに、実体が 17.14.0.0 のため FileLoadException で失敗する。
+        // 常に PowerShell + Add-AppxPackage によるレジスト経路にフォールバックさせる。
+        const useDeployAppRecipeExe = false;
+        if (useDeployAppRecipeExe &&
+            vsVersion.gte(version_1.default.fromString('16.8.30906.45')) &&
             fs_1.default.existsSync(deployAppxRecipeExePath)) {
             await commandWithProgress(...);
         } else {
             await runPowerShellScriptFunction('Installing dependent framework packages', ...);
-            await build.buildSolution(buildTools, slnFile, 'Debug', options.arch, { DeployLayout: 'true' }, ...);
+            // Patch: sln 全体への MSBuild Deploy は vcxproj 側を巻き込み appxrecipe 欠如で失敗する。
+            // wapproj 出力の AppxManifest.xml を Add-AppxPackage -Register で直接登録する。
+            await commandWithProgress(newSpinner('Registering app package'),
+                `Registering ${appxManifestPath}`, powershell,
+                ['-NoProfile', '-Command',
+                 `Add-AppxPackage -Register '${appxManifestPath}' -ForceUpdateFromAnyVersion`],
+                verbose, 'DeployRecipeFailure');
         }
```

### 5.3 配線

`package.json` の `scripts` に以下を配線する (既に設定済み)。

```json
{
  "scripts": {
    "postinstall": "patch-package"
  },
  "devDependencies": {
    "patch-package": "^8.0.1"
  }
}
```

### 5.4 パッチ再生成手順 (RNW を上げたとき)

`react-native-windows` のバージョンを上げる際は、上流が修正済みでないかを最初に確認する。未修正なら以下でパッチを再生成する。

```powershell
# 1) 旧パッチを削除
Remove-Item .\patches\@react-native-windows+cli+*.patch

# 2) 新バージョンの vsInstalls.js と deploy.js に §5.2 の変更を再適用
# (node_modules/@react-native-windows/cli/lib-commonjs/utils/ 配下を編集)

# 3) パッチを再生成
npx patch-package @react-native-windows/cli
```

## 6. Windows Developer Mode の有効化

`run-windows` のデプロイ段は **Developer Mode が有効** であることを前提とする。Windows 設定 GUI の「プライバシーとセキュリティ → 開発者向け → 開発者モード」をオンにするのが最短だが、レジストリ直接設定でも代替できる。

```powershell
# 管理者権限の PowerShell で 1 度だけ実行
Set-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock" `
  -Name AllowDevelopmentWithoutDevLicense -Value 1 -Type DWord
Set-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock" `
  -Name AllowAllTrustedApps -Value 1 -Type DWord
```

`AllowAllTrustedApps` を設定していないと、RNW CLI が UAC 昇格を要求するスクリプトを実行しようとし、非対話的なシェル (CI 等) では `The operation was canceled by the user` で失敗する。

設定確認:

```powershell
$k = 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock'
Get-ItemProperty -Path $k -Name AllowDevelopmentWithoutDevLicense, AllowAllTrustedApps
```

両方の値が `1` になっていれば良い。

## 7. RNW プロジェクトのビルド

パッチ適用 + Developer Mode 有効化後、フロントエンドプロジェクトのルートで以下を実行する。

```powershell
cd C:\work\work-navigation-app\src\frontend\app
npx react-native run-windows
```

成功時の出力 (要所のみ):

```
✔ Building Solution
✔ Enabling Developer Mode
✔ Installing dependent framework packages
✔ Registering app package
✔ Verifying loopbackExempt
```

起動までさせたくない場合は `--no-launch`、ビルドのみで止めたい場合は `--no-launch --no-deploy` を付ける。

```powershell
npx react-native run-windows --no-launch          # ビルド + デプロイのみ
npx react-native run-windows --no-launch --no-deploy  # ビルドのみ
```

## 8. トラブルシューティング

### 8.1 `Could not find MSBuild with VCTools` がパッチ適用後も解消しない

`patches/@react-native-windows+cli+*.patch` が node_modules に適用されていない可能性がある。次で確認する。

```powershell
# パッチが当たっていれば '-products *' が表示される
Select-String -Path .\node_modules\@react-native-windows\cli\lib-commonjs\utils\vsInstalls.js -Pattern "products \*"

# 手動で再適用
npx patch-package
```

それでも解消しない場合は、`vswhere` の直接実行で installPath が返るかを確認する。

```powershell
& "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" `
  -products * `
  -requires Microsoft.Component.MSBuild Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
  -version "[17.11,18.0)" `
  -property installationPath
```

最終手段として MSBuild を直接呼ぶ運用にフォールバックできる。

```powershell
$msbuild = & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" `
  -products * -find MSBuild\**\Bin\MSBuild.exe | Select-Object -First 1
& $msbuild .\windows\app.sln /p:Configuration=Debug /p:Platform=x64 /restore
```

### 8.2 デプロイ段で `The operation was canceled by the user`

`Enabling Developer Mode` で `Start-Process -Verb RunAs` が UAC 昇格を要求し、対話不能な環境で拒否されるケース。§6 の `AllowAllTrustedApps` 設定が抜けている可能性が高い。下記で確認・設定する。

```powershell
$k = 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock'
Get-ItemProperty -Path $k -Name AllowAllTrustedApps
# 値が 1 でなければ管理者 PowerShell で:
Set-ItemProperty -Path $k -Name AllowAllTrustedApps -Value 1 -Type DWord
```

### 8.3 デプロイ段で `ReflectionTypeLoadException` / `NuGet.VisualStudio.Contracts ... 17.14.3.0`

§5 のパッチが `deploy.js` に当たっていない。次で確認する。

```powershell
# パッチが当たっていれば 'useDeployAppRecipeExe = false' が表示される
Select-String -Path .\node_modules\@react-native-windows\cli\lib-commonjs\utils\deploy.js -Pattern "useDeployAppRecipeExe"

# 手動で再適用
npx patch-package
```

### 8.4 デプロイ段で `You must first build the project before deploying it`

§5 の `deploy.js` パッチが古い (Add-AppxPackage -Register 化が抜けている) 可能性がある。§5.4 の手順で再生成する。

### 8.5 winget が `Microsoft.VisualStudio.2022.BuildTools` を見つけられない

winget ソースが古い可能性がある。次で更新する。

```powershell
winget source update
```

### 8.6 既存の BuildTools にワークロードを追加したい

既にスタンドアロン版を入れているがコンポーネントが足りない場合、`vs_installer.exe` を `modify` で呼び直す。

```powershell
& "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vs_installer.exe" `
  modify --installPath "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools" `
  --add Microsoft.VisualStudio.ComponentGroup.UWP.VC.BuildTools `
  --quiet --wait --norestart
```

### 8.7 VS 2026 / Insiders がインストール済みでも認識されない

`react-native-windows@0.82.x` 系は VS 17.11 〜 18.0 未満を対象にしているため、VS 2026 (18.x) は未サポート。BuildTools 17.x 系を並列導入するのが現状の回避策である (microsoft/react-native-windows#15399 参照)。

## 9. 参考リンク

- [Visual Studio Build Tools workload and component IDs (Microsoft Learn)](https://learn.microsoft.com/en-us/visualstudio/install/workload-component-id-vs-build-tools?view=vs-2022)
- [System Requirements — React Native for Windows](https://microsoft.github.io/react-native-windows/docs/rnw-dependencies)
- [react-native-windows / rnw-dependencies.ps1 (公式インストールスクリプト)](https://github.com/microsoft/react-native-windows/blob/main/vnext/Scripts/rnw-dependencies.ps1)
- [Issue #10262 — BuildTools 検出不具合の記録](https://github.com/microsoft/react-native-windows/issues/10262)
- [Issue #15399 — VS 2026 未対応の記録](https://github.com/microsoft/react-native-windows/issues/15399)
- [patch-package (npm)](https://www.npmjs.com/package/patch-package)
