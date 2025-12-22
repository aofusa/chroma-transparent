# chroma-transparent

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-Apache--2.0-blue)

指定した画像・動画の指定された色をクロマキー処理して透過ファイルに変換するCLIツールです。

グリーンバック画像/動画やブルーバック画像/動画など、単色背景から被写体を切り抜いて透過PNG/WebMを生成できます。

## 機能

- **クロマキー処理**: 指定した色（HEXコードまたはCSS色名）を透過に変換
- **CSS 147色対応**: `lime`, `blue`, `magenta` などの色名で指定可能
- **HSV色空間での色検出**: 照明条件の変化に強い色検出アルゴリズム
- **モルフォロジー演算**: 収縮（Erode）/膨張（Dilate）でマスクを微調整
- **フェザリング**: ガウシアンブラーでエッジを滑らかに
- **デスピル**: 被写体の縁に残った背景色の反射（色かぶり）を除去
- **柔軟なパラメータ調整**: すべての処理パラメータをコマンドラインから調整可能
- **ディレクトリ一括処理**: ディレクトリ内の全画像ファイルを一括処理
- **再帰的探索**: サブディレクトリも含めて処理（深さ指定可能）
- **動画対応** (feature: `video`): ffmpegを使用してMP4, WebM, MOV等の動画を透過動画（WebM/MOV）に変換
- **Webサーバモード** (feature: `server`): ブラウザからGUIで操作、REST API提供、OpenAPI対応

## インストール

### ソースからビルド

```bash
git clone https://github.com/aofusa/chroma-transparent.git
cd chroma-transparent
cargo build --release
```

ビルド後、実行ファイルは `target/release/chroma-transparent` に生成されます。

### 動画機能を有効にしてビルド

動画処理機能はオプションです。有効にするには `video` featureを指定してビルドします。

```bash
# 動画機能を有効にしてビルド
cargo build --release --features video

# 動画機能を有効にしてインストール
cargo install --path . --features video
```

### Cargoでインストール

```bash
# 画像処理のみ（デフォルト）
cargo install --path .

# 動画処理も有効にする
cargo install --path . --features video
```

## 使い方

### 基本的な使用法

```bash
# グリーンバック画像を透過PNGに変換（デフォルト: lime）
chroma-transparent input.png

# 出力ファイル名を指定
chroma-transparent input.png -o output.png

# 色名で指定（CSS色名対応）
chroma-transparent input.png -c blue
chroma-transparent input.png -c magenta
chroma-transparent input.png -c skyblue

# HEXコードで指定
chroma-transparent input.png -c 0000FF
chroma-transparent input.png -c "#FF00FF"
```

### ディレクトリの一括処理

```bash
# ディレクトリ内の画像を一括処理（出力先も指定）
chroma-transparent ./input_dir -o ./output_dir

# 再帰的にサブディレクトリも処理（深さ2まで）
chroma-transparent ./input_dir -o ./output_dir -r 2

# 無制限に再帰処理（深さ0 = 無制限）
chroma-transparent ./input_dir -o ./output_dir -r 0

# 単一ファイルを出力ディレクトリに保存
chroma-transparent input.png -o ./output_dir/
```

### 動画の処理（feature: `video`）

動画ファイルを処理する場合は、`video` featureを有効にしてビルドし、システムにffmpegがインストールされている必要があります。

```bash
# 動画を透過WebMに変換（デフォルト）
chroma-transparent video.mp4

# 出力ファイルを指定
chroma-transparent video.mp4 -o output.webm

# ProRes 4444 MOV形式で出力（高品質、プロ用途）
chroma-transparent video.mp4 --video-format mov -o output.mov

# PNG連番で出力
chroma-transparent video.mp4 --video-format png-sequence -o ./frames/

# 品質とフレームレートを指定
chroma-transparent video.mp4 --video-quality 90 --fps 30

# ffmpegのパスを指定
chroma-transparent video.mp4 --ffmpeg /usr/local/bin/ffmpeg

# 詳細ログを表示
chroma-transparent video.mp4 -o output.webm -v
```

