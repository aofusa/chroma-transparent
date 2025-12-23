/**
 * chroma-transparent Web Worker
 * 
 * 2つのモードをサポート:
 * 1. SharedArrayBuffer モード - 共有メモリで高速転送
 * 2. Worker モード - コピー転送（フォールバック）
 */

// === 定数 (wasm.js と同じ) ===
const SHARED_CONTROL_SIZE = 256;

// 制御ブロックオフセット (Int32)
const CTRL_STATUS = 0;
const CTRL_TASK_TYPE = 1;
const CTRL_INPUT_SIZE = 2;
const CTRL_OUTPUT_SIZE = 3;
const CTRL_MAX_SIZE = 4;
const CTRL_ERROR = 5;

// 状態定数
const STATUS_IDLE = 0;
const STATUS_PROCESSING = 1;
const STATUS_DONE = 2;
const STATUS_ERROR = 3;

// タスク種別
const TASK_PREVIEW = 1;
const TASK_PROCESS = 2;

// パラメータオフセット
const PARAM_TOLERANCE = 0;
const PARAM_DESPILL = 1;
const PARAM_FEATHER = 2;
const PARAM_ERODE = 3;
const PARAM_DILATE = 4;

const COLOR_OFFSET = 128;
const COLOR_SIZE = 64;
const DATA_OFFSET = SHARED_CONTROL_SIZE;

// === WASM モジュール ===
let wasmModule = null;
let wasmParams = null;
let wasmReady = false;

// === SharedArrayBuffer モード用 ===
let sharedBuffer = null;
let controlView = null;
let paramView = null;
let dataView = null;
let isSharedMode = false;

/**
 * WASMモジュールを初期化
 */
async function initWasm(wasmUrl) {
    try {
        const modulePath = wasmUrl || './chroma_transparent.js';
        const wasm = await import(modulePath);
        await wasm.default();
        
        wasmModule = wasm;
        wasmParams = new wasm.WasmProcessParams();
        wasmReady = true;
        
        return wasm.getVersion();
    } catch (error) {
        throw new Error(`WASM initialization failed: ${error.message}`);
    }
}

/**
 * SharedArrayBuffer モードを初期化
 */
function initSharedMode(buffer) {
    sharedBuffer = buffer;
    controlView = new Int32Array(sharedBuffer, 0, 16);
    paramView = new Float32Array(sharedBuffer, 64, 8);
    dataView = new Uint8Array(sharedBuffer, DATA_OFFSET);
    isSharedMode = true;
}

/**
 * 共有メモリからパラメータを読み取り
 */
function readParamsFromShared() {
    if (!wasmParams) return;
    
    // Float32 パラメータ
    wasmParams.setTolerance(paramView[PARAM_TOLERANCE]);
    wasmParams.setDespill(paramView[PARAM_DESPILL]);
    wasmParams.setFeather(Math.floor(paramView[PARAM_FEATHER]));
    wasmParams.setErode(Math.floor(paramView[PARAM_ERODE]));
    wasmParams.setDilate(Math.floor(paramView[PARAM_DILATE]));
    
    // 色文字列
    const colorBytes = new Uint8Array(sharedBuffer, COLOR_OFFSET, COLOR_SIZE);
    let colorEnd = colorBytes.indexOf(0);
    if (colorEnd === -1) colorEnd = COLOR_SIZE;
    const color = new TextDecoder().decode(colorBytes.slice(0, colorEnd));
    wasmParams.setColor(color || 'lime');
}

/**
 * パラメータを設定（Workerモード用）
 */
function setParams(params) {
    if (!wasmParams) return;
    
    if (params.color !== undefined) {
        wasmParams.setColor(String(params.color));
    }
    if (params.tolerance !== undefined) {
        wasmParams.setTolerance(parseFloat(params.tolerance));
    }
    if (params.feather !== undefined) {
        wasmParams.setFeather(parseInt(params.feather, 10));
    }
    if (params.despill !== undefined) {
        wasmParams.setDespill(parseFloat(params.despill));
    }
    if (params.erode !== undefined) {
        wasmParams.setErode(parseInt(params.erode, 10));
    }
    if (params.dilate !== undefined) {
        wasmParams.setDilate(parseInt(params.dilate, 10));
    }
}

/**
 * プレビュー画像を生成
 */
