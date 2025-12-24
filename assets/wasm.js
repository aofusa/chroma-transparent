/**
 * chroma-transparent WASM モード
 * Advanced Compare View with Zoom/Pan/Slider
 * 
 * 3段階フォールバック対応:
 * 1. SharedArrayBuffer モード (COOP/COEP環境で最高速)
 * 2. Worker モード (コピー転送、UIブロックなし)
 * 3. Direct モード (メインスレッド、フォールバック)
 */

// === 処理モード定数 ===
const ProcessorMode = {
    SHARED: 'shared',   // SharedArrayBuffer (最高速)
    WORKER: 'worker',   // Worker + コピー転送
    DIRECT: 'direct'    // メインスレッド
};

// === SharedArrayBuffer メモリレイアウト ===
const SHARED_CONTROL_SIZE = 256;  // 制御ブロック
const SHARED_MAX_IMAGE_SIZE = 100 * 1024 * 1024;  // 100MB

// 制御ブロックオフセット (Int32)
const CTRL_STATUS = 0;       // 状態
const CTRL_TASK_TYPE = 1;    // タスク種別
const CTRL_INPUT_SIZE = 2;   // 入力サイズ
const CTRL_OUTPUT_SIZE = 3;  // 出力サイズ
const CTRL_MAX_SIZE = 4;     // プレビュー最大サイズ
const CTRL_ERROR = 5;        // エラーコード

// 状態定数
const STATUS_IDLE = 0;
const STATUS_PROCESSING = 1;
const STATUS_DONE = 2;
const STATUS_ERROR = 3;

// タスク種別
const TASK_PREVIEW = 1;
const TASK_PROCESS = 2;

// パラメータオフセット (Float32, byte offset 64)
const PARAM_TOLERANCE = 0;   // [64-67]
const PARAM_DESPILL = 1;     // [68-71]
const PARAM_FEATHER = 2;     // [72-75] as int
const PARAM_ERODE = 3;       // [76-79] as int
const PARAM_DILATE = 4;      // [80-83] as int

// 色オフセット (byte offset 128, 64 bytes for color string)
const COLOR_OFFSET = 128;
const COLOR_SIZE = 64;

// データオフセット
const DATA_OFFSET = SHARED_CONTROL_SIZE;

// === WASM/Worker 処理管理 ===

/**
 * ChromaProcessor - 3段階フォールバック対応
 */
class ChromaProcessor {
    constructor() {
        // モード
        this.mode = null;  // 'shared' | 'worker' | 'direct'
        
        // Shared モード用
        this.sharedBuffer = null;
        this.controlView = null;
        this.paramView = null;
        this.dataView = null;
        
        // Worker モード用
        this.worker = null;
        this.pendingTasks = new Map();
        this.taskId = 0;
        
        // Direct モード用
        this.wasmModule = null;
        this.wasmParams = null;
        
        // 共通
        this.ready = false;
        this.version = '-';
        this.initPromise = null;
    }
    
    /**
     * 初期化 - 最適なモードを自動選択
     */
    async init() {
        if (this.initPromise) {
            return this.initPromise;
        }
        
        this.initPromise = this._init();
        return this.initPromise;
    }
    
    async _init() {
        // 優先度1: SharedArrayBuffer
        if (this._checkSharedArrayBufferSupport()) {
            try {
                await this._initSharedMode();
                this.mode = ProcessorMode.SHARED;
                console.log('ChromaProcessor: Using SharedArrayBuffer mode (fastest)');
                return this.version;
            } catch (e) {
                console.warn('SharedArrayBuffer init failed:', e.message);
            }
        }
        
        // 優先度2: Worker (コピー転送)
        if (this._checkWorkerSupport()) {
            try {
                await this._initWorkerMode();
                this.mode = ProcessorMode.WORKER;
                console.log('ChromaProcessor: Using Worker mode (copy transfer)');
                return this.version;
            } catch (e) {
                console.warn('Worker init failed:', e.message);
            }
        }
        
        // 優先度3: Direct (メインスレッド)
        await this._initDirectMode();
        this.mode = ProcessorMode.DIRECT;
        console.log('ChromaProcessor: Using Direct mode (main thread)');
        return this.version;
    }
    
    /**
     * SharedArrayBuffer サポートチェック
     */
    _checkSharedArrayBufferSupport() {
        // 1. SharedArrayBuffer が存在するか
        if (typeof SharedArrayBuffer === 'undefined') {
            console.log('SharedArrayBuffer: not available');
            return false;
        }
        
        // 2. crossOriginIsolated かどうか（COOP/COEPヘッダーが設定されているか）
        if (!self.crossOriginIsolated) {
            console.log('SharedArrayBuffer: page is not cross-origin isolated');
            return false;
        }
        
        console.log('SharedArrayBuffer: available');
        return true;
    }
    
    /**
     * Worker サポートチェック
     */
    _checkWorkerSupport() {
        if (typeof Worker === 'undefined') {
            return false;
        }
        
        // file:// プロトコルでは Worker が動作しない
        if (location.protocol === 'file:') {
            return false;
        }
        
        return true;
    }
    
    /**
     * SharedArrayBuffer モードで初期化
     */
    async _initSharedMode() {
        return new Promise((resolve, reject) => {
            const timeout = setTimeout(() => {
                reject(new Error('SharedArrayBuffer Worker initialization timeout'));
            }, 15000);
            
            // SharedArrayBuffer を作成
            const bufferSize = SHARED_CONTROL_SIZE + SHARED_MAX_IMAGE_SIZE * 2;
            this.sharedBuffer = new SharedArrayBuffer(bufferSize);
            
            // ビューを作成
            this.controlView = new Int32Array(this.sharedBuffer, 0, 16);
            this.paramView = new Float32Array(this.sharedBuffer, 64, 8);
            this.dataView = new Uint8Array(this.sharedBuffer, DATA_OFFSET);
            
            // 初期状態を設定
            Atomics.store(this.controlView, CTRL_STATUS, STATUS_IDLE);
            
            // Worker を起動
            this.worker = new Worker('./chroma-worker.js', { type: 'module' });
            
            const handleMessage = (e) => {
                const { type, taskId, payload } = e.data;
                
                if (type === 'loaded') {
                    // Worker読み込み完了、SharedArrayBufferで初期化
                    this.worker.postMessage({
                        type: 'init-shared',
                        taskId: 0,
                        payload: { 
                            wasmUrl: './chroma_transparent.js',
                            sharedBuffer: this.sharedBuffer
                        }
                    });
                } else if (type === 'ready' && taskId === 0) {
                    clearTimeout(timeout);
                    this.ready = true;
                    this.version = payload.version;
                    resolve(this.version);
                } else if (type === 'error' && taskId === 0) {
                    clearTimeout(timeout);
                    reject(new Error(payload.message));
                }
            };
            
            this.worker.onmessage = handleMessage;
            this.worker.onerror = (e) => {
                clearTimeout(timeout);
                reject(new Error(`Worker error: ${e.message}`));
            };
        });
    }
    