### 詳細なパラメータ調整

```bash
# 許容範囲を広げて色ムラに対応
chroma-transparent input.png -t 0.4

# フェザリングを強めてエッジを滑らかに
chroma-transparent input.png -f 10

# デスピルを強めて色かぶりを除去
chroma-transparent input.png -d 0.9

# ノイズが多い画像に対応（収縮処理を追加）
chroma-transparent input.png -e 2 -D 2

# すべてのオプションを指定
chroma-transparent input.png \
  --output result.png \
  --color 00FF00 \
  --tolerance 0.35 \
  --feather 8 \
  --despill 0.8 \
  --erode 1 \
  --dilate 2 \
  --verbose
```

## コマンドラインオプション

| オプション | 短縮形 | 説明 | デフォルト値 | 範囲 |
|-----------|-------|------|-------------|------|
| `<INPUT>` | - | 入力画像・動画またはディレクトリのパス（必須） | - | - |
| `--output` | `-o` | 出力ファイルまたはディレクトリのパス | `<入力ファイル名>.chroma.png/webm` | - |
| `--color` | `-c` | クロマキー処理する色（HEXコードまたはCSS色名） | `lime`（緑） | CSS色名 or RRGGBB |
| `--tolerance` | `-t` | 色の許容範囲 | `0.3` | 0.0 - 1.0 |
| `--feather` | `-f` | フェザリング量 | `5` | 0 - 50 |
| `--despill` | `-d` | デスピル強度 | `0.7` | 0.0 - 1.0 |
| `--erode` | `-e` | 収縮回数 | `0` | 0 - 10 |
| `--dilate` | `-D` | 膨張回数 | `1` | 0 - 10 |
| `--recursive` | `-r` | 再帰的探索の深さ (0 = 無制限) | なし (直下のみ) | 0 - ∞ |
| `--video-format` | - | 動画出力フォーマット | `webm` | webm, mov, png-sequence |
| `--video-quality` | - | 動画出力品質 | `80` | 1 - 100 |
| `--fps` | - | 動画出力フレームレート | 入力と同じ | - |
| `--ffmpeg` | - | ffmpegのパス | `ffmpeg` | - |
| `--verbose` | `-v` | 詳細ログを出力 | `false` | - |
| `--help` | `-h` | ヘルプを表示 | - | - |
| `--version` | `-V` | バージョンを表示 | - | - |

## パラメータガイド

### tolerance（色の許容範囲）

色検出の許容範囲を指定します。値が大きいほど、ターゲット色に近い色も透過対象になります。

| 値 | 用途 |
|----|-----|
| 0.1 - 0.2 | 完全に均一な色の背景（CGなど） |
| **0.3** | **デフォルト** - 一般的なグリーンバック |
| 0.4 - 0.5 | 照明ムラがある背景 |
| 0.6+ | 非常に不均一な背景（誤検出に注意） |

### feather（フェザリング量）

エッジのぼかし量を指定します。ギザギザしたエッジを滑らかにします。

| 値 | 用途 |
|----|-----|
| 0 | フェザリングなし（シャープなエッジ） |
| 1 - 3 | 軽微なスムージング |
| **5** | **デフォルト** - 自然なエッジ |
| 10+ | ソフトなエッジ効果 |

### despill（デスピル強度）

被写体の縁に残った背景色の反射（色かぶり）を除去する強度を指定します。

| 値 | 用途 |
|----|-----|
| 0.0 | デスピルなし |
| 0.5 | 軽い色かぶり除去 |
| **0.7** | **デフォルト** - 一般的な強度 |
| 0.9 - 1.0 | 強い色かぶりの除去 |

### erode / dilate（収縮 / 膨張）

マスクのモルフォロジー演算回数を指定します。

