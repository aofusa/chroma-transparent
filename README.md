# chroma-transparent

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-Apache--2.0-blue)

指定した画像・動画の指定された色をクロマキー処理して透過ファイルに変換するCLIツールです。

グリーンバック画像/動画やブルーバック画像/動画など、単色背景から被写体を切り抜いて透過PNG/WebMを生成できます。

## 機能

- **クロマキー処理**: 指定した色（HEXコードまたはCSS色名）を透過に変換
- **CSS 147色対応**: `lime`, `blue`, `magenta` などの色名で指定可能
- **HSV色空間での色検出**: 照明条件の変化に強い色検出アルゴリズム
- **拡張色空間対応**: HSV、LAB、LCH、YUV色空間に対応
- **多色検出**: 複数の色を同時に検出してマスクを生成
- **モルフォロジー演算**: 収縮（Erode）/膨張（Dilate）でマスクを微調整
- **フェザリング**: ガウシアンブラーでエッジを滑らかに
- **デスピル**: 被写体の縁に残った背景色の反射（色かぶり）を除去
- **バイラテラルフィルタ**: エッジを保持しながらノイズを除去
- **マルチスケール処理**: 複数の解像度で処理して統合
- **マットエッジ最適化**: アルファチャンネルのエッジを最適化
- **影の処理**: 背景の影を検出して除去
- **エッジシャープニング**: アンシャープマスクでエッジを強調
- **柔軟なパラメータ調整**: すべての処理パラメータをコマンドラインから調整可能
- **ディレクトリ一括処理**: ディレクトリ内の全画像ファイルを一括処理
- **再帰的探索**: サブディレクトリも含めて処理（深さ指定可能）
- **動画対応** (feature: `video`): ffmpegを使用してMP4, WebM, MOV等の動画を透過動画（WebM/MOV）に変換
- **Webサーバモード** (feature: `server`): ブラウザからGUIで操作、REST API提供、OpenAPI対応、Graceful Shutdown対応
- **WASMモード** (feature: `wasm`): サーバー不要でブラウザ上のみで画像処理が可能な静的配信版

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

# Webサーバモードを有効にする
cargo install --path . --features server

# 全機能（CLI + サーバ + 動画）
cargo install --path . --features full
```

### WASMビルド

ブラウザ上のみで動作する静的配信版をビルドするには、`wasm-pack` が必要です。

```bash
# wasm-packのインストール（未インストールの場合）
cargo install wasm-pack

# WASMビルド（ビルドスクリプト使用）
./scripts/build-wasm.sh

# リリースビルド（最適化あり）
./scripts/build-wasm.sh --release