    /**
     * Worker モードで初期化 (コピー転送)
     */
    async _initWorkerMode() {
        return new Promise((resolve, reject) => {
            const timeout = setTimeout(() => {
                reject(new Error('Worker initialization timeout'));
            }, 10000);
            
            this.worker = new Worker('./chroma-worker.js', { type: 'module' });
            
            const handleMessage = (e) => {
                const { type, taskId, payload } = e.data;
                
                if (type === 'loaded') {
                    this.worker.postMessage({
                        type: 'init',
                        taskId: 0,
                        payload: { wasmUrl: './chroma_transparent.js' }
                    });
                } else if (type === 'ready' && taskId === 0) {
                    clearTimeout(timeout);
                    this.ready = true;
                    this.version = payload.version;
                    resolve(this.version);
                } else if (type === 'error' && taskId === 0) {
                    clearTimeout(timeout);
                    reject(new Error(payload.message));
                } else {
                    this._handleWorkerMessage(e);
                }
            };
            
            this.worker.onmessage = handleMessage;
            this.worker.onerror = (e) => {
                clearTimeout(timeout);
                reject(new Error(`Worker error: ${e.message}`));
            };
        });
    }
    
    /**
     * Direct モードで初期化
     */
    async _initDirectMode() {
        const wasm = await import('./chroma_transparent.js');
        await wasm.default();
        
        this.wasmModule = wasm;
        this.wasmParams = new wasm.WasmProcessParams();
        this.ready = true;
        this.version = wasm.getVersion();
    }
    
    /**
     * Workerからのメッセージを処理 (Workerモード用)
     */
    _handleWorkerMessage(e) {
        const { type, taskId, payload } = e.data;
        const task = this.pendingTasks.get(taskId);
        
        if (!task) return;
        
        this.pendingTasks.delete(taskId);
        
        if (type === 'result') {
            task.resolve(payload);
        } else if (type === 'error') {
            task.reject(new Error(payload.message));
        }
    }
    
    /**
     * Workerにタスクを送信 (Workerモード用)
     */
    _sendToWorker(type, payload) {
        return new Promise((resolve, reject) => {
            const taskId = ++this.taskId;
            
            this.pendingTasks.set(taskId, { resolve, reject });
            
            const transferables = [];
            let messagePayload = { ...payload };
            
            if (payload.imageData) {
                const copy = payload.imageData.slice(0);
                messagePayload.imageData = copy;
                transferables.push(copy);
            }
            
            this.worker.postMessage(
                { type, taskId, payload: messagePayload },
                transferables
            );
        });
    }
    
    /**
     * パラメータを共有メモリに書き込み (Sharedモード用)
     */
    _writeParamsToShared(params) {
        // Float32 パラメータ
        this.paramView[PARAM_TOLERANCE] = parseFloat(params.tolerance) || 0.3;
        this.paramView[PARAM_DESPILL] = parseFloat(params.despill) || 0.7;
        this.paramView[PARAM_FEATHER] = parseInt(params.feather, 10) || 5;
        this.paramView[PARAM_ERODE] = parseInt(params.erode, 10) || 0;
        this.paramView[PARAM_DILATE] = parseInt(params.dilate, 10) || 1;
        
        // 色文字列 (byte 128-191)
        const colorBytes = new TextEncoder().encode(String(params.color || 'lime'));
        const colorView = new Uint8Array(this.sharedBuffer, COLOR_OFFSET, COLOR_SIZE);
        colorView.fill(0);
        colorView.set(colorBytes.slice(0, COLOR_SIZE - 1));
    }
    
    /**
     * SharedArrayBuffer経由で処理 (Sharedモード用)
     */
    async _processShared(taskType, imageData, maxSize = 512) {
        // パラメータは呼び出し前に書き込み済み
        
        // 入力データを書き込み
        this.dataView.set(imageData, 0);
        Atomics.store(this.controlView, CTRL_INPUT_SIZE, imageData.length);
        Atomics.store(this.controlView, CTRL_MAX_SIZE, maxSize);
        
        // タスク開始
        Atomics.store(this.controlView, CTRL_TASK_TYPE, taskType);
        Atomics.store(this.controlView, CTRL_STATUS, STATUS_PROCESSING);
        Atomics.notify(this.controlView, CTRL_STATUS);
        
        // 完了を待機 (ポーリング)
        await this._waitForCompletion();
        
        // 結果を取得
        const status = Atomics.load(this.controlView, CTRL_STATUS);
        if (status === STATUS_ERROR) {
            throw new Error('Processing failed in worker');
        }
        
        const inputSize = Atomics.load(this.controlView, CTRL_INPUT_SIZE);
        const outputSize = Atomics.load(this.controlView, CTRL_OUTPUT_SIZE);
        
        // 出力データをコピー
        return this.dataView.slice(inputSize, inputSize + outputSize);
    }
    
    /**
     * 完了を待機 (Sharedモード用)
     */
    async _waitForCompletion() {
        return new Promise((resolve) => {
            const check = () => {
                const status = Atomics.load(this.controlView, CTRL_STATUS);
                if (status === STATUS_DONE || status === STATUS_ERROR) {
                    resolve();
                } else {
                    // ポーリング間隔を短めに
                    setTimeout(check, 5);
                }
            };
            check();
        });
    }
    
