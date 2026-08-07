# ProjectLens

ソースコードプロジェクト（スクラッチ開発）や RPA 定義ファイル（Power Platform / Power Automate for Desktop / UiPath）を解析し、社内AI **NewtonX** による生成AI解析を用いて、以下3種のドキュメントを自動生成するデスクトップアプリです。

1. システム仕様書（ユーザー/開発者向け）
2. システム基本設計書（開発者向け）
3. 詳細設計書（ファイル/コンポーネント単位、開発者向け）

対象OSはWindowsのみ、生成ドキュメント・UIとも日本語のみです。詳細は [`CLAUDE.md`](./CLAUDE.md) と [`docs/`](./docs) を参照してください（実装前に必読）。

## 技術スタック

| レイヤ | 技術 |
|---|---|
| デスクトップ基盤 | Tauri v2 |
| バックエンド | Rust（tokio / serde / thiserror / rusqlite） |
| フロントエンド | React + TypeScript（Vite） |
| 状態管理 / i18n | Zustand / react-i18next |
| UI | Tailwind CSS + shadcn/ui |
| 静的解析 | tree-sitter |
| AI | NewtonX（Python ADK をサイドカー経由で利用） |
| DB | SQLite |

## 開発環境構築

### 前提ツール

| ツール | 用途 | 確認コマンド |
|---|---|---|
| Rust（stable, `x86_64-pc-windows-msvc`） | Tauriバックエンド | `rustc --version` |
| MSVC Build Tools（C++ワークロード） | Rustのリンカ（`link.exe`） | `where link.exe`（Visual Studio由来のものが必要） |
| Node.js 20+ / npm | フロントエンドビルド | `node --version` |
| Python 3.11+ | NewtonXサイドカー（`newtonx_bridge.py`）の開発・PyInstaller化 | `python --version` |
| WebView2 Runtime | TauriのWebView（Windows 11は標準搭載） | - |

未インストールの場合:

```powershell
# Rust
winget install --id Rustlang.Rustup -e

# MSVC Build Tools（C++ワークロード）
winget install --id Microsoft.VisualStudio.2022.BuildTools -e `
  --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

### 起動手順（初回セットアップ〜検証）

#### 1. フロントエンド依存のインストール

```bash
npm install
```

#### 2. NewtonX ADKを「アプリが実際に呼び出すPython」にインストール

アプリ本体（`src-tauri/src/state.rs` の `python_exe`）は、PATH上の **`python` コマンドをそのまま子プロセスとして起動**し、`src-tauri/sidecar/newtonx_bridge.py` を実行します（venvは自動で使われません）。そのため、`python` が指す環境に直接ADKをインストールしてください。

```powershell
python -m pip install newtonx_adk/newtonx_adk-0.10.5-py3-none-any.whl
python -c "import newtonx_adk; print('OK')"   # インポート確認
```

※ 別のPythonを使いたい場合（pyenv/複数バージョン共存等）は `AppState.python_exe`（`src-tauri/src/state.rs`）を絶対パスに変更してください（現状はconfig.json等からの外出しは未実装で、コード直書きです）。

`newtonx_adk/tools/setup_config.py` / `check_config.py`（`myvenv/` にセットアップ済み）は、ADK単体の疎通確認用の参考ツールです。アプリ本体の認証設定とは別物（別の設定ファイル）なので、アプリの動作検証には必須ではありません。

#### 3. アプリを起動する

```bash
npm run tauri dev
```

初回はRustのビルドが走るため数十秒〜数分かかります。以降は増分ビルドで高速化されます。

> **⚠️ 起動後に開くProjectLensの専用ウィンドウを操作してください。**
> ターミナルに `Local: http://localhost:1420/` というURLが表示されますが、これは内部のViteサーバであり、
> **通常のブラウザ（Chrome/Edge等）で直接開かないでください**。ブラウザで開くとTauriのIPC機構
> （`window.__TAURI_INTERNALS__`）が存在せず、ボタン操作時に `Cannot read properties of undefined (reading 'invoke')`
> というエラーになります（`npm run dev` だけを実行した場合も同様）。必ず `npm run tauri dev` が自動で開く
> ネイティブウィンドウ側を使用してください。なお、誤ってブラウザで開いた場合はアプリ側でも検知し、
> 案内メッセージを表示するようにしています。

#### 4. アプリ内でNewtonX認証を行う

1. ホーム画面 →「NewtonX認証」を開く
2. **Host**: 会社サブドメイン（例: `seraku`）を入力
3. **Personal Access Token**: NewtonX Web版のアカウントメニュー「アクセストークン」から発行したPATを貼り付け
4. 「PATを保存」→「接続テスト」で疎通確認（PATはOS資格情報ストアに保存され、config.jsonには平文保存されません。詳細: [`docs/04_実装詳細.md`](./docs/04_実装詳細.md) §3.4）

#### 5. 解析を実行する

1. ホーム画面 →「プロジェクトフォルダを選択」でソースコード/RPA定義ファイルを含むフォルダを指定
2. 解析画面で4フェーズ（スキャン→静的解析→AI解析→ドキュメント生成）の進捗を確認
3. 完了後、結果画面の3タブ（システム仕様書 / システム基本設計書 / 詳細設計書）を確認
4. エクスポート画面から成果物（MD/HTML/JSON）を出力し、`projectlens-docs/`（既定）配下を確認

### 既知の未検証事項（このセッションでは確認できなかった点）

- **GUIの目視確認**: 実装時の環境にインタラクティブなディスプレイがなく、`npm run tauri dev` はビルド・起動まで確認できたが画面描画は未確認
- **RPA解析の精度**: Power Platform / PAD / UiPathの実サンプルファイルが手元になく、`RpaAnalyzer`の検出・抽出ロジックは未検証（`newtonx_adk`のようなサンプルを`docs/04_実装詳細.md` §10 #6の方針で用意すると検証しやすい）
- **NewtonX実接続**: 実PATでの認証・解析・レート制限挙動は未確認

## ディレクトリ構成

```
├── CLAUDE.md            # 開発ガイド（必読）
├── docs/                # 要件定義書/仕様書/設計書/実装詳細書
├── newtonx_adk/          # NewtonX ADK 参考資料・サンプル・配布パッケージ
├── src/                  # フロントエンド（React + TypeScript）
├── src-tauri/            # バックエンド（Rust）
└── package.json
```

## ドキュメント優先順位

`04_実装詳細.md` > `03_設計書.md` > `02_仕様書.md` > `01_要件定義書.md`（矛盾時は番号の大きい方を優先）