# 手動でビルドする場合
wasm-pack build --target web --out-dir pkg -- --no-default-features --features wasm
```

ビルド後、`static/` ディレクトリに静的配信用ファイルが生成されます。

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

# 色空間を変更（LAB色空間を使用）
chroma-transparent input.png --color-space lab

# 多色検出（複数の色を同時に検出）
chroma-transparent input.png \
  --multi-color lime:0.3 \
  --multi-color blue:0.4

# バイラテラルフィルタでノイズ除去
chroma-transparent input.png \
  --bilateral \
  --bilateral-spatial-sigma 5.0 \
  --bilateral-color-sigma 50.0 \
  --bilateral-radius 5

# マルチスケール処理で精度向上
chroma-transparent input.png \
  --multiscale \
  --multiscale-levels 3 \
  --multiscale-scale-factor 0.5

# マットエッジ最適化
chroma-transparent input.png \
  --edge-optimization \
  --edge-threshold 0.1 \
  --edge-smoothness 0.5

# 影の処理
chroma-transparent input.png \
  --shadow-removal \
  --shadow-threshold 0.3 \
  --shadow-removal-strength 0.7

# エッジシャープニング
chroma-transparent input.png \
  --sharpen \
  --sharpen-amount 0.5 \
  --sharpen-radius 1.0 \
  --sharpen-threshold 0.0

# 適応的許容範囲（照明ムラがある画像に有効）
chroma-transparent input.png \
  --adaptive-tolerance \
  --adaptive-tolerance-grid-w 12 \
  --adaptive-tolerance-grid-h 12 \
  --adaptive-tolerance-sensitivity 1.2
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
| `--multi-color` | - | 多色検出（複数回指定可能、形式: COLOR:TOLERANCE） | なし | - |
| `--color-space` | - | 色空間 (hsv, lab, lch, yuv) | `hsv` | hsv, lab, lch, yuv |
| `--bilateral` | - | バイラテラルフィルタを有効化 | `false` | - |
| `--bilateral-spatial-sigma` | - | バイラテラルフィルタ: 空間的重みの標準偏差 | `5.0` | 1.0 - 20.0 |
| `--bilateral-color-sigma` | - | バイラテラルフィルタ: 色の重みの標準偏差 | `50.0` | 10.0 - 100.0 |
| `--bilateral-radius` | - | バイラテラルフィルタ: カーネル半径 | `5` | 1 - 10 |
| `--multiscale` | - | マルチスケール処理を有効化 | `false` | - |
| `--multiscale-levels` | - | マルチスケール処理: スケールレベル数 | `3` | 1 - 5 |
| `--multiscale-scale-factor` | - | マルチスケール処理: スケール係数 | `0.5` | 0.25 - 0.75 |
| `--edge-optimization` | - | マットエッジ最適化を有効化 | `false` | - |
| `--edge-threshold` | - | マットエッジ最適化: エッジ検出の閾値 | `0.1` | 0.0 - 1.0 |
| `--edge-smoothness` | - | マットエッジ最適化: エッジの滑らかさ | `0.5` | 0.0 - 1.0 |
| `--shadow-removal` | - | 影の処理を有効化 | `false` | - |
| `--shadow-threshold` | - | 影の処理: 影検出の閾値 | `0.3` | 0.0 - 1.0 |
| `--shadow-removal-strength` | - | 影の処理: 影除去の強度 | `0.7` | 0.0 - 1.0 |
| `--sharpen` | - | エッジシャープニングを有効化 | `false` | - |
| `--sharpen-amount` | - | エッジシャープニング: シャープニング強度 | `0.5` | 0.0 - 2.0 |
| `--sharpen-radius` | - | エッジシャープニング: シャープニング半径 | `1.0` | 0.1 - 5.0 |
| `--sharpen-threshold` | - | エッジシャープニング: シャープニング閾値 | `0.0` | 0.0 - 1.0 |
| `--adaptive-tolerance` | - | 適応的許容範囲を有効化 | `false` | - |
| `--adaptive-tolerance-grid-w` | - | 適応的許容範囲: グリッドサイズ（幅） | `8` | 4 - 32 |
| `--adaptive-tolerance-grid-h` | - | 適応的許容範囲: グリッドサイズ（高さ） | `8` | 4 - 32 |
| `--adaptive-tolerance-sensitivity` | - | 適応的許容範囲: 感度調整 | `1.0` | 0.0 - 2.0 |
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

### adaptive-tolerance（適応的許容範囲）

画像をグリッド分割し、各領域の色分布を分析して最適なtolerance値を自動計算します。照明ムラがある画像や不均一な背景色を持つ画像での精度向上に有効です。

```bash
# 適応的許容範囲を有効化
chroma-transparent input.png --adaptive-tolerance

# グリッドサイズを指定（デフォルト: 8x8）
chroma-transparent input.png \
  --adaptive-tolerance \
  --adaptive-tolerance-grid-w 16 \
  --adaptive-tolerance-grid-h 16

# 感度を調整（デフォルト: 1.0）
chroma-transparent input.png \
  --adaptive-tolerance \
  --adaptive-tolerance-sensitivity 1.5