    /**
     * パラメータをWASMに同期（Directモード用）
     */
    _syncParams(params) {
        if (!this.wasmParams) return;
        
        if (params.color !== undefined) {
            this.wasmParams.setColor(String(params.color));
        }
        if (params.tolerance !== undefined) {
            this.wasmParams.setTolerance(parseFloat(params.tolerance));
        }
        if (params.feather !== undefined) {
            this.wasmParams.setFeather(parseInt(params.feather, 10));
        }
        if (params.despill !== undefined) {
            this.wasmParams.setDespill(parseFloat(params.despill));
        }
        if (params.erode !== undefined) {
            this.wasmParams.setErode(parseInt(params.erode, 10));
        }
        if (params.dilate !== undefined) {
            this.wasmParams.setDilate(parseInt(params.dilate, 10));
        }
        
        // 新規パラメータ
        if (params.colorSpace !== undefined) {
            this.wasmParams.setColorSpace(String(params.colorSpace));
        }
        if (params.bilateral !== undefined) {
            this.wasmParams.setBilateralEnabled(params.bilateral.enabled || false);
            if (params.bilateral.enabled) {
                if (params.bilateral.spatialSigma !== undefined) {
                    this.wasmParams.setBilateralSpatialSigma(parseFloat(params.bilateral.spatialSigma));
                }
                if (params.bilateral.colorSigma !== undefined) {
                    this.wasmParams.setBilateralColorSigma(parseFloat(params.bilateral.colorSigma));
                }
                if (params.bilateral.radius !== undefined) {
                    this.wasmParams.setBilateralRadius(parseInt(params.bilateral.radius, 10));
                }
            }
        }
        if (params.multiscale !== undefined) {
            this.wasmParams.setMultiscaleEnabled(params.multiscale.enabled || false);
            if (params.multiscale.enabled) {
                if (params.multiscale.levels !== undefined) {
                    this.wasmParams.setMultiscaleLevels(parseInt(params.multiscale.levels, 10));
                }
                if (params.multiscale.scaleFactor !== undefined) {
                    this.wasmParams.setMultiscaleScaleFactor(parseFloat(params.multiscale.scaleFactor));
                }
            }
        }
        if (params.edgeOptimization !== undefined) {
            this.wasmParams.setEdgeOptimizationEnabled(params.edgeOptimization.enabled || false);
            if (params.edgeOptimization.enabled) {
                if (params.edgeOptimization.threshold !== undefined) {
                    this.wasmParams.setEdgeThreshold(parseFloat(params.edgeOptimization.threshold));
                }
                if (params.edgeOptimization.smoothness !== undefined) {
                    this.wasmParams.setEdgeSmoothness(parseFloat(params.edgeOptimization.smoothness));
                }
            }
        }
        if (params.shadowRemoval !== undefined) {
            this.wasmParams.setShadowRemovalEnabled(params.shadowRemoval.enabled || false);
            if (params.shadowRemoval.enabled) {
                if (params.shadowRemoval.threshold !== undefined) {
                    this.wasmParams.setShadowThreshold(parseFloat(params.shadowRemoval.threshold));
                }
                if (params.shadowRemoval.strength !== undefined) {
                    this.wasmParams.setShadowRemovalStrength(parseFloat(params.shadowRemoval.strength));
                }
            }
        }
        if (params.sharpen !== undefined) {
            this.wasmParams.setSharpenEnabled(params.sharpen.enabled || false);
            if (params.sharpen.enabled) {
                if (params.sharpen.amount !== undefined) {
                    this.wasmParams.setSharpenAmount(parseFloat(params.sharpen.amount));
                }
                if (params.sharpen.radius !== undefined) {
                    this.wasmParams.setSharpenRadius(parseFloat(params.sharpen.radius));
                }
                if (params.sharpen.threshold !== undefined) {
                    this.wasmParams.setSharpenThreshold(parseFloat(params.sharpen.threshold));
                }
            }
        }
    }
    
    // === 統一API ===
    
    /**
     * プレビュー画像を生成
     */
    async processPreview(imageData, params, maxSize = 512) {
        if (!this.ready) {
            throw new Error('Processor not ready');
        }
        
        switch (this.mode) {
            case ProcessorMode.SHARED:
                this._writeParamsToShared(params);
                const sharedResult = await this._processShared(TASK_PREVIEW, imageData, maxSize);
                return new Blob([sharedResult], { type: 'image/png' });
                
            case ProcessorMode.WORKER:
                const workerResult = await this._sendToWorker('preview', {
                    imageData: imageData.buffer,
                    params,
                    maxSize
                });
                return new Blob([new Uint8Array(workerResult.data)], { type: 'image/png' });
                
            case ProcessorMode.DIRECT:
                this._syncParams(params);
                const directResult = this.wasmModule.processPreview(imageData, this.wasmParams, maxSize);
                return new Blob([directResult], { type: 'image/png' });
                
            default:
                throw new Error('Invalid processor mode');
        }
    }
    
    /**
     * フル画像を処理
     */
    async processImage(imageData, params) {
        if (!this.ready) {
            throw new Error('Processor not ready');
        }
        
        switch (this.mode) {
            case ProcessorMode.SHARED:
                this._writeParamsToShared(params);
                const sharedResult = await this._processShared(TASK_PROCESS, imageData);
                return new Blob([sharedResult], { type: 'image/png' });
                
            case ProcessorMode.WORKER:
                const workerResult = await this._sendToWorker('process', {
                    imageData: imageData.buffer,
                    params
                });
                return new Blob([new Uint8Array(workerResult.data)], { type: 'image/png' });
                
            case ProcessorMode.DIRECT:
                this._syncParams(params);
                const directResult = this.wasmModule.processImage(imageData, this.wasmParams);
                return new Blob([directResult], { type: 'image/png' });
                
            default:
                throw new Error('Invalid processor mode');
        }
    }
    
    /**
     * 画像情報を取得
     */
    async getImageInfo(imageData) {
        if (!this.ready) {
            throw new Error('Processor not ready');
        }
        
        // Shared/Worker モードはWorker経由
        if (this.mode === ProcessorMode.WORKER) {
            return await this._sendToWorker('info', {
                imageData: imageData.buffer
            });
        } else {
            // Direct モードはメインスレッドで
            return this.wasmModule.getImageInfo(imageData);
        }
    }
    
