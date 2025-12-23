/**
 * chroma-transparent WASM モード
 * 
 * サーバー不要でブラウザ上でクロマキー処理を実行
 */

// WASM モジュールの動的インポート
let wasmModule = null;
let wasmParams = null;
let wasmReady = false;

// 初期化状態のPromise
let initPromise = null;

/**
 * WASMモジュールを初期化
 * @returns {Promise<void>}
 */
async function initWasm() {
    if (initPromise) {
        return initPromise;
    }

    initPromise = (async () => {
        try {
            // wasm-packが生成するモジュールをインポート
            const wasm = await import('./chroma_transparent.js');
            await wasm.default();
            
            wasmModule = wasm;
            wasmParams = new wasm.WasmProcessParams();
            wasmReady = true;
            
            console.log('WASM initialized, version:', wasm.getVersion());
            
            // バージョン表示を更新
            const versionEl = document.getElementById('version');
            if (versionEl) {
                versionEl.textContent = wasm.getVersion();
            }
            
            return wasm;
        } catch (error) {
            console.error('Failed to initialize WASM:', error);
            throw error;
        }
    })();

    return initPromise;
}

/**
 * WASMの準備状態を確認
 * @returns {boolean}
 */
function isWasmReady() {
    return wasmReady;
}

/**
 * 画像データをArrayBufferに変換
 * @param {File|Blob} file 
 * @returns {Promise<Uint8Array>}
 */
async function fileToUint8Array(file) {
    const buffer = await file.arrayBuffer();
    return new Uint8Array(buffer);
}

/**
 * 処理パラメータを更新
 * @param {string} name パラメータ名
 * @param {string|number} value 値
 */
function updateParams(name, value) {
    if (!wasmParams) return;
    
    switch (name) {
        case 'color':
            wasmParams.setColor(String(value));
            break;
        case 'tolerance':
            wasmParams.setTolerance(parseFloat(value));
            break;
        case 'feather':
            wasmParams.setFeather(parseInt(value, 10));
            break;
        case 'despill':
            wasmParams.setDespill(parseFloat(value));
            break;
        case 'erode':
            wasmParams.setErode(parseInt(value, 10));
            break;
        case 'dilate':
            wasmParams.setDilate(parseInt(value, 10));
            break;
    }
}

/**
 * パラメータをリセット
 */
function resetParams() {
    if (wasmParams) {
        wasmParams.reset();
    }
}

/**
 * 画像を処理（フル解像度）
 * @param {Uint8Array} imageData 入力画像データ
 * @returns {Promise<Blob>} 処理後のPNG Blob
 */
async function processImageLocal(imageData) {
    if (!wasmReady) {
        throw new Error('WASM not ready');
    }
    
    const result = wasmModule.processImage(imageData, wasmParams);
    return new Blob([result], { type: 'image/png' });
}

/**
 * プレビュー画像を生成（縮小版）
 * @param {Uint8Array} imageData 入力画像データ
 * @param {number} maxSize 最大サイズ
 * @returns {Promise<Blob>} 処理後のPNG Blob
 */
async function processPreviewLocal(imageData, maxSize = 512) {
    if (!wasmReady) {
        throw new Error('WASM not ready');
    }
    
    const result = wasmModule.processPreview(imageData, wasmParams, maxSize);
    return new Blob([result], { type: 'image/png' });
}

/**
 * 画像情報を取得
 * @param {Uint8Array} imageData 入力画像データ
 * @returns {Promise<{width: number, height: number}>}
 */
async function getImageInfoLocal(imageData) {
    if (!wasmReady) {
        throw new Error('WASM not ready');
    }
    
    return wasmModule.getImageInfo(imageData);
}

// ===== 以下、既存のapp.jsからの移植（APIをWASM呼び出しに置換） =====

// 状態管理
let currentImageData = null;  // Uint8Array
let currentImageFile = null;
let isProcessing = false;
let debounceTimer = null;

// DOM要素
const uploadSection = document.getElementById('upload-section');
const editorSection = document.getElementById('editor-section');
const dropzone = document.getElementById('dropzone');
const fileInput = document.getElementById('file-input');
const fileInputEditor = document.getElementById('file-input-editor');
const originalImage = document.getElementById('original-image');
const previewImage = document.getElementById('preview-image');
const loadingOverlay = document.getElementById('loading-overlay');
const statusFilename = document.getElementById('status-filename');
const statusDimensions = document.getElementById('status-dimensions');