```

| パラメータ | 説明 | デフォルト値 | 範囲 |
|-----------|------|------------|------|
| `--adaptive-tolerance` | 適応的許容範囲を有効化 | `false` | - |
| `--adaptive-tolerance-grid-w` | グリッドサイズ（幅） | `8` | 4 - 32 |
| `--adaptive-tolerance-grid-h` | グリッドサイズ（高さ） | `8` | 4 - 32 |
| `--adaptive-tolerance-sensitivity` | 感度調整 | `1.0` | 0.0 - 2.0 |

**使用例**:
- 照明ムラがある画像: `--adaptive-tolerance --adaptive-tolerance-sensitivity 1.2`
- 不均一な背景色: `--adaptive-tolerance --adaptive-tolerance-grid-w 12 --adaptive-tolerance-grid-h 12`
- 高精度が必要な場合: `--adaptive-tolerance --adaptive-tolerance-grid-w 16 --adaptive-tolerance-grid-h 16`

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

### color-space（色空間）

色検出に使用する色空間を指定します。異なる色空間は異なる特性を持ち、画像の特性に応じて最適な色空間を選択できます。

| 色空間 | 説明 | 用途 |
|--------|------|-----|
| **hsv** | **デフォルト** - 色相・彩度・明度 | 一般的なクロマキー処理、照明条件の変化に強い |
| lab | 知覚的均一性に優れた色空間 | より正確な色検出が必要な場合 |
| lch | LABの極座標表現 | 色相と彩度を重視した検出 |
| yuv | 輝度と色差成分 | 動画処理や特定の色域に適している場合 |

### multi-color（多色検出）

複数の色を同時に検出してマスクを生成します。背景が複数の色で構成されている場合や、異なる照明条件で撮影された画像に有効です。

```bash
# 複数の色を指定（各色に許容範囲を設定）
chroma-transparent input.png \
  --multi-color lime:0.3 \
  --multi-color blue:0.4 \
  --multi-color green:0.35
```

### bilateral（バイラテラルフィルタ）

エッジを保持しながらノイズを除去するフィルタです。アルファチャンネルのノイズが多い場合に有効です。

| パラメータ | 説明 | 推奨値 |
|-----------|------|--------|
| `--bilateral-spatial-sigma` | 空間的重みの標準偏差（大きいほど広範囲を平滑化） | 5.0 |
| `--bilateral-color-sigma` | 色の重みの標準偏差（大きいほど色の違いを許容） | 50.0 |
| `--bilateral-radius` | カーネル半径（処理範囲） | 5 |

### multiscale（マルチスケール処理）

複数の解像度で処理して統合することで、細部と全体のバランスを取ります。複雑なエッジや細かい部分が多い画像に有効です。

| パラメータ | 説明 | 推奨値 |
|-----------|------|--------|
| `--multiscale-levels` | スケールレベル数（多いほど精度が上がるが処理時間も増加） | 3 |
| `--multiscale-scale-factor` | スケール係数（各レベルでの縮小率） | 0.5 |

### edge-optimization（マットエッジ最適化）

アルファチャンネルのエッジを最適化して、より自然なエッジ表現を実現します。

| パラメータ | 説明 | 推奨値 |
|-----------|------|--------|
| `--edge-threshold` | エッジ検出の閾値（低いほど多くのエッジを検出） | 0.1 |
| `--edge-smoothness` | エッジの滑らかさ（高いほど滑らかになる） | 0.5 |

### shadow-removal（影の処理）

背景の影を検出して除去します。グリーンバックなどで影が残っている場合に有効です。

| パラメータ | 説明 | 推奨値 |
|-----------|------|--------|
| `--shadow-threshold` | 影検出の閾値（低いほど多くの影を検出） | 0.3 |
| `--shadow-removal-strength` | 影除去の強度（高いほど強く除去） | 0.7 |

### sharpen（エッジシャープニング）

アンシャープマスクを使用してエッジを強調します。エッジがぼやけている場合に有効です。

| パラメータ | 説明 | 推奨値 |
|-----------|------|--------|
| `--sharpen-amount` | シャープニング強度（高いほど強く強調） | 0.5 |
| `--sharpen-radius` | シャープニング半径（大きいほど広範囲を強調） | 1.0 |
| `--sharpen-threshold` | シャープニング閾値（低いほど多くのエッジを強調） | 0.0 |

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
色空間変換（HSV/LAB/LCH/YUV）
    ↓
クロマキーマスク生成（指定色を検出、多色検出対応）
    ↓
影の処理（有効時）
    ↓
モルフォロジー演算（収縮 → 膨張）
    ↓
アルファチャンネル生成
    ↓
マットエッジ最適化（有効時）
    ↓
フェザリング（ガウシアンブラー）
    ↓
バイラテラルフィルタ（有効時）
    ↓
デスピル処理（色かぶり除去）
    ↓
エッジシャープニング（有効時）
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

Web UIでは以下のパラメータを調整できます：

- **基本パラメータ**: クロマキー色、許容範囲、フェザリング、デスピル、収縮/膨張
- **色空間選択**: HSV、LAB、LCH、YUVから選択
- **多色検出**: 複数の色を同時に検出（各色に許容範囲を設定）
- **適応的許容範囲**: 画像領域ごとに最適な許容範囲を自動計算（照明ムラがある画像に有効）
- **バイラテラルフィルタ**: エッジを保持しながらノイズを除去
- **マルチスケール処理**: 複数の解像度で処理して統合
- **マットエッジ最適化**: アルファチャンネルのエッジを最適化
- **影の処理**: 背景の影を検出して除去
- **エッジシャープニング**: アンシャープマスクでエッジを強調

各パラメータはリアルタイムプレビューで確認できます。

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

### Graceful Shutdown

サーバーは Ctrl+C（SIGINT）または SIGTERM シグナルを受け取ると、処理中のリクエストが完了するまで待機してから安全に終了します。

```bash
# サーバー起動
chroma-transparent --serve