    /**
     * 現在のモードを取得
     */
    getMode() {
        return this.mode;
    }
    
    /**
     * モード表示用のラベル
     */
    getModeLabel() {
        switch (this.mode) {
            case ProcessorMode.SHARED: return 'SharedArrayBuffer';
            case ProcessorMode.WORKER: return 'Worker';
            case ProcessorMode.DIRECT: return 'Direct';
            default: return 'Unknown';
        }
    }
    
    /**
     * リソースを解放
     */
    dispose() {
        if (this.worker) {
            this.worker.terminate();
            this.worker = null;
        }
        this.sharedBuffer = null;
        this.controlView = null;
        this.paramView = null;
        this.dataView = null;
        this.wasmModule = null;
        this.wasmParams = null;
        this.ready = false;
        this.pendingTasks.clear();
    }
}

// グローバルプロセッサインスタンス
const processor = new ChromaProcessor();

// === メインアプリケーション ===

class ChromaApp {
    constructor() {
        this.processor = processor;
        this.initElements();
        this.initState();
        this.bindEvents();
        this.loadConfig();
    }

    initElements() {
        // Sections
        this.uploadSection = document.getElementById('upload-section');
        this.editorSection = document.getElementById('editor-section');
        
        // Upload (両方のファイル入力)
        this.dropzone = document.getElementById('dropzone');
        this.fileInput = document.getElementById('file-input');
        this.fileInputEditor = document.getElementById('file-input-editor');
        
        // Viewport
        this.viewport = document.getElementById('image-viewport');
        this.container = document.getElementById('image-container');
        this.originalImage = document.getElementById('original-image');
        this.previewImage = document.getElementById('preview-image');
        this.loadingOverlay = document.getElementById('loading-overlay');
        this.compareSlider = document.getElementById('compare-slider');
        
        // Toolbar
        this.zoomLevelEl = document.getElementById('zoom-level');
        this.modeBtns = document.querySelectorAll('.mode-btn');
        this.bgBtns = document.querySelectorAll('.bg-btn');
        this.bgColorPicker = document.getElementById('bg-color-picker');
        
        // Status
        this.statusFilename = document.getElementById('status-filename');
        this.statusDimensions = document.getElementById('status-dimensions');
        this.statusPosition = document.getElementById('status-position');
        
        // Resizer
        this.panelResizer = document.getElementById('panel-resizer');
        this.paramsPanel = document.getElementById('params-panel');
        
        // Controls
        this.colorSelect = document.getElementById('color-select');
        this.colorCustom = document.getElementById('color-custom');
        this.colorPreview = document.getElementById('color-preview');
        this.chromaColorPicker = document.getElementById('chroma-color-picker');
        this.toleranceSlider = document.getElementById('tolerance');
        this.featherSlider = document.getElementById('feather');
        this.despillSlider = document.getElementById('despill');
        this.erodeSlider = document.getElementById('erode');
        this.dilateSlider = document.getElementById('dilate');
        this.colorSpaceSelect = document.getElementById('color-space');
        
        // 新規コントロール
        this.multiColorEnabled = document.getElementById('multi-color-enabled');
        this.multiColorList = document.getElementById('multi-color-list');
        this.btnAddColor = document.getElementById('btn-add-color');
        
        this.bilateralEnabled = document.getElementById('bilateral-enabled');
        this.bilateralParams = document.getElementById('bilateral-params');
        this.bilateralSpatialSigma = document.getElementById('bilateral-spatial-sigma');
        this.bilateralColorSigma = document.getElementById('bilateral-color-sigma');
        this.bilateralRadius = document.getElementById('bilateral-radius');
        
        this.multiscaleEnabled = document.getElementById('multiscale-enabled');
        this.multiscaleParams = document.getElementById('multiscale-params');
        this.multiscaleLevels = document.getElementById('multiscale-levels');
        this.multiscaleScaleFactor = document.getElementById('multiscale-scale-factor');
        
        this.edgeOptimizationEnabled = document.getElementById('edge-optimization-enabled');
        this.edgeOptimizationParams = document.getElementById('edge-optimization-params');
        this.edgeThreshold = document.getElementById('edge-threshold');
        this.edgeSmoothness = document.getElementById('edge-smoothness');
        
        this.shadowRemovalEnabled = document.getElementById('shadow-removal-enabled');
        this.shadowRemovalParams = document.getElementById('shadow-removal-params');
        this.shadowThreshold = document.getElementById('shadow-threshold');
        this.shadowRemovalStrength = document.getElementById('shadow-removal-strength');
        
        this.sharpenEnabled = document.getElementById('sharpen-enabled');
        this.sharpenParams = document.getElementById('sharpen-params');
        this.sharpenAmount = document.getElementById('sharpen-amount');
        this.sharpenRadius = document.getElementById('sharpen-radius');
        this.sharpenThreshold = document.getElementById('sharpen-threshold');
        
        // Buttons
        this.resetBtn = document.getElementById('reset-btn');
        this.processBtn = document.getElementById('process-btn');
    }

    initState() {
        // Image state
        this.originalFile = null;
        this.originalImageData = null;
        this.imageWidth = 0;
        this.imageHeight = 0;
        
        // View state
        this.zoom = 1;
        this.minZoom = 0.1;
        this.maxZoom = 8;
        this.panX = 0;
        this.panY = 0;
        this.viewMode = 'compare';
        this.sliderRatio = 0.5;
        this.bgMode = 'checker';
        
        // Interaction state
        this.isDragging = false;
        this.isSliderDragging = false;
        this.isPanelResizing = false;
        this.dragStartX = 0;
        this.dragStartY = 0;
        this.panStartX = 0;
        this.panStartY = 0;
        
        // Debounce
        this.debounceTimer = null;
        this.isProcessing = false;
        
        // Defaults
        this.defaults = {
            color: 'lime',
            tolerance: 0.3,
            feather: 5,
            despill: 0.7,
            erode: 0,
            dilate: 1
        };
        
        // Color map
        this.colorMap = {
            lime: '#00ff00',
            green: '#008000',
            blue: '#0000ff',
            magenta: '#ff00ff',
            cyan: '#00ffff',
            red: '#ff0000',
            yellow: '#ffff00',
            white: '#ffffff',
            black: '#000000'
        };
    }

