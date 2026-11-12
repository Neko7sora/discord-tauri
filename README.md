# discord-tauri

Tauri 2 + TypeScript + Vite で構築した Discord 向けデスクトップブラウザです。
現在は `https://discord.com/login` のみをアプリ内 WebView で開き、それ以外の `http/https` 遷移や新規ウィンドウ要求は既定の外部ブラウザへ委譲する構成になっています。

## 現在の挙動

- 起動時に `discord.com/login` をメインウィンドウで開く
- ウィンドウタイトルをページタイトルに追従させる
- `https://discord.com` だけを WebView 内で許可する
- `discord.com` 以外への `http/https` 通常遷移を外部ブラウザで開く
- `window.open` などの別窓要求で `discord.com` 以外へ出る場合も外部ブラウザで開く
- 非 `http/https` スキームの遷移は WebView 内で拒否する
- Tauri の capability は `main` ウィンドウにだけ割り当て、IPC 権限は付与しない

## 技術スタック

- Tauri 2
- TypeScript 6
- Vite 8
- Rust 2021
- npm + Volta

フロントエンドは軽量化のため、React を使わず最小の Vite + TypeScript 構成にしています。

## 必要環境

Windows と macOS（Apple Silicon / Intel）に対応しています。

- Node.js `24.21.0`
- npm `12.2.0`
- Rust stable

Windows では、追加で次の環境が必要です。

- WebView2 Runtime
- Microsoft C++ Build Tools

macOS では、Xcode または Xcode Command Line Tools が必要です。Apple Silicon では、Node.js と Rust も arm64 版を使用してください。

```sh
xcode-select --install
```

詳しくは [Tauri の必要環境](https://v2.tauri.app/start/prerequisites/)を参照してください。

`package.json` に Volta 設定があるため、Volta を使うと Node.js / npm の版を合わせやすくなります。

## セットアップ

```powershell
volta install node@24.21.0
volta install npm@12.2.0
rustup update stable
npm install
```

バージョン確認:

```powershell
node --version
npm --version
rustc --version
npm exec tauri --version
```

## 開発

フロントエンドのみ起動:

```powershell
npm run dev
```

Tauri アプリとして起動:

```powershell
npm run tauri:dev
```

## ビルド

```powershell
npm run build
npm run tauri:build
```

`npm run build` と `npm run typecheck` は `tsc -b` を使うため、2 系統の TypeScript 設定をインクリメンタルにまとめて検査します。

主な生成先:

- `dist/`
- `src-tauri/target/release/`
- `src-tauri/target/release/bundle/`

### Apple Silicon 向けビルド

macOS 上で次を実行します。

```sh
rustup target add aarch64-apple-darwin
npm ci
npm run tauri:build:macos:arm64
```

生成先:

- アプリ: `src-tauri/target/aarch64-apple-darwin/release/bundle/macos/discord-tauri.app`
- インストーラー: `src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/`

Apple Silicon では Rosetta を使わずネイティブで動作するビルドになります。開発起動は通常どおり `npm run tauri:dev` を使用します。

macOS のバンドルには [Tauri の ad-hoc 署名](https://v2.tauri.app/distribute/sign/macos/#ad-hoc-signing)を設定しています。Apple による公証は行わないため、ダウンロードしたアプリの初回起動時に macOS の「プライバシーとセキュリティ」で実行の許可が必要になる場合があります。

## 利用できるスクリプト

- `npm run dev`: Vite 開発サーバー
- `npm run build`: TypeScript チェック + Vite 本番ビルド
- `npm run lint`: ESLint
- `npm run typecheck`: TypeScript 型チェック
- `npm run format`: Prettier チェック
- `npm run format:write`: Prettier 整形
- `npm run tauri`: Tauri CLI
- `npm run tauri:dev`: Tauri 開発起動
- `npm run tauri:build`: Tauri 本番ビルド
- `npm run tauri:build:macos:arm64`: macOS Apple Silicon 向け本番ビルド（macOS 上で実行）

## リリース

GitHub Actions の [`.github/workflows/tauri-action.yml`](./.github/workflows/tauri-action.yml) で、`release` ブランチへの push または手動実行時にリリース用ビルドを作成します。

- macOS Apple Silicon（`aarch64-apple-darwin`、`aarch64.dmg`）
- macOS Intel（`x86_64-apple-darwin`、`x64.dmg`）
- Ubuntu 24.04
- Windows

生成された成果物は GitHub Release の draft に添付されます。

## 関連ファイル

- [`src-tauri/src/main.rs`](./src-tauri/src/main.rs): WebView の起動とナビゲーション制御
- [`src-tauri/capabilities/main-window.json`](./src-tauri/capabilities/main-window.json): `main` ウィンドウ向けの最小 capability
- [`src-tauri/tauri.conf.json`](./src-tauri/tauri.conf.json): Tauri 設定
- [`package.json`](./package.json): Node.js / npm / スクリプト定義