# 停止（Ctrl+C で Graceful Shutdown）
# 処理中のリクエストが完了してから終了
^C
# Server stopped gracefully
```

## WASMモード（静的配信）

`--features wasm` でビルドすると、サーバー不要でブラウザ上のみで動作する静的配信版を作成できます。画像データはサーバーに送信されず、すべてブラウザ上で処理されます。

### WASMビルド

```bash
# ビルドスクリプトを使用（推奨）
./scripts/build-wasm.sh --release

# 生成されるファイル
static/
├── index.html                     # WASM用HTML
├── style.css                      # スタイルシート
├── app.js                         # WASMモードJS
├── chroma_transparent.js          # WASMバインディング
└── chroma_transparent_bg.wasm     # WASMバイナリ (~4MB)
```

### ローカルで確認

```bash
# Python の HTTP サーバーで配信（Workerモード）
python3 -m http.server 8000 --directory static

# SharedArrayBufferモード（COOP/COEPヘッダー付き、最高速）
python3 scripts/serve-with-coop-coep.py

# ポート指定
python3 scripts/serve-with-coop-coep.py -p 3000

# 外部公開（LAN内の他デバイスからアクセス可能）
python3 scripts/serve-with-coop-coep.py --public

# または npx serve を使用
npx serve static

# ブラウザでアクセス
# http://localhost:8000/
```

### 処理モードの自動選択

WASM版は環境に応じて最適な処理モードを自動選択します：

| モード | 条件 | パフォーマンス | 表示 |
|--------|------|---------------|------|
| **SharedArrayBuffer** | COOP/COEPヘッダーあり | 最高速（ゼロコピー） | `(SharedArrayBuffer)` |
| **Worker** | Workerが利用可能 | 高速（UIブロックなし） | `(Worker)` |
| **Direct** | file://など | 動作保証（UIブロックあり） | `(Direct)` |

### ホスティング環境別の動作

| ホスティング | 追加設定 | 使用モード |
|-------------|---------|------------|
| **GitHub Pages** | 不要 | Worker |
| **Netlify** | `_headers`ファイル追加で高速化可能 | SharedArrayBuffer (設定時) |
| **Vercel** | `vercel.json`設定で高速化可能 | SharedArrayBuffer (設定時) |
| **ローカル** | `serve-with-coop-coep.py`使用 | SharedArrayBuffer |

### Netlify で SharedArrayBuffer を有効化

`static/_headers` ファイルを作成：

```
/*
  Cross-Origin-Opener-Policy: same-origin
  Cross-Origin-Embedder-Policy: require-corp
```

### Vercel で SharedArrayBuffer を有効化

`vercel.json` を作成：

```json
{
  "headers": [
    {
      "source": "/(.*)",
      "headers": [
        { "key": "Cross-Origin-Opener-Policy", "value": "same-origin" },
        { "key": "Cross-Origin-Embedder-Policy", "value": "require-corp" }
      ]
    }
  ]
}
```

### serve-with-coop-coep.py オプション

| オプション | 説明 | デフォルト |
|-----------|------|----------|
| `-p, --port PORT` | ポート番号 | 8080 |
| `-H, --host HOST` | バインドするホストアドレス | 127.0.0.1 |
| `-P, --public` | 外部公開モード（0.0.0.0にバインド） | - |
| `-d, --directory DIR` | サーブするディレクトリ | static/ |

```bash
# 使用例
python3 scripts/serve-with-coop-coep.py                    # localhost:8080
python3 scripts/serve-with-coop-coep.py -p 3000            # localhost:3000
python3 scripts/serve-with-coop-coep.py --public           # 0.0.0.0:8080（外部公開）
python3 scripts/serve-with-coop-coep.py -P -p 3000         # 0.0.0.0:3000（外部公開）
python3 scripts/serve-with-coop-coep.py -H 192.168.1.100   # 指定IPでバインド
```

### 特徴

- **オフライン動作**: サーバー不要、静的ファイルのみで動作
- **プライバシー**: 画像データはブラウザ外に送信されない
- **デプロイ容易**: 任意の静的ホスティングサービス（GitHub Pages, Netlify, Vercel等）で配信可能

### 制限事項

- **シングルスレッド**: WebAssembly では Rayon 並列処理が無効のため、大画像の処理に時間がかかる場合があります
- **メモリ制限**: ブラウザのメモリ制限により、非常に大きな画像（16MP以上）は処理できない場合があります
- **動画非対応**: WASM版では動画処理機能は利用できません

### JavaScript API

WASM版はJavaScriptから直接呼び出すこともできます。

```javascript
import init, { WasmProcessParams, processImage, processPreview, getVersion } from './chroma_transparent.js';

// 初期化
await init();

// パラメータ設定
const params = new WasmProcessParams();
params.setColor("lime");
params.setTolerance(0.3);
params.setFeather(5);
params.setDespill(0.7);

// 画像処理
const imageData = new Uint8Array(await file.arrayBuffer());
const result = processImage(imageData, params);  // Uint8Array (PNG)

// プレビュー生成（縮小版）
const preview = processPreview(imageData, params, 512);

// Blob に変換してダウンロード
const blob = new Blob([result], { type: 'image/png' });
const url = URL.createObjectURL(blob);
```

## 動作要件

- Rust 1.70 以上
- 動画処理を使用する場合 (`--features video`): ffmpeg + ffprobe

## Featureフラグ

| Feature | 説明 | デフォルト |
|---------|------|----------|
| `cli` | CLIツール機能（clap, env_logger等） | **有効** |
| `parallel` | Rayon並列処理（高速化） | **有効** |
| `video` | 動画処理機能（ffmpeg連携） | 無効 |
| `server` | Webサーバモード（REST API + GUI） | 無効 |
| `wasm` | WebAssembly版（ブラウザ上で動作） | 無効 |
| `full` | `cli` + `parallel` + `video` + `server` | 無効 |

### ビルドパターン

```bash
# デフォルト（CLI + 並列処理）
cargo build --release

# サーバモード
cargo build --release --features server

# 動画対応
cargo build --release --features video

# 全機能
cargo build --release --features full

# WASM版（CLI/サーバ/並列処理は無効）
wasm-pack build --target web -- --no-default-features --features wasm
```

### アーキテクチャ

```
┌─────────────────────────────────────────────────────────┐
│                    chroma-transparent                    │
├─────────────────────────────────────────────────────────┤
│  Core Modules (共通)                                     │
│  ┌─────────┐ ┌─────────┐ ┌──────────┐ ┌──────────────┐ │
│  │ color   │ │ config  │ │ pipeline │ │ processor/*  │ │
│  └─────────┘ └─────────┘ └──────────┘ └──────────────┘ │
├─────────────────────────────────────────────────────────┤
│  CLI Mode (--features cli)      │  WASM Mode            │
│  ┌─────────┐ ┌─────────┐       │  (--features wasm)    │
│  │ cli.rs  │ │ main.rs │       │  ┌─────────────┐      │
│  └─────────┘ └─────────┘       │  │ wasm.rs     │      │
│  + server (--features server)   │  │ wasm.js     │      │
│                                 │  │ index.html  │      │
├─────────────────────────────────┴──┴─────────────┴──────┤
│  Parallel Processing (--features parallel)               │
│  ┌──────────────────────────────────────────────────┐   │
│  │ rayon (有効時のみ、WASM非対応)                     │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## ライセンス

[Apache-2.0](LICENSE)

(c) 2025 aofusa