| erode | dilate | 用途 |
|-------|--------|-----|
| **0** | **1** | **デフォルト** - 一般的な画像 |
| 1-2 | 1-2 | ノイズが多い画像 |
| 0 | 2-3 | 細いエッジを保持したい場合 |
| 2-3 | 0 | 背景の残りを確実に除去 |

## 対応色名一覧

HTML 4.01 基本16色 + CSS3 拡張色（計147色）に対応しています。

### クロマキーでよく使う色

| 色名 | 説明 |
|------|------|
| `lime` | 明るい緑（00FF00）- **デフォルト** |
| `green` | 暗い緑（008000）- HTML標準 |
| `blue` | 青（0000FF） |
| `magenta` / `fuchsia` | マゼンタ（FF00FF） |
| `cyan` / `aqua` | シアン（00FFFF） |

### HTML 4.01 基本16色

`white`, `silver`, `gray`, `black`, `red`, `maroon`, `yellow`, `olive`, `lime`, `green`, `aqua`, `teal`, `blue`, `navy`, `fuchsia`, `purple`

### CSS3 拡張色（一部）

**赤系**: `indianred`, `lightcoral`, `salmon`, `crimson`, `darkred`, `pink`, `hotpink`, `deeppink`

**オレンジ系**: `coral`, `tomato`, `orangered`, `darkorange`, `orange`

**黄系**: `gold`, `lightyellow`, `khaki`

**緑系**: `limegreen`, `forestgreen`, `darkgreen`, `seagreen`, `springgreen`

**青系**: `skyblue`, `lightblue`, `deepskyblue`, `dodgerblue`, `royalblue`, `darkblue`, `midnightblue`

**紫系**: `lavender`, `violet`, `orchid`, `purple`, `indigo`, `rebeccapurple`

**茶系**: `brown`, `chocolate`, `sienna`, `tan`, `beige`

**灰系**: `lightgray`, `darkgray`, `slategray`