    bindEvents() {
        // File input (初期画面)
        this.dropzone.addEventListener('click', (e) => {
            if (e.target.tagName !== 'LABEL') this.fileInput.click();
        });
        this.dropzone.addEventListener('dragover', (e) => this.handleDragOver(e));
        this.dropzone.addEventListener('dragleave', () => this.dropzone.classList.remove('dragover'));
        this.dropzone.addEventListener('drop', (e) => this.handleDrop(e));
        this.fileInput.addEventListener('change', (e) => this.handleFileSelect(e));
        
        // File input (エディタ画面)
        this.fileInputEditor.addEventListener('change', (e) => this.handleFileSelect(e));
        
        // Viewport - Zoom (wheel)
        this.viewport.addEventListener('wheel', (e) => this.handleWheel(e), { passive: false });
        
        // Viewport - Pan (pointer)
        this.viewport.addEventListener('pointerdown', (e) => this.handleViewportPointerDown(e));
        document.addEventListener('pointermove', (e) => this.handlePointerMove(e));
        document.addEventListener('pointerup', (e) => this.handlePointerUp(e));
        
        // Compare slider
        this.compareSlider.addEventListener('pointerdown', (e) => this.handleSliderPointerDown(e));
        
        // Toolbar buttons
        document.getElementById('btn-zoom-out').addEventListener('click', () => this.zoomBy(-0.25));
        document.getElementById('btn-zoom-in').addEventListener('click', () => this.zoomBy(0.25));
        document.getElementById('btn-zoom-fit').addEventListener('click', () => this.zoomToFit());
        document.getElementById('btn-zoom-100').addEventListener('click', () => this.zoomTo(1));
        document.getElementById('btn-reset-view').addEventListener('click', () => this.resetView());
        document.getElementById('btn-fullscreen').addEventListener('click', () => this.toggleFullscreen());
        
        // Mode buttons
        this.modeBtns.forEach(btn => {
            btn.addEventListener('click', () => this.setViewMode(btn.dataset.mode));
        });
        
        // Background buttons
        this.bgBtns.forEach(btn => {
            btn.addEventListener('click', () => {
                if (btn.dataset.bg === 'custom') {
                    this.bgColorPicker.click();
                } else {
                    this.setBgMode(btn.dataset.bg);
                }
            });
        });
        this.bgColorPicker.addEventListener('input', (e) => {
            this.setBgMode('custom', e.target.value);
        });
        
        // Panel resizer
        this.panelResizer.addEventListener('pointerdown', (e) => this.handleResizerDown(e));
        
        // Keyboard
        document.addEventListener('keydown', (e) => this.handleKeyDown(e));
        
        // Controls
        this.colorSelect.addEventListener('change', () => this.handleColorChange());
        this.colorCustom.addEventListener('input', () => this.handleColorChange());
        
        // Chroma color picker
        this.chromaColorPicker.addEventListener('input', (e) => {
            this.colorSelect.value = 'custom';
            this.colorCustom.value = e.target.value;
            this.colorCustom.hidden = false;
            this.updateColorPreview();
            this.schedulePreview();
        });
        
        const sliders = [this.toleranceSlider, this.featherSlider, this.despillSlider, 
                        this.erodeSlider, this.dilateSlider];
        sliders.forEach(slider => {
            slider.addEventListener('input', () => {
                this.updateSliderValue(slider);
                this.schedulePreview();
            });
        });
        
        // Action buttons
        this.resetBtn.addEventListener('click', () => this.resetParams());
        this.processBtn.addEventListener('click', () => this.processAndDownload());
        
        // Window resize
        window.addEventListener('resize', () => this.updateCompareView());
        
        // Viewport drag & drop
        this.viewport.addEventListener('dragover', (e) => {
            e.preventDefault();
            e.stopPropagation();
        });
        this.viewport.addEventListener('drop', (e) => {
            e.preventDefault();
            e.stopPropagation();
            if (e.dataTransfer.files.length > 0) {
                this.loadFile(e.dataTransfer.files[0]);
            }
        });
    }

    async loadConfig() {
        try {
            await this.processor.init();
            const versionEl = document.getElementById('version');
            if (versionEl) {
                versionEl.textContent = `${this.processor.version} (${this.processor.getModeLabel()})`;
            }
        } catch (e) {
            console.error('Failed to initialize processor:', e);
            alert('WASMの初期化に失敗しました。ブラウザがWebAssemblyをサポートしているか確認してください。');
        }
        this.updateColorPreview();
        this.setBgMode('checker');
    }

    // === File Handling ===
    
    handleDragOver(e) {
        e.preventDefault();
        e.stopPropagation();
        this.dropzone.classList.add('dragover');
    }

    handleDrop(e) {
        e.preventDefault();
        e.stopPropagation();
        this.dropzone.classList.remove('dragover');
        if (e.dataTransfer.files.length > 0) {
            this.loadFile(e.dataTransfer.files[0]);
        }
    }

    handleFileSelect(e) {
        if (e.target.files.length > 0) {
            this.loadFile(e.target.files[0]);
            e.target.value = '';
        }
    }

    async loadFile(file) {
        if (!file.type.startsWith('image/')) {
            alert('画像ファイルを選択してください');
            return;
        }

        this.originalFile = file;
        
        const buffer = await file.arrayBuffer();
        this.originalImageData = new Uint8Array(buffer);

        const img = new Image();
        img.src = URL.createObjectURL(file);
        
        await new Promise(resolve => img.onload = resolve);
        
        this.imageWidth = img.naturalWidth;
        this.imageHeight = img.naturalHeight;
        
        this.originalImage.src = img.src;
        this.originalImage.style.width = this.imageWidth + 'px';
        this.originalImage.style.height = this.imageHeight + 'px';
        
        this.previewImage.style.width = this.imageWidth + 'px';
        this.previewImage.style.height = this.imageHeight + 'px';
        
        this.container.style.width = this.imageWidth + 'px';
        this.container.style.height = this.imageHeight + 'px';
        
        this.uploadSection.hidden = true;
        this.editorSection.hidden = false;
        
        this.statusFilename.textContent = file.name;
        this.statusDimensions.textContent = `${this.imageWidth} × ${this.imageHeight}`;
        
        this.sliderRatio = 0.5;
        
        requestAnimationFrame(() => {
            this.zoomToFit();
            this.updatePreview();
        });
    }

