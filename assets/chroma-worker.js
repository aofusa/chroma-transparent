/**
 * chroma-transparent Web Worker
 * 
 * WASMクロマキー処理をバックグラウンドスレッドで実行
 * メインスレッドのUIブロックを回避
 */

// === WASM モジュール ===
let wasmModule = null;
let wasmParams = null;
let wasmReady = false;

/**
 * WASMモジュールを初期化
 */
async function initWasm(wasmUrl) {
    try {
        // Web Worker内でのES modules動的インポート
        // wasm-packが生成するモジュールをインポート
        const modulePath = wasmUrl || './chroma_transparent.js';
        
        // importScriptsは古い形式なので、動的importを使用
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
 * パラメータを設定
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
function processPreview(imageData, params, maxSize) {
    if (!wasmReady) {
        throw new Error('WASM not ready');
    }
    
    setParams(params);
    const result = wasmModule.processPreview(imageData, wasmParams, maxSize);
    return result;
}

/**
 * フル画像を処理
 */
function processImage(imageData, params) {
    if (!wasmReady) {
        throw new Error('WASM not ready');
    }
    
    setParams(params);
    const result = wasmModule.processImage(imageData, wasmParams);
    return result;
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

// === メッセージハンドラ ===

self.onmessage = async function(e) {
    const { type, taskId, payload } = e.data;
    
    try {
        switch (type) {
            case 'init': {
                const version = await initWasm(payload?.wasmUrl);
                self.postMessage({
                    type: 'ready',
                    taskId,
                    payload: { version }
                });
                break;
            }
            
            case 'preview': {
                const { imageData, params, maxSize } = payload;
                const result = processPreview(
                    new Uint8Array(imageData),
                    params,
                    maxSize || 512
                );
                
                // Transferableオブジェクトとして転送（コピーを避ける）
                self.postMessage({
                    type: 'result',
                    taskId,
                    payload: { data: result.buffer }
                }, [result.buffer]);
                break;
            }
            
            case 'process': {
                const { imageData, params } = payload;
                const result = processImage(
                    new Uint8Array(imageData),
                    params
                );
                
                // Transferableオブジェクトとして転送
                self.postMessage({
                    type: 'result',
                    taskId,
                    payload: { data: result.buffer }
                }, [result.buffer]);
                break;
            }
            
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