> 完全な一覧は [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/#named-colors) を参照してください。

## 処理フロー

```
入力画像
    ↓
RGB → HSV 変換
    ↓
クロマキーマスク生成（指定色を検出）
    ↓
モルフォロジー演算（収縮 → 膨張）
    ↓
アルファチャンネル生成
    ↓
フェザリング（ガウシアンブラー）
    ↓
デスピル処理（色かぶり除去）
    ↓
透過PNG出力
```

## パフォーマンス最適化

高速な画像処理を実現するため、以下の最適化が実装されています。

### 並列処理（Rayon）

- 画像の行単位で並列処理を実行
- CPUコア数に応じて自動スケーリング
- マスク生成、モルフォロジー演算、フェザリング、デスピルすべてに適用

### RGB距離による事前フィルタリング

- HSV変換前にRGB距離で明らかに異なる色を除外
- 不要なHSV計算を約70%削減
- 色検出の高速化

### LUT（ルックアップテーブル）

- RGB→HSV変換用の事前計算テーブル
- 32x32x32 = 32,768エントリ（約400KB）
- 量子化による高速変換

### SIMD最適化

プラットフォームに応じた最適なSIMD命令を自動選択：

| アーキテクチャ | 使用命令セット | 処理単位 |
|---------------|---------------|---------|
| x86_64 (AVX2対応) | AVX2 | 32バイト/回 |
| x86_64 (SSE2のみ) | SSE2 | 16バイト/回 |
| aarch64 (ARM64) | NEON | 16バイト/回 |
| その他 | スカラー | 1バイト/回 |

適用箇所：
- アルファチャンネル適用
- マスク反転処理

### メモリ効率

- **ダブルバッファリング**: モルフォロジー演算で中間画像の再割り当てを削減
- **インプレース処理**: デスピル、アルファ適用で画像コピーを回避
- **バッファ直接操作**: `as_raw()` / `as_mut()` で画像バッファに直接アクセス

### ベンチマーク参考値

| 画像サイズ | 処理時間（目安） | 環境 |
|-----------|----------------|------|
| 1920x1080 | ~50ms | 8コア CPU |
| 3840x2160 | ~150ms | 8コア CPU |
| 7680x4320 | ~500ms | 8コア CPU |

※ 処理時間はCPU性能、パラメータ設定により変動します

## ライブラリとして使用

このプロジェクトはライブラリとしても使用できます。

```rust
use chroma_transparent::{ChromaPipeline, ProcessConfig, Rgb};

fn main() -> anyhow::Result<()> {
    // 設定を構築
    let config = ProcessConfig {
        chroma_color: Rgb::from_hex("00FF00")?,
        tolerance: 0.3,
        feather_amount: 5,
        despill_strength: 0.7,
        erode_iterations: 0,
        dilate_iterations: 1,
        verbose: false,
    };

    // パイプラインを作成
    let pipeline = ChromaPipeline::new(config);

    // 画像を読み込んで処理
    let image = image::open("input.png")?.to_rgba8();
    let result = pipeline.process(&image);

    // 保存
    result.save("output.png")?;

    Ok(())
}
```

## 対応フォーマット

### 画像入力

- PNG
- JPEG
- GIF
- BMP
- WebP
- TIFF
- その他 `image` クレートがサポートするフォーマット

### 画像出力

- PNG（アルファチャンネル付き）

### 動画入力

- MP4
- WebM
- MOV
- AVI
- MKV
- その他 ffmpeg がサポートするフォーマット

### 動画出力

| フォーマット | 説明 | 用途 |
|-------------|------|-----|
| WebM (VP9) | アルファチャンネル付き動画 | Web, 一般用途 |
| MOV (ProRes 4444) | 高品質アルファ動画 | プロ向け編集ソフト |
| PNG連番 | 各フレームをPNGファイルとして出力 | 後処理、編集 |

## Webサーバモード（feature: `server`）

`--features server` でビルドすると、Webサーバモードが利用可能になります。

### サーバの起動

```bash
# ビルド
cargo build --release --features server

# 起動
chroma-transparent --serve

# ポートとストレージを指定
chroma-transparent --serve --port 3000 --storage-dir ./output

# 動画処理も有効化（fullビルド時）
chroma-transparent --serve --enable-video
```

### Web UI

ブラウザで `http://localhost:8080` にアクセスすると、Web UIが表示されます。

- 画像をドラッグ＆ドロップでアップロード
- スライダーでパラメータをリアルタイム調整
- プレビューを確認しながら設定
- 処理してダウンロード

### REST API

| Method | Endpoint | 説明 |
|--------|----------|------|
| GET | `/api/health` | ヘルスチェック |
| GET | `/api/config` | パラメータ設定取得 |
| GET | `/api/docs` | Swagger UI |
| GET | `/api/openapi.json` | OpenAPIスキーマ |
| POST | `/api/preview` | プレビュー生成 |
| POST | `/api/process` | フル画像処理 |
| GET | `/api/download/{id}` | ファイルダウンロード |
| DELETE | `/api/files/{id}` | ファイル削除 |

### ストレージ

- デフォルト: 一時ディレクトリ（サーバ停止時に自動削除）
- `--storage-dir` 指定時: 指定ディレクトリに永続保存
- ファイル名形式: `YYYYMMDD_HHMMSS_hash8.png`

## 動作要件

- Rust 1.70 以上
- 動画処理を使用する場合 (`--features video`): ffmpeg + ffprobe

## Featureフラグ

| Feature | 説明 | デフォルト |
|---------|------|----------|
| `video` | 動画処理機能（ffmpeg連携） | 無効 |
| `server` | Webサーバモード（REST API + GUI） | 無効 |
| `full` | `video` + `server` | 無効 |

## ライセンス

[Apache-2.0](LICENSE)

(c) 2025 aofusa