    // === Background Mode ===
    
    setBgMode(mode, customColor = null) {
        this.bgMode = mode;
        
        this.bgBtns.forEach(btn => {
            btn.classList.toggle('active', btn.dataset.bg === mode);
        });
        
        this.viewport.classList.remove('bg-checker', 'bg-white', 'bg-black');
        
        if (mode === 'checker') {
            this.viewport.classList.add('bg-checker');
            this.viewport.style.backgroundColor = '';
        } else if (mode === 'white') {
            this.viewport.classList.add('bg-white');
            this.viewport.style.backgroundColor = '';
        } else if (mode === 'black') {
            this.viewport.classList.add('bg-black');
            this.viewport.style.backgroundColor = '';
        } else if (mode === 'custom') {
            const color = customColor || this.bgColorPicker.value;
            this.viewport.style.backgroundColor = color;
            this.bgBtns.forEach(btn => {
                btn.classList.toggle('active', btn.dataset.bg === 'custom');
            });
        }
    }

    // === Zoom & Pan ===

    handleWheel(e) {
        e.preventDefault();
        
        const rect = this.viewport.getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;
        
        const delta = e.deltaY > 0 ? -0.1 : 0.1;
        const newZoom = Math.max(this.minZoom, Math.min(this.maxZoom, this.zoom * (1 + delta)));
        
        if (newZoom !== this.zoom) {
            const zoomRatio = newZoom / this.zoom;
            this.panX = mouseX - (mouseX - this.panX) * zoomRatio;
            this.panY = mouseY - (mouseY - this.panY) * zoomRatio;
            this.zoom = newZoom;
            this.updateTransform();
        }
    }

    handleViewportPointerDown(e) {
        if (e.target.closest('.compare-slider')) {
            return;
        }
        
        e.preventDefault();
        this.isDragging = true;
        this.dragStartX = e.clientX;
        this.dragStartY = e.clientY;
        this.panStartX = this.panX;
        this.panStartY = this.panY;
        this.viewport.classList.add('dragging');
        this.viewport.setPointerCapture(e.pointerId);
    }

    handleSliderPointerDown(e) {
        e.preventDefault();
        e.stopPropagation();
        this.isSliderDragging = true;
        this.compareSlider.setPointerCapture(e.pointerId);
        this.updateSliderFromEvent(e);
    }

    handlePointerMove(e) {
        if (this.imageWidth > 0 && this.viewport) {
            this.updateStatusPosition(e);
        }
        
        if (this.isDragging) {
            const dx = e.clientX - this.dragStartX;
            const dy = e.clientY - this.dragStartY;
            this.panX = this.panStartX + dx;
            this.panY = this.panStartY + dy;
            this.updateTransform();
        } else if (this.isSliderDragging) {
            this.updateSliderFromEvent(e);
        } else if (this.isPanelResizing) {
            this.updatePanelHeight(e);
        }
    }

    handlePointerUp(e) {
        if (this.isDragging) {
            this.viewport.classList.remove('dragging');
        }
        this.isDragging = false;
        this.isSliderDragging = false;
        this.isPanelResizing = false;
    }

    updateSliderFromEvent(e) {
        const rect = this.viewport.getBoundingClientRect();
        const x = e.clientX - rect.left;
        this.sliderRatio = Math.max(0, Math.min(1, x / rect.width));
        this.updateCompareView();
    }

    handleResizerDown(e) {
        this.isPanelResizing = true;
        this.resizerStartY = e.clientY;
        this.panelStartHeight = this.paramsPanel.offsetHeight;
        this.panelResizer.setPointerCapture(e.pointerId);
    }

    updatePanelHeight(e) {
        const dy = this.resizerStartY - e.clientY;
        const newHeight = Math.max(100, Math.min(400, this.panelStartHeight + dy));
        this.paramsPanel.style.height = newHeight + 'px';
    }

    zoomBy(delta) {
        const rect = this.viewport.getBoundingClientRect();
        const centerX = rect.width / 2;
        const centerY = rect.height / 2;
        
        const newZoom = Math.max(this.minZoom, Math.min(this.maxZoom, this.zoom + delta));
        
        if (newZoom !== this.zoom) {
            const zoomRatio = newZoom / this.zoom;
            this.panX = centerX - (centerX - this.panX) * zoomRatio;
            this.panY = centerY - (centerY - this.panY) * zoomRatio;
            this.zoom = newZoom;
            this.updateTransform();
        }
    }

    zoomTo(level) {
        const rect = this.viewport.getBoundingClientRect();
        const centerX = rect.width / 2;
        const centerY = rect.height / 2;
        
        const zoomRatio = level / this.zoom;
        this.panX = centerX - (centerX - this.panX) * zoomRatio;
        this.panY = centerY - (centerY - this.panY) * zoomRatio;
        this.zoom = level;
        this.updateTransform();
    }

    zoomToFit() {
        if (!this.imageWidth || !this.imageHeight) return;
        
        const rect = this.viewport.getBoundingClientRect();
        const scaleX = rect.width / this.imageWidth;
        const scaleY = rect.height / this.imageHeight;
        this.zoom = Math.min(scaleX, scaleY) * 0.95;
        
        this.panX = (rect.width - this.imageWidth * this.zoom) / 2;
        this.panY = (rect.height - this.imageHeight * this.zoom) / 2;
        
        this.updateTransform();
    }

    resetView() {
        this.zoomToFit();
        this.sliderRatio = 0.5;
        this.updateCompareView();
    }

    updateTransform() {
        this.container.style.transform = `translate(${this.panX}px, ${this.panY}px) scale(${this.zoom})`;
        this.zoomLevelEl.textContent = Math.round(this.zoom * 100) + '%';
        this.updateCompareView();
    }