function processPreview(imageData, maxSize) {
    if (!wasmReady) {
        throw new Error('WASM not ready');
    }
    
    return wasmModule.processPreview(imageData, wasmParams, maxSize);
}

/**
 * フル画像を処理
 */
function processImage(imageData) {
    if (!wasmReady) {
        throw new Error('WASM not ready');
    }
    
    return wasmModule.processImage(imageData, wasmParams);
}

/**
 * 画像情報を取得
 */
function getImageInfo(imageData) {
    if (!wasmReady) {
        throw new Error('WASM not ready');
    }
    
    return wasmModule.getImageInfo(imageData);
}

/**
 * SharedArrayBuffer モードのメインループ
 */
async function sharedModeLoop() {
    while (isSharedMode) {
        // タスク待機
        const status = Atomics.load(controlView, CTRL_STATUS);
        
        if (status === STATUS_PROCESSING) {
            try {
                // パラメータを読み取り
                readParamsFromShared();
                
                const taskType = Atomics.load(controlView, CTRL_TASK_TYPE);
                const inputSize = Atomics.load(controlView, CTRL_INPUT_SIZE);
                const maxSize = Atomics.load(controlView, CTRL_MAX_SIZE);
                
                // 入力データを取得
                const inputData = dataView.slice(0, inputSize);
                
                // 処理実行
                let result;
                if (taskType === TASK_PREVIEW) {
                    result = processPreview(inputData, maxSize || 512);
                } else if (taskType === TASK_PROCESS) {
                    result = processImage(inputData);
                } else {
                    throw new Error(`Unknown task type: ${taskType}`);
                }
                
                // 結果を書き込み
                dataView.set(result, inputSize);
                Atomics.store(controlView, CTRL_OUTPUT_SIZE, result.length);
                
                // 完了を通知
                Atomics.store(controlView, CTRL_STATUS, STATUS_DONE);
                Atomics.notify(controlView, CTRL_STATUS);
                
            } catch (e) {
                console.error('SharedMode processing error:', e);
                Atomics.store(controlView, CTRL_ERROR, 1);
                Atomics.store(controlView, CTRL_STATUS, STATUS_ERROR);
                Atomics.notify(controlView, CTRL_STATUS);
            }
        } else {
            // ポーリング間隔
            await new Promise(resolve => setTimeout(resolve, 1));
        }
    }
}

// === メッセージハンドラ ===

self.onmessage = async function(e) {
    const { type, taskId, payload } = e.data;
    
    try {
        switch (type) {
            // 通常の Worker モード初期化
            case 'init': {
                const version = await initWasm(payload?.wasmUrl);
                self.postMessage({
                    type: 'ready',
                    taskId,
                    payload: { version }
                });
                break;
            }
            
            // SharedArrayBuffer モード初期化
            case 'init-shared': {
                const version = await initWasm(payload?.wasmUrl);
                initSharedMode(payload.sharedBuffer);
                
                self.postMessage({
                    type: 'ready',
                    taskId,
                    payload: { version }
                });
                
                // 共有メモリモードのループを開始
                sharedModeLoop();
                break;
            }
            
            // Worker モード: プレビュー
            case 'preview': {
                const { imageData, params, maxSize } = payload;
                setParams(params);
                const result = processPreview(
                    new Uint8Array(imageData),
                    maxSize || 512
                );
                
                self.postMessage({
                    type: 'result',
                    taskId,
                    payload: { data: result.buffer }
                }, [result.buffer]);
                break;
            }
            
            // Worker モード: フル処理
            case 'process': {
                const { imageData, params } = payload;
                setParams(params);
                const result = processImage(new Uint8Array(imageData));
                
                self.postMessage({
                    type: 'result',
                    taskId,
                    payload: { data: result.buffer }
                }, [result.buffer]);
                break;
            }
            
            // Worker モード: 画像情報
            case 'info': {
                const { imageData } = payload;
                const info = getImageInfo(new Uint8Array(imageData));
                
                self.postMessage({
                    type: 'result',
                    taskId,
                    payload: { 
                        width: info.width,
                        height: info.height 
                    }
                });
                break;
            }
            
            default:
                throw new Error(`Unknown message type: ${type}`);
        }
    } catch (error) {
        self.postMessage({
            type: 'error',
            taskId,
            payload: { message: error.message }
        });
    }
};

// Worker準備完了を通知
self.postMessage({ type: 'loaded' });
