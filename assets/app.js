/**
 * chroma-transparent Web UI
 * Advanced Compare View with Zoom/Pan/Slider
 */

class ChromaApp {
    constructor() {
        this.initElements();
        this.initState();
        this.bindEvents();
        this.loadConfig();
    }

    initElements() {
        // Sections
        this.uploadSection = document.getElementById('upload-section');
        this.editorSection = document.getElementById('editor-section');
        
        // Upload
        this.dropzone = document.getElementById('dropzone');
        this.fileInput = document.getElementById('file-input');
        
        // Viewport
        this.viewport = document.getElementById('image-viewport');
        this.container = document.getElementById('image-container');
        this.originalLayer = document.getElementById('original-layer');
        this.previewLayer = document.getElementById('preview-layer');
        this.originalImage = document.getElementById('original-image');
        this.previewImage = document.getElementById('preview-image');
        this.loadingOverlay = document.getElementById('loading-overlay');
        this.compareSlider = document.getElementById('compare-slider');
        
        // Toolbar
        this.zoomLevelEl = document.getElementById('zoom-level');
        this.modeBtns = document.querySelectorAll('.mode-btn');
        
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
        this.toleranceSlider = document.getElementById('tolerance');
        this.featherSlider = document.getElementById('feather');
        this.despillSlider = document.getElementById('despill');
        this.erodeSlider = document.getElementById('erode');
        this.dilateSlider = document.getElementById('dilate');
        
        // Buttons
        this.resetBtn = document.getElementById('reset-btn');
        this.processBtn = document.getElementById('process-btn');
    }