    updateCompareView() {
        const rect = this.viewport.getBoundingClientRect();
        
        if (this.viewMode === 'compare') {
            const sliderScreenX = rect.width * this.sliderRatio;
            
            this.compareSlider.style.left = sliderScreenX + 'px';
            this.compareSlider.style.display = 'block';
            
            const sliderImageX = (sliderScreenX - this.panX) / this.zoom;
            
            const originalClipRight = Math.max(0, this.imageWidth - sliderImageX);
            this.originalImage.style.clipPath = `inset(0 ${originalClipRight}px 0 0)`;
            
            const previewClipLeft = Math.max(0, sliderImageX);
            this.previewImage.style.clipPath = `inset(0 0 0 ${previewClipLeft}px)`;
            
            this.originalImage.style.visibility = 'visible';
            this.previewImage.style.visibility = 'visible';
        } else {
            this.compareSlider.style.display = 'none';
            this.originalImage.style.clipPath = 'none';
            this.previewImage.style.clipPath = 'none';
            
            if (this.viewMode === 'original') {
                this.originalImage.style.visibility = 'visible';
                this.previewImage.style.visibility = 'hidden';
            } else {
                this.originalImage.style.visibility = 'hidden';
                this.previewImage.style.visibility = 'visible';
            }
        }
    }

    updateStatusPosition(e) {
        const rect = this.viewport.getBoundingClientRect();
        const x = Math.round((e.clientX - rect.left - this.panX) / this.zoom);
        const y = Math.round((e.clientY - rect.top - this.panY) / this.zoom);
        
        if (x >= 0 && x < this.imageWidth && y >= 0 && y < this.imageHeight) {
            this.statusPosition.textContent = `${x}, ${y}`;
        } else {
            this.statusPosition.textContent = '-';
        }
    }

    // === View Mode ===

    setViewMode(mode) {
        this.viewMode = mode;
        this.modeBtns.forEach(btn => {
            btn.classList.toggle('active', btn.dataset.mode === mode);
        });
        this.updateCompareView();
    }

    toggleFullscreen() {
        document.body.classList.toggle('fullscreen');
        
        if (document.body.classList.contains('fullscreen')) {
            if (document.documentElement.requestFullscreen) {
                document.documentElement.requestFullscreen();
            }
        } else {
            if (document.exitFullscreen) {
                document.exitFullscreen();
            }
        }
    }

    // === Keyboard ===

    handleKeyDown(e) {
        if (e.target.tagName === 'INPUT' || e.target.tagName === 'SELECT') return;
        
        switch(e.key) {
            case '+':
            case '=':
                this.zoomBy(0.25);
                break;
            case '-':
                this.zoomBy(-0.25);
                break;
            case '0':
                this.zoomToFit();
                break;
            case '1':
                this.zoomTo(1);
                break;
            case 'Tab':
                e.preventDefault();
                this.cycleViewMode();
                break;
            case 'f':
            case 'F':
                this.toggleFullscreen();
                break;
            case 'Enter':
                this.processAndDownload();
                break;
        }
    }

    cycleViewMode() {
        const modes = ['compare', 'original', 'preview'];
        const idx = modes.indexOf(this.viewMode);
        this.setViewMode(modes[(idx + 1) % modes.length]);
    }

    // === Controls ===

    handleColorChange() {
        const value = this.colorSelect.value;
        this.colorCustom.hidden = value !== 'custom';
        this.updateColorPreview();
        this.schedulePreview();
    }

    getColor() {
        if (this.colorSelect.value === 'custom') {
            return this.colorCustom.value.trim().replace('#', '') || 'lime';
        }
        return this.colorSelect.value;
    }

    getParams() {
        const params = {
            color: this.getColor(),
            tolerance: parseFloat(this.toleranceSlider.value),
            feather: parseInt(this.featherSlider.value, 10),
            despill: parseFloat(this.despillSlider.value),
            erode: parseInt(this.erodeSlider.value, 10),
            dilate: parseInt(this.dilateSlider.value, 10),
            colorSpace: this.colorSpaceSelect.value,
        };
        
        // 多色検出（WASM側では未サポートのため、UIのみ対応）
        // 将来的にWASM側でサポートされた際に使用可能
        if (this.multiColorEnabled.checked) {
            const multiColors = [];
            const multiColorItems = this.multiColorList.querySelectorAll('.multi-color-item');
            multiColorItems.forEach(item => {
                const colorInput = item.querySelector('.multi-color-color');
                const toleranceInput = item.querySelector('.multi-color-tolerance');
                if (colorInput && toleranceInput) {
                    multiColors.push({
                        color: colorInput.value,
                        tolerance: parseFloat(toleranceInput.value)
                    });
                }
            });
            if (multiColors.length > 0) {
                params.multiColors = multiColors;
                // 注意: WASM側では多色検出は未サポートのため、最初の色のみ使用
                console.warn('Multi-color detection is not yet supported in WASM mode. Using first color only.');
            }
        }
        
        // バイラテラルフィルタ
        if (this.bilateralEnabled.checked) {
            params.bilateral = {
                enabled: true,
                spatialSigma: parseFloat(this.bilateralSpatialSigma.value),
                colorSigma: parseFloat(this.bilateralColorSigma.value),
                radius: parseInt(this.bilateralRadius.value, 10)
            };
        }
        
        // マルチスケール処理
        if (this.multiscaleEnabled.checked) {
            params.multiscale = {
                enabled: true,
                levels: parseInt(this.multiscaleLevels.value, 10),
                scaleFactor: parseFloat(this.multiscaleScaleFactor.value)
            };
        }
        
        // マットエッジ最適化
        if (this.edgeOptimizationEnabled.checked) {
            params.edgeOptimization = {
                enabled: true,
                threshold: parseFloat(this.edgeThreshold.value),
                smoothness: parseFloat(this.edgeSmoothness.value)
            };
        }
        
        // 影の処理
        if (this.shadowRemovalEnabled.checked) {
            params.shadowRemoval = {
                enabled: true,
                threshold: parseFloat(this.shadowThreshold.value),
                strength: parseFloat(this.shadowRemovalStrength.value)
            };
        }
        
        // エッジシャープニング
        if (this.sharpenEnabled.checked) {
            params.sharpen = {
                enabled: true,
                amount: parseFloat(this.sharpenAmount.value),
                radius: parseFloat(this.sharpenRadius.value),
                threshold: parseFloat(this.sharpenThreshold.value)
            };
        }
        
        return params;
    }