// パラメータ要素
const colorSelect = document.getElementById('color-select');
const colorCustom = document.getElementById('color-custom');
const colorPreview = document.getElementById('color-preview');
const chromaColorPicker = document.getElementById('chroma-color-picker');
const toleranceInput = document.getElementById('tolerance');
const featherInput = document.getElementById('feather');
const despillInput = document.getElementById('despill');
const erodeInput = document.getElementById('erode');
const dilateInput = document.getElementById('dilate');
const resetBtn = document.getElementById('reset-btn');
const processBtn = document.getElementById('process-btn');

// 色名とHEXのマッピング
const colorMap = {
    'lime': '#00ff00',
    'green': '#008000',
    'blue': '#0000ff',
    'magenta': '#ff00ff',
    'cyan': '#00ffff',
    'red': '#ff0000',
    'yellow': '#ffff00',
    'white': '#ffffff',
    'black': '#000000'
};

/**
 * 初期化
 */
async function init() {
    // WASM初期化
    try {
        await initWasm();
    } catch (error) {
        console.error('WASM initialization failed:', error);
        alert('WASMの初期化に失敗しました。ブラウザがWebAssemblyをサポートしているか確認してください。');
        return;
    }

    // イベントリスナーを設定
    setupEventListeners();
    
    // 初期色プレビュー
    updateColorPreview();
}

/**
 * イベントリスナーを設定
 */
function setupEventListeners() {
    // ドラッグ&ドロップ
    dropzone.addEventListener('dragover', handleDragOver);
    dropzone.addEventListener('dragleave', handleDragLeave);
    dropzone.addEventListener('drop', handleDrop);
    
    // ファイル選択
    fileInput.addEventListener('change', handleFileSelect);
    fileInputEditor.addEventListener('change', handleFileSelect);
    
    // パラメータ変更
    colorSelect.addEventListener('change', handleColorChange);
    colorCustom.addEventListener('input', handleColorCustomChange);
    chromaColorPicker.addEventListener('input', handleColorPickerChange);
    colorPreview.addEventListener('click', () => chromaColorPicker.click());
    
    toleranceInput.addEventListener('input', handleParamChange);
    featherInput.addEventListener('input', handleParamChange);
    despillInput.addEventListener('input', handleParamChange);
    erodeInput.addEventListener('input', handleParamChange);
    dilateInput.addEventListener('input', handleParamChange);
    
    // ボタン
    resetBtn.addEventListener('click', handleReset);
    processBtn.addEventListener('click', handleProcess);
}

/**
 * ドラッグオーバー
 */
function handleDragOver(e) {
    e.preventDefault();
    dropzone.classList.add('dragover');
}

/**
 * ドラッグリーブ
 */
function handleDragLeave(e) {
    e.preventDefault();
    dropzone.classList.remove('dragover');
}

/**
 * ドロップ
 */
async function handleDrop(e) {
    e.preventDefault();
    dropzone.classList.remove('dragover');
    
    const files = e.dataTransfer.files;
    if (files.length > 0) {
        await loadImage(files[0]);
    }
}

/**
 * ファイル選択
 */
async function handleFileSelect(e) {
    const files = e.target.files;
    if (files.length > 0) {
        await loadImage(files[0]);
    }
}

/**
 * 画像を読み込み
 * @param {File} file 
 */
async function loadImage(file) {
    if (!file.type.startsWith('image/')) {
        alert('画像ファイルを選択してください。');
        return;
    }
    
    currentImageFile = file;
    currentImageData = await fileToUint8Array(file);
    
    // 元画像を表示
    const url = URL.createObjectURL(file);
    originalImage.src = url;
    
    // 画像情報を取得
    try {
        const info = await getImageInfoLocal(currentImageData);
        statusFilename.textContent = file.name;
        statusDimensions.textContent = `${info.width} × ${info.height}`;
    } catch (error) {
        console.error('Failed to get image info:', error);
    }
    
    // エディタセクションを表示
    uploadSection.hidden = true;
    editorSection.hidden = false;
    
    // プレビューを生成
    await updatePreview();
}

/**
 * 色変更ハンドラ
 */
function handleColorChange(e) {
    const value = e.target.value;
    
    if (value === 'custom') {
        colorCustom.hidden = false;
    } else {
        colorCustom.hidden = true;
        updateParams('color', value);
        updateColorPreview();
        debouncedUpdatePreview();
    }
}

