let wasm;

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } } );

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}
/**
 * パニック時のスタックトレースをconsoleに出力
 */
export function init() {
    wasm.init();
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_export_3.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}
/**
 * 画像をクロマキー処理
 *
 * # Arguments
 * * `image_data` - 入力画像データ（PNG/JPEG/GIF/WebPなど）
 * * `params` - 処理パラメータ
 *
 * # Returns
 * 処理後のPNG画像データ
 * @param {Uint8Array} image_data
 * @param {WasmProcessParams} params
 * @returns {Uint8Array}
 */
export function processImage(image_data, params) {
    const ptr0 = passArray8ToWasm0(image_data, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    _assertClass(params, WasmProcessParams);
    const ret = wasm.processImage(ptr0, len0, params.__wbg_ptr);
    if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
    }
    var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v2;
}

/**
 * プレビュー画像を生成（縮小版）
 *
 * # Arguments
 * * `image_data` - 入力画像データ
 * * `params` - 処理パラメータ
 * * `max_size` - 最大サイズ（幅または高さ）
 *
 * # Returns
 * 処理後のPNG画像データ（縮小版）
 * @param {Uint8Array} image_data
 * @param {WasmProcessParams} params
 * @param {number} max_size
 * @returns {Uint8Array}
 */
export function processPreview(image_data, params, max_size) {
    const ptr0 = passArray8ToWasm0(image_data, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    _assertClass(params, WasmProcessParams);
    const ret = wasm.processPreview(ptr0, len0, params.__wbg_ptr, max_size);
    if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
    }
    var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v2;
}

/**
 * 画像情報を取得
 *
 * # Arguments
 * * `image_data` - 入力画像データ
 *
 * # Returns
 * 画像情報（幅、高さ）
 * @param {Uint8Array} image_data
 * @returns {ImageInfo}
 */
export function getImageInfo(image_data) {
    const ptr0 = passArray8ToWasm0(image_data, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.getImageInfo(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return ImageInfo.__wrap(ret[0]);
}

/**
 * バージョン情報を取得
 * @returns {string}
 */
export function getVersion() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.getVersion();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_export_3.get(mem.getUint32(i, true)));
    }
    wasm.__externref_drop_slice(ptr, len);
    return result;
}
/**
 * 利用可能な色名のリストを取得
 * @returns {any[]}
 */
export function getAvailableColors() {
    const ret = wasm.getAvailableColors();
    var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
    return v1;
}

const ImageInfoFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_imageinfo_free(ptr >>> 0, 1));
/**
 * 画像情報
 */
export class ImageInfo {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ImageInfo.prototype);
        obj.__wbg_ptr = ptr;
        ImageInfoFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ImageInfoFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_imageinfo_free(ptr, 0);
    }
    /**
     * 幅を取得
     * @returns {number}
     */
    get width() {
        const ret = wasm.imageinfo_width(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * 高さを取得
     * @returns {number}
     */
    get height() {
        const ret = wasm.imageinfo_height(this.__wbg_ptr);
        return ret >>> 0;
    }
}

const WasmProcessParamsFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmprocessparams_free(ptr >>> 0, 1));
/**
 * 処理パラメータ
 */