    updateColorPreview() {
        const color = this.getColor();
        const hex = this.colorMap[color] || (color.startsWith('#') ? color : '#' + color);
        this.colorPreview.style.backgroundColor = hex;
        this.chromaColorPicker.value = hex.startsWith('#') ? hex : '#' + hex;
    }

    updateSliderValue(slider) {
        const valueSpan = document.getElementById(slider.id + '-value');
        if (valueSpan) {
            const value = parseFloat(slider.value);
            valueSpan.textContent = Number.isInteger(value) ? value : value.toFixed(2);
        }
    }

    schedulePreview() {
        clearTimeout(this.debounceTimer);
        this.debounceTimer = setTimeout(() => this.updatePreview(), 200);
    }

    async updatePreview() {
        if (!this.originalImageData || this.isProcessing) return;

        this.isProcessing = true;
        // プレビュー更新時はローディングアニメーションを表示しない（UX向上）

        try {
            const blob = await this.processor.processPreview(
                this.originalImageData,
                this.getParams(),
                512
            );
            
            if (this.previewImage.src.startsWith('blob:')) {
                URL.revokeObjectURL(this.previewImage.src);
            }
            
            this.previewImage.src = URL.createObjectURL(blob);
        } catch (e) {
            console.error('Preview error:', e);
        } finally {
            this.isProcessing = false;
        }
    }

    resetParams() {
        this.colorSelect.value = this.defaults.color;
        this.colorCustom.value = '';
        this.colorCustom.hidden = true;
        this.toleranceSlider.value = this.defaults.tolerance;
        this.featherSlider.value = this.defaults.feather;
        this.despillSlider.value = this.defaults.despill;
        this.erodeSlider.value = this.defaults.erode;
        this.dilateSlider.value = this.defaults.dilate;
        
        // 新規パラメータのリセット
        this.colorSpaceSelect.value = 'hsv';
        this.multiColorEnabled.checked = false;
        this.multiColorList.hidden = true;
        this.multiColorList.innerHTML = '';
        
        this.bilateralEnabled.checked = false;
        this.bilateralParams.hidden = true;
        this.bilateralSpatialSigma.value = 5.0;
        this.bilateralColorSigma.value = 50.0;
        this.bilateralRadius.value = 5;
        
        this.multiscaleEnabled.checked = false;
        this.multiscaleParams.hidden = true;
        this.multiscaleLevels.value = 3;
        this.multiscaleScaleFactor.value = 0.5;
        
        this.edgeOptimizationEnabled.checked = false;
        this.edgeOptimizationParams.hidden = true;
        this.edgeThreshold.value = 0.1;
        this.edgeSmoothness.value = 0.5;
        
        this.shadowRemovalEnabled.checked = false;
        this.shadowRemovalParams.hidden = true;
        this.shadowThreshold.value = 0.3;
        this.shadowRemovalStrength.value = 0.7;
        
        this.sharpenEnabled.checked = false;
        this.sharpenParams.hidden = true;
        this.sharpenAmount.value = 0.5;
        this.sharpenRadius.value = 1.0;
        this.sharpenThreshold.value = 0.0;

        [this.toleranceSlider, this.featherSlider, this.despillSlider, 
         this.erodeSlider, this.dilateSlider].forEach(s => this.updateSliderValue(s));
        
        // 新規スライダーの更新
        this.updateSliderValue(this.bilateralSpatialSigma);
        this.updateSliderValue(this.bilateralColorSigma);
        this.updateSliderValue(this.bilateralRadius);
        this.updateSliderValue(this.multiscaleLevels);
        this.updateSliderValue(this.multiscaleScaleFactor);
        this.updateSliderValue(this.edgeThreshold);
        this.updateSliderValue(this.edgeSmoothness);
        this.updateSliderValue(this.shadowThreshold);
        this.updateSliderValue(this.shadowRemovalStrength);
        this.updateSliderValue(this.sharpenAmount);
        this.updateSliderValue(this.sharpenRadius);
        this.updateSliderValue(this.sharpenThreshold);

        this.updateColorPreview();
        this.updatePreview();
    }
    
    addMultiColor() {
        const item = document.createElement('div');
        item.className = 'multi-color-item';
        item.innerHTML = `
            <input type="text" class="multi-color-color" placeholder="lime" value="lime">
            <input type="number" class="multi-color-tolerance" placeholder="0.3" value="0.3" min="0" max="1" step="0.01">
            <button type="button" class="btn-remove-color">×</button>
        `;
        
        const removeBtn = item.querySelector('.btn-remove-color');
        removeBtn.addEventListener('click', () => {
            item.remove();
            this.schedulePreview();
        });
        
        [item.querySelector('.multi-color-color'), item.querySelector('.multi-color-tolerance')].forEach(input => {
            input.addEventListener('input', () => this.schedulePreview());
        });
        
        this.multiColorList.appendChild(item);
        this.multiColorList.hidden = false;
        this.multiColorEnabled.checked = true;
    }

    async processAndDownload() {
        if (!this.originalImageData || this.isProcessing) return;

        this.isProcessing = true;
        this.processBtn.disabled = true;
        this.processBtn.textContent = '処理中...';
        this.loadingOverlay.hidden = false;

        try {
            const blob = await this.processor.processImage(
                this.originalImageData,
                this.getParams()
            );

            const url = URL.createObjectURL(blob);
            const link = document.createElement('a');
            link.href = url;
            link.download = (this.originalFile?.name?.replace(/\.[^.]+$/, '') || 'image') + '.chroma.png';
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
            URL.revokeObjectURL(url);

            this.processBtn.textContent = '✓ 完了';
            setTimeout(() => {
                this.processBtn.textContent = '処理してダウンロード';
                this.processBtn.disabled = false;
            }, 2000);

        } catch (e) {
            console.error('Process error:', e);
            alert('処理に失敗しました: ' + e.message);
            this.processBtn.textContent = '処理してダウンロード';
            this.processBtn.disabled = false;
        } finally {
            this.isProcessing = false;
            this.loadingOverlay.hidden = true;
        }
    }
}

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    window.app = new ChromaApp();
});

// Export for module usage
export { ChromaProcessor, processor, ProcessorMode };