    initState() {
        // Image state
        this.originalFile = null;
        this.resizedBlob = null;
        this.imageWidth = 0;
        this.imageHeight = 0;
        
        // View state
        this.zoom = 1;
        this.minZoom = 0.1;
        this.maxZoom = 8;
        this.panX = 0;
        this.panY = 0;
        this.viewMode = 'compare'; // 'compare' | 'original' | 'preview'
        this.sliderPosition = 0.5; // 0.0 - 1.0
        
        // Interaction state
        this.isDragging = false;
        this.isSliderDragging = false;
        this.isPanelResizing = false;
        this.dragStartX = 0;
        this.dragStartY = 0;
        this.panStartX = 0;
        this.panStartY = 0;
        this.spacePressed = false;
        
        // Debounce
        this.debounceTimer = null;
        this.abortController = null;
        
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
            yellow: '#ffff00'
        };
    }

    bindEvents() {
        // File input
        this.dropzone.addEventListener('click', (e) => {
            if (e.target.tagName !== 'LABEL') this.fileInput.click();
        });
        this.dropzone.addEventListener('dragover', (e) => this.handleDragOver(e));
        this.dropzone.addEventListener('dragleave', () => this.dropzone.classList.remove('dragover'));
        this.dropzone.addEventListener('drop', (e) => this.handleDrop(e));
        this.fileInput.addEventListener('change', (e) => this.handleFileSelect(e));
        
        // Viewport - Zoom
        this.viewport.addEventListener('wheel', (e) => this.handleWheel(e), { passive: false });
        
        // Viewport - Pan
        this.viewport.addEventListener('pointerdown', (e) => this.handlePointerDown(e));
        document.addEventListener('pointermove', (e) => this.handlePointerMove(e));
        document.addEventListener('pointerup', (e) => this.handlePointerUp(e));
        
        // Compare slider
        this.compareSlider.addEventListener('pointerdown', (e) => this.handleSliderDown(e));
        
        // Toolbar buttons
        document.getElementById('btn-zoom-out').addEventListener('click', () => this.zoomBy(-0.25));
        document.getElementById('btn-zoom-in').addEventListener('click', () => this.zoomBy(0.25));
        document.getElementById('btn-zoom-fit').addEventListener('click', () => this.zoomToFit());
        document.getElementById('btn-zoom-100').addEventListener('click', () => this.zoomTo(1));
        document.getElementById('btn-reset-view').addEventListener('click', () => this.resetView());
        document.getElementById('btn-fullscreen').addEventListener('click', () => this.toggleFullscreen());
        document.getElementById('btn-new-image').addEventListener('click', () => this.fileInput.click());
        
        // Mode buttons
        this.modeBtns.forEach(btn => {
            btn.addEventListener('click', () => this.setViewMode(btn.dataset.mode));
        });
        
        // Panel resizer
        this.panelResizer.addEventListener('pointerdown', (e) => this.handleResizerDown(e));
        
        // Keyboard
        document.addEventListener('keydown', (e) => this.handleKeyDown(e));
        document.addEventListener('keyup', (e) => this.handleKeyUp(e));
        
        // Controls
        this.colorSelect.addEventListener('change', () => this.handleColorChange());
        this.colorCustom.addEventListener('input', () => this.handleColorChange());
        
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
        window.addEventListener('resize', () => this.updateLayout());
    }

    async loadConfig() {
        try {
            const res = await fetch('/api/health');
            const health = await res.json();
            document.getElementById('version').textContent = health.version || '0.1.0';
        } catch (e) {
            console.warn('Failed to load config:', e);
        }
        this.updateColorPreview();
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
        }
    }

    async loadFile(file) {
        if (!file.type.startsWith('image/')) {
            alert('画像ファイルを選択してください');
            return;
        }

        this.originalFile = file;

        // Load image to get dimensions
        const img = new Image();
        img.src = URL.createObjectURL(file);
        
        await new Promise(resolve => img.onload = resolve);
        
        this.imageWidth = img.naturalWidth;
        this.imageHeight = img.naturalHeight;
        
        // Set original image
        this.originalImage.src = img.src;
        
        // Resize for preview
        this.resizedBlob = await this.resizeImage(file, 512);
        
        // Switch to editor
        this.uploadSection.hidden = true;
        this.editorSection.hidden = false;
        
        // Update status
        this.statusFilename.textContent = file.name;
        this.statusDimensions.textContent = `${this.imageWidth} × ${this.imageHeight}`;
        
        // Fit to view
        requestAnimationFrame(() => {
            this.zoomToFit();
            this.updatePreview();
        });
    }

    async resizeImage(file, maxSize) {
        return new Promise((resolve) => {
            const img = new Image();
            img.onload = () => {
                const canvas = document.createElement('canvas');
                let { width, height } = { width: img.naturalWidth, height: img.naturalHeight };
                
                if (width > maxSize || height > maxSize) {
                    const ratio = maxSize / Math.max(width, height);
                    width = Math.round(width * ratio);
                    height = Math.round(height * ratio);
                }
                
                canvas.width = width;
                canvas.height = height;
                canvas.getContext('2d').drawImage(img, 0, 0, width, height);
                
                canvas.toBlob((blob) => {
                    URL.revokeObjectURL(img.src);
                    resolve(blob);
                }, 'image/png');
            };
            img.src = URL.createObjectURL(file);
        });
    }

    // === Zoom & Pan ===

    handleWheel(e) {
        e.preventDefault();
        
        const rect = this.viewport.getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;
        
        // Zoom centered on mouse position
        const delta = e.deltaY > 0 ? -0.1 : 0.1;
        const newZoom = Math.max(this.minZoom, Math.min(this.maxZoom, this.zoom * (1 + delta)));
        
        if (newZoom !== this.zoom) {
            // Adjust pan to zoom centered on mouse
            const zoomRatio = newZoom / this.zoom;
            this.panX = mouseX - (mouseX - this.panX) * zoomRatio;
            this.panY = mouseY - (mouseY - this.panY) * zoomRatio;
            this.zoom = newZoom;
            this.updateTransform();
        }
    }

    handlePointerDown(e) {
        if (e.target.closest('.compare-slider')) return;
        
        this.isDragging = true;
        this.dragStartX = e.clientX;
        this.dragStartY = e.clientY;
        this.panStartX = this.panX;
        this.panStartY = this.panY;
        this.viewport.setPointerCapture(e.pointerId);
    }

    handlePointerMove(e) {
        if (this.isDragging) {
            const dx = e.clientX - this.dragStartX;
            const dy = e.clientY - this.dragStartY;
            this.panX = this.panStartX + dx;
            this.panY = this.panStartY + dy;
            this.updateTransform();
            this.updateStatusPosition(e);
        } else if (this.isSliderDragging) {
            this.updateSliderFromEvent(e);
        } else if (this.isPanelResizing) {
            this.updatePanelHeight(e);
        }
    }

    handlePointerUp(e) {
        this.isDragging = false;
        this.isSliderDragging = false;
        this.isPanelResizing = false;
    }

    handleSliderDown(e) {
        e.stopPropagation();
        this.isSliderDragging = true;
        this.compareSlider.setPointerCapture(e.pointerId);
        this.updateSliderFromEvent(e);
    }

    updateSliderFromEvent(e) {
        const rect = this.viewport.getBoundingClientRect();
        const x = e.clientX - rect.left;
        this.sliderPosition = Math.max(0, Math.min(1, x / rect.width));
        this.updateCompareClip();
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
        
        // Center
        this.panX = (rect.width - this.imageWidth * this.zoom) / 2;
        this.panY = (rect.height - this.imageHeight * this.zoom) / 2;
        
        this.updateTransform();
    }

    resetView() {
        this.zoomToFit();
        this.sliderPosition = 0.5;
        this.updateCompareClip();
    }

    updateTransform() {
        this.container.style.transform = `translate(${this.panX}px, ${this.panY}px) scale(${this.zoom})`;
        this.zoomLevelEl.textContent = Math.round(this.zoom * 100) + '%';
        this.updateCompareClip();
    }

    updateCompareClip() {
        if (this.viewMode === 'compare') {
            const rect = this.viewport.getBoundingClientRect();
            const sliderX = rect.width * this.sliderPosition;
            
            // Position the slider
            this.compareSlider.style.left = sliderX + 'px';
            this.compareSlider.style.display = 'block';
            
            // Clip the preview layer to the right of the slider
            // Convert screen position to image coordinates
            const imageX = (sliderX - this.panX) / this.zoom;
            this.previewLayer.style.clipPath = `inset(0 0 0 ${imageX}px)`;
            
            // Show both layers
            this.originalLayer.style.opacity = '1';
            this.previewLayer.style.opacity = '1';
        } else {
            this.compareSlider.style.display = 'none';
            this.previewLayer.style.clipPath = 'none';
            
            if (this.viewMode === 'original') {
                this.originalLayer.style.opacity = '1';
                this.previewLayer.style.opacity = '0';
            } else {
                this.originalLayer.style.opacity = '0';
                this.previewLayer.style.opacity = '1';
            }
        }
    }

    updateLayout() {
        if (this.imageWidth) {
            this.updateCompareClip();
        }
    }

    updateStatusPosition(e) {
        const rect = this.viewport.getBoundingClientRect();
        const x = Math.round((e.clientX - rect.left - this.panX) / this.zoom);
        const y = Math.round((e.clientY - rect.top - this.panY) / this.zoom);
        
        if (x >= 0 && x < this.imageWidth && y >= 0 && y < this.imageHeight) {
            this.statusPosition.textContent = `${x}, ${y}`;
        }
    }

    // === View Mode ===

    setViewMode(mode) {
        this.viewMode = mode;
        this.modeBtns.forEach(btn => {
            btn.classList.toggle('active', btn.dataset.mode === mode);
        });
        this.updateCompareClip();
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
            case ' ':
                this.spacePressed = true;
                this.viewport.classList.add('space-drag');
                break;
            case 'Enter':
                this.processAndDownload();
                break;
        }
    }

    handleKeyUp(e) {
        if (e.key === ' ') {
            this.spacePressed = false;
            this.viewport.classList.remove('space-drag');
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
            return this.colorCustom.value.trim() || 'lime';
        }
        return this.colorSelect.value;
    }

    updateColorPreview() {
        const color = this.getColor();
        this.colorPreview.style.backgroundColor = this.colorMap[color] || color;
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
        if (!this.resizedBlob) return;

        if (this.abortController) {
            this.abortController.abort();
        }
        this.abortController = new AbortController();

        this.loadingOverlay.hidden = false;

        const formData = new FormData();
        formData.append('image', this.resizedBlob, 'preview.png');
        formData.append('color', this.getColor());
        formData.append('tolerance', this.toleranceSlider.value);
        formData.append('feather', this.featherSlider.value);
        formData.append('despill', this.despillSlider.value);
        formData.append('erode', this.erodeSlider.value);
        formData.append('dilate', this.dilateSlider.value);

        try {
            const res = await fetch('/api/preview', {
                method: 'POST',
                body: formData,
                signal: this.abortController.signal
            });

            if (!res.ok) throw new Error('Preview failed');

            const blob = await res.blob();
            
            if (this.previewImage.src.startsWith('blob:')) {
                URL.revokeObjectURL(this.previewImage.src);
            }
            
            this.previewImage.src = URL.createObjectURL(blob);
        } catch (e) {
            if (e.name !== 'AbortError') {
                console.error('Preview error:', e);
            }
        } finally {
            this.loadingOverlay.hidden = true;
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

        [this.toleranceSlider, this.featherSlider, this.despillSlider, 
         this.erodeSlider, this.dilateSlider].forEach(s => this.updateSliderValue(s));

        this.updateColorPreview();
        this.updatePreview();
    }

    async processAndDownload() {
        if (!this.originalFile) return;

        this.processBtn.disabled = true;
        this.processBtn.textContent = '処理中...';

        const formData = new FormData();
        formData.append('image', this.originalFile);
        formData.append('color', this.getColor());
        formData.append('tolerance', this.toleranceSlider.value);
        formData.append('feather', this.featherSlider.value);
        formData.append('despill', this.despillSlider.value);
        formData.append('erode', this.erodeSlider.value);
        formData.append('dilate', this.dilateSlider.value);

        try {
            const res = await fetch('/api/process', {
                method: 'POST',
                body: formData
            });

            if (!res.ok) {
                const error = await res.json().catch(() => ({}));
                throw new Error(error.message || 'Processing failed');
            }

            const result = await res.json();

            const link = document.createElement('a');
            link.href = result.download_url;
            link.download = result.filename;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);

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
        }
    }
}

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    window.app = new ChromaApp();
});