export class WasmProcessParams {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmProcessParamsFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmprocessparams_free(ptr, 0);
    }
    /**
     * 新しいパラメータを作成（デフォルト値）
     */
    constructor() {
        const ret = wasm.wasmprocessparams_new();
        this.__wbg_ptr = ret >>> 0;
        WasmProcessParamsFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * 色を設定
     * @param {string} color
     */
    setColor(color) {
        const ptr0 = passStringToWasm0(color, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.wasmprocessparams_setColor(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * 色を取得
     * @returns {string}
     */
    getColor() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmprocessparams_getColor(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * 許容範囲を設定
     * @param {number} v
     */
    setTolerance(v) {
        wasm.wasmprocessparams_setTolerance(this.__wbg_ptr, v);
    }
    /**
     * 許容範囲を取得
     * @returns {number}
     */
    getTolerance() {
        const ret = wasm.wasmprocessparams_getTolerance(this.__wbg_ptr);
        return ret;
    }
    /**
     * フェザリング量を設定
     * @param {number} v
     */
    setFeather(v) {
        wasm.wasmprocessparams_setFeather(this.__wbg_ptr, v);
    }
    /**
     * フェザリング量を取得
     * @returns {number}
     */
    getFeather() {
        const ret = wasm.wasmprocessparams_getFeather(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * デスピル強度を設定
     * @param {number} v
     */
    setDespill(v) {
        wasm.wasmprocessparams_setDespill(this.__wbg_ptr, v);
    }
    /**
     * デスピル強度を取得
     * @returns {number}
     */
    getDespill() {
        const ret = wasm.wasmprocessparams_getDespill(this.__wbg_ptr);
        return ret;
    }
    /**
     * 収縮回数を設定
     * @param {number} v
     */
    setErode(v) {
        wasm.wasmprocessparams_setErode(this.__wbg_ptr, v);
    }
    /**
     * 収縮回数を取得
     * @returns {number}
     */
    getErode() {
        const ret = wasm.wasmprocessparams_getErode(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * 膨張回数を設定
     * @param {number} v
     */
    setDilate(v) {
        wasm.wasmprocessparams_setDilate(this.__wbg_ptr, v);
    }
    /**
     * 膨張回数を取得
     * @returns {number}
     */
    getDilate() {
        const ret = wasm.wasmprocessparams_getDilate(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * パラメータをリセット
     */
    reset() {
        wasm.wasmprocessparams_reset(this.__wbg_ptr);
    }
    /**
     * @param {string} v
     */
    setColorSpace(v) {
        const ptr0 = passStringToWasm0(v, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.wasmprocessparams_setColorSpace(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    getColorSpace() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmprocessparams_getColorSpace(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {boolean} v
     */
    setBilateralEnabled(v) {
        wasm.wasmprocessparams_setBilateralEnabled(this.__wbg_ptr, v);
    }
    /**
     * @returns {boolean}
     */
    getBilateralEnabled() {
        const ret = wasm.wasmprocessparams_getBilateralEnabled(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {number} v
     */
    setBilateralSpatialSigma(v) {
        wasm.wasmprocessparams_setBilateralSpatialSigma(this.__wbg_ptr, v);
    }
    /**
     * @returns {number}
     */
    getBilateralSpatialSigma() {
        const ret = wasm.wasmprocessparams_getBilateralSpatialSigma(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} v
     */
    setBilateralColorSigma(v) {
        wasm.wasmprocessparams_setBilateralColorSigma(this.__wbg_ptr, v);
    }
    /**
     * @returns {number}
     */
    getBilateralColorSigma() {
        const ret = wasm.wasmprocessparams_getBilateralColorSigma(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} v
     */
    setBilateralRadius(v) {
        wasm.wasmprocessparams_setBilateralRadius(this.__wbg_ptr, v);
    }
    /**
     * @returns {number}
     */
    getBilateralRadius() {
        const ret = wasm.wasmprocessparams_getBilateralRadius(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {boolean} v
     */
    setMultiscaleEnabled(v) {
        wasm.wasmprocessparams_setMultiscaleEnabled(this.__wbg_ptr, v);
    }
    /**
     * @returns {boolean}
     */
    getMultiscaleEnabled() {
        const ret = wasm.wasmprocessparams_getMultiscaleEnabled(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {number} v
     */
    setMultiscaleLevels(v) {
        wasm.wasmprocessparams_setMultiscaleLevels(this.__wbg_ptr, v);
    }
    /**
     * @returns {number}
     */
    getMultiscaleLevels() {
        const ret = wasm.wasmprocessparams_getMultiscaleLevels(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} v
     */
    setMultiscaleScaleFactor(v) {
        wasm.wasmprocessparams_setMultiscaleScaleFactor(this.__wbg_ptr, v);
    }
    /**
     * @returns {number}
     */
    getMultiscaleScaleFactor() {
        const ret = wasm.wasmprocessparams_getMultiscaleScaleFactor(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {boolean} v
     */
    setEdgeOptimizationEnabled(v) {
        wasm.wasmprocessparams_setEdgeOptimizationEnabled(this.__wbg_ptr, v);
    }
    /**
     * @returns {boolean}
     */
    getEdgeOptimizationEnabled() {
        const ret = wasm.wasmprocessparams_getEdgeOptimizationEnabled(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {number} v
     */
    setEdgeThreshold(v) {
        wasm.wasmprocessparams_setEdgeThreshold(this.__wbg_ptr, v);
    }
    /**
     * @returns {number}
     */
    getEdgeThreshold() {
        const ret = wasm.wasmprocessparams_getEdgeThreshold(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} v
     */
    setEdgeSmoothness(v) {
        wasm.wasmprocessparams_setEdgeSmoothness(this.__wbg_ptr, v);
    }
    /**
     * @returns {number}
     */
    getEdgeSmoothness() {
        const ret = wasm.wasmprocessparams_getEdgeSmoothness(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {boolean} v
     */
    setShadowRemovalEnabled(v) {
        wasm.wasmprocessparams_setShadowRemovalEnabled(this.__wbg_ptr, v);
    }
    /**
     * @returns {boolean}
     */
    getShadowRemovalEnabled() {
        const ret = wasm.wasmprocessparams_getShadowRemovalEnabled(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {number} v
     */
    setShadowThreshold(v) {
        wasm.wasmprocessparams_setShadowThreshold(this.__wbg_ptr, v);
    }
    /**
     * @returns {number}
     */
    getShadowThreshold() {
        const ret = wasm.wasmprocessparams_getShadowThreshold(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} v
     */
    setShadowRemovalStrength(v) {
        wasm.wasmprocessparams_setShadowRemovalStrength(this.__wbg_ptr, v);
    }
    /**
     * @returns {number}
     */
    getShadowRemovalStrength() {
        const ret = wasm.wasmprocessparams_getShadowRemovalStrength(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {boolean} v
     */
    setSharpenEnabled(v) {
        wasm.wasmprocessparams_setSharpenEnabled(this.__wbg_ptr, v);
    }
    /**
     * @returns {boolean}
     */
    getSharpenEnabled() {
        const ret = wasm.wasmprocessparams_getSharpenEnabled(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {number} v
     */
    setSharpenAmount(v) {
        wasm.wasmprocessparams_setSharpenAmount(this.__wbg_ptr, v);
    }
    /**
     * @returns {number}
     */
    getSharpenAmount() {
        const ret = wasm.wasmprocessparams_getSharpenAmount(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} v
     */
    setSharpenRadius(v) {
        wasm.wasmprocessparams_setSharpenRadius(this.__wbg_ptr, v);
    }
    /**
     * @returns {number}
     */
    getSharpenRadius() {
        const ret = wasm.wasmprocessparams_getSharpenRadius(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} v
     */
    setSharpenThreshold(v) {
        wasm.wasmprocessparams_setSharpenThreshold(this.__wbg_ptr, v);
    }
    /**
     * @returns {number}
     */
    getSharpenThreshold() {
        const ret = wasm.wasmprocessparams_getSharpenThreshold(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {boolean} v
     */
    setAdaptiveToleranceEnabled(v) {
        wasm.wasmprocessparams_setAdaptiveToleranceEnabled(this.__wbg_ptr, v);
    }
    /**
     * @returns {boolean}
     */
    getAdaptiveToleranceEnabled() {
        const ret = wasm.wasmprocessparams_getAdaptiveToleranceEnabled(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {number} v
     */
    setAdaptiveToleranceGridW(v) {
        wasm.wasmprocessparams_setAdaptiveToleranceGridW(this.__wbg_ptr, v);
    }
    /**
     * @returns {number}
     */
    getAdaptiveToleranceGridW() {
        const ret = wasm.wasmprocessparams_getAdaptiveToleranceGridW(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} v
     */
    setAdaptiveToleranceGridH(v) {
        wasm.wasmprocessparams_setAdaptiveToleranceGridH(this.__wbg_ptr, v);
    }
    /**
     * @returns {number}
     */
    getAdaptiveToleranceGridH() {
        const ret = wasm.wasmprocessparams_getAdaptiveToleranceGridH(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} v
     */
    setAdaptiveToleranceSensitivity(v) {
        wasm.wasmprocessparams_setAdaptiveToleranceSensitivity(this.__wbg_ptr, v);
    }
    /**
     * @returns {number}
     */
    getAdaptiveToleranceSensitivity() {
        const ret = wasm.wasmprocessparams_getAdaptiveToleranceSensitivity(this.__wbg_ptr);
        return ret;
    }
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_error_7534b8e9a36f1ab4 = function(arg0, arg1) {
        let deferred0_0;
        let deferred0_1;
        try {
            deferred0_0 = arg0;
            deferred0_1 = arg1;
            console.error(getStringFromWasm0(arg0, arg1));
        } finally {
            wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
        }
    };
    imports.wbg.__wbg_new_8a6f238a6ece86ea = function() {
        const ret = new Error();
        return ret;
    };
    imports.wbg.__wbg_stack_0ed75d68575b0f3c = function(arg0, arg1) {
        const ret = arg1.stack;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_export_3;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
        ;
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };

    return imports;
}

function __wbg_init_memory(imports, memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('chroma_transparent_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