/**
 * カスタム色変更ハンドラ
 */
function handleColorCustomChange(e) {
    const value = e.target.value.replace('#', '');
    if (/^[0-9a-fA-F]{6}$/.test(value)) {
        updateParams('color', value);
        updateColorPreview();
        debouncedUpdatePreview();
    }
}

/**
 * カラーピッカー変更ハンドラ
 */
function handleColorPickerChange(e) {
    const hex = e.target.value.replace('#', '');
    colorSelect.value = 'custom';
    colorCustom.hidden = false;
    colorCustom.value = '#' + hex;
    updateParams('color', hex);
    updateColorPreview();
    debouncedUpdatePreview();
}

/**
 * パラメータ変更ハンドラ
 */
function handleParamChange(e) {
    const input = e.target;
    const name = input.id;
    const value = input.value;
    
    // 値表示を更新
    const valueSpan = document.getElementById(`${name}-value`);
    if (valueSpan) {
        if (name === 'tolerance' || name === 'despill') {
            valueSpan.textContent = parseFloat(value).toFixed(2);
        } else {
            valueSpan.textContent = value;
        }
    }
    
    updateParams(name, value);
    debouncedUpdatePreview();
}

/**
 * 色プレビューを更新
 */
function updateColorPreview() {
    let color;
    const selected = colorSelect.value;
    
    if (selected === 'custom') {
        color = colorCustom.value;
    } else {
        color = colorMap[selected] || '#00ff00';
    }
    
    colorPreview.style.backgroundColor = color;
    chromaColorPicker.value = color;
}

/**
 * リセットハンドラ
 */
async function handleReset() {
    resetParams();
    
    // UIを更新
    colorSelect.value = 'lime';
    colorCustom.hidden = true;
    toleranceInput.value = 0.3;
    featherInput.value = 5;
    despillInput.value = 0.7;
    erodeInput.value = 0;
    dilateInput.value = 1;
    
    document.getElementById('tolerance-value').textContent = '0.30';
    document.getElementById('feather-value').textContent = '5';
    document.getElementById('despill-value').textContent = '0.70';
    document.getElementById('erode-value').textContent = '0';
    document.getElementById('dilate-value').textContent = '1';
    
    updateColorPreview();
    await updatePreview();
}

/**
 * 処理ハンドラ（ダウンロード）
 */
async function handleProcess() {
    if (!currentImageData || isProcessing) return;
    
    isProcessing = true;
    processBtn.disabled = true;
    processBtn.textContent = '処理中...';
    loadingOverlay.hidden = false;
    
    try {
        const blob = await processImageLocal(currentImageData);
        
        // ダウンロード
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = (currentImageFile?.name?.replace(/\.[^.]+$/, '') || 'image') + '.chroma.png';
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    } catch (error) {
        console.error('Processing failed:', error);
        alert('処理に失敗しました: ' + error.message);
    } finally {
        isProcessing = false;
        processBtn.disabled = false;
        processBtn.textContent = '処理してダウンロード';
        loadingOverlay.hidden = true;
    }
}

/**
 * プレビューを更新（デバウンス付き）
 */
function debouncedUpdatePreview() {
    if (debounceTimer) {
        clearTimeout(debounceTimer);
    }
    debounceTimer = setTimeout(updatePreview, 150);
}

/**
 * プレビューを更新
 */
async function updatePreview() {
    if (!currentImageData || isProcessing) return;
    
    isProcessing = true;
    loadingOverlay.hidden = false;
    
    try {
        const blob = await processPreviewLocal(currentImageData, 512);
        const url = URL.createObjectURL(blob);
        
        // 古いURLを解放
        if (previewImage.src && previewImage.src.startsWith('blob:')) {
            URL.revokeObjectURL(previewImage.src);
        }
        
        previewImage.src = url;
    } catch (error) {
        console.error('Preview generation failed:', error);
    } finally {
        isProcessing = false;
        loadingOverlay.hidden = true;
    }
}

// 初期化を実行
document.addEventListener('DOMContentLoaded', init);

// エクスポート（モジュールとして使用する場合）
export {
    initWasm,
    isWasmReady,
    updateParams,
    resetParams,
    processImageLocal,
    processPreviewLocal,
    getImageInfoLocal
};

