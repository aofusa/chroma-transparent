/**
 * chroma-transparent Web UI
 * Industrial Precision Design
 */

class ChromaApp {
    constructor() {
        // Elements
        this.dropzone = document.getElementById('dropzone');
        this.fileInput = document.getElementById('file-input');
        this.previewSection = document.getElementById('preview-section');
        this.controlsSection = document.getElementById('controls-section');
        this.originalImage = document.getElementById('original-image');
        this.previewImage = document.getElementById('preview-image');
        this.loadingOverlay = document.getElementById('loading-overlay');
        
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
        
        // State
        this.originalFile = null;
        this.resizedBlob = null;
        this.debounceTimer = null;
        this.abortController = null;
        
        // Default values
        this.defaults = {
            color: 'lime',
            tolerance: 0.3,
            feather: 5,
            despill: 0.7,
            erode: 0,
            dilate: 1
        };
        
        // Color map for preview
        this.colorMap = {
            lime: '#00ff00',
            green: '#008000',
            blue: '#0000ff',
            magenta: '#ff00ff',
            cyan: '#00ffff',
            red: '#ff0000',
            yellow: '#ffff00'
        };
        
        this.init();
    }
    
    init() {
        this.bindEvents();
        this.loadConfig();
        this.updateColorPreview();
    }
    
    bindEvents() {
        // Dropzone events
        this.dropzone.addEventListener('click', (e) => {
            if (e.target.tagName !== 'LABEL') {
                this.fileInput.click();
            }
        });
        this.dropzone.addEventListener('dragover', (e) => this.handleDragOver(e));
        this.dropzone.addEventListener('dragleave', () => this.dropzone.classList.remove('dragover'));
        this.dropzone.addEventListener('drop', (e) => this.handleDrop(e));
        this.fileInput.addEventListener('change', (e) => this.handleFileSelect(e));
        
        // Control events
        this.colorSelect.addEventListener('change', () => this.handleColorChange());
        this.colorCustom.addEventListener('input', () => this.handleColorChange());
        
        const sliders = [
            this.toleranceSlider,
            this.featherSlider,
            this.despillSlider,
            this.erodeSlider,
            this.dilateSlider
        ];
        
        sliders.forEach(slider => {
            slider.addEventListener('input', () => {
                this.updateSliderValue(slider);
                this.schedulePreview();
            });
        });
        
        // Button events
        this.resetBtn.addEventListener('click', () => this.resetParams());
        this.processBtn.addEventListener('click', () => this.processAndDownload());
    }
    
    async loadConfig() {
        try {
            const healthRes = await fetch('/api/health');
            const health = await healthRes.json();
            document.getElementById('version').textContent = health.version || '0.1.0';
        } catch (e) {
            console.warn('Failed to load config:', e);
        }
    }
    
    handleDragOver(e) {
        e.preventDefault();
        e.stopPropagation();
        this.dropzone.classList.add('dragover');
    }
    
    handleDrop(e) {
        e.preventDefault();
        e.stopPropagation();
        this.dropzone.classList.remove('dragover');
        
        const files = e.dataTransfer.files;
        if (files.length > 0) {
            this.loadFile(files[0]);
        }
    }
    
    handleFileSelect(e) {
        if (e.target.files.length > 0) {
            this.loadFile(e.target.files[0]);
        }
    }
    
    async loadFile(file) {
        if (!file.type.startsWith('image/')) {
            this.showNotification('画像ファイルを選択してください', 'error');
            return;
        }
        
        this.originalFile = file;
        
        // Display original image
        const url = URL.createObjectURL(file);
        this.originalImage.src = url;
        
        // Resize for preview (smaller = faster)
        this.resizedBlob = await this.resizeImage(file, 512);
        
        // Show sections with animation
        this.previewSection.hidden = false;
        this.controlsSection.hidden = false;
        
        // Update dropzone to show file info
        const fileSize = this.formatFileSize(file.size);
        this.dropzone.innerHTML = `
            <div class="dropzone-content">
                <svg class="upload-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                    <path d="M9 12l2 2 4-4"/>
                    <circle cx="12" cy="12" r="10"/>
                </svg>
                <p>${this.escapeHtml(file.name)}</p>
                <p class="hint">${fileSize} — 別の画像をドロップして置き換え</p>
            </div>
        `;
        
        // Generate initial preview
        this.updatePreview();
    }
    
    async resizeImage(file, maxSize) {
        return new Promise((resolve) => {
            const img = new Image();
            img.onload = () => {
                const canvas = document.createElement('canvas');
                let { width, height } = img;
                
                if (width > maxSize || height > maxSize) {
                    const ratio = maxSize / Math.max(width, height);
                    width = Math.round(width * ratio);
                    height = Math.round(height * ratio);
                }
                
                canvas.width = width;
                canvas.height = height;
                
                const ctx = canvas.getContext('2d');
                ctx.drawImage(img, 0, 0, width, height);
                
                canvas.toBlob((blob) => {
                    URL.revokeObjectURL(img.src);
                    resolve(blob);
                }, 'image/png');
            };
            img.src = URL.createObjectURL(file);
        });
    }
    
    handleColorChange() {
        const value = this.colorSelect.value;
        this.colorCustom.hidden = value !== 'custom';
        this.updateColorPreview();
        this.schedulePreview();
    }
    
    getColor() {
        if (this.colorSelect.value === 'custom') {
            const custom = this.colorCustom.value.trim();
            return custom || 'lime';
        }
        return this.colorSelect.value;
    }
    
    updateColorPreview() {
        const color = this.getColor();
        const hex = this.colorMap[color] || color;
        this.colorPreview.style.backgroundColor = hex;
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
        
        // Cancel previous request
        if (this.abortController) {
            this.abortController.abort();
        }
        this.abortController = new AbortController();
        
        // Show loading
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
            
            if (!res.ok) {
                const error = await res.json().catch(() => ({}));
                throw new Error(error.message || 'Preview failed');
            }
            
            const blob = await res.blob();
            
            // Revoke old URL
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
        
        // Update displayed values
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
            
            // Download
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
            this.showNotification('処理に失敗しました: ' + e.message, 'error');
            this.processBtn.textContent = '処理してダウンロード';
            this.processBtn.disabled = false;
        }
    }
    
    // Utility methods
    formatFileSize(bytes) {
        if (bytes < 1024) return bytes + ' B';
        if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
        return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
    }
    
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
    
    showNotification(message, type = 'info') {
        // Simple alert for now - can be enhanced with toast notifications
        alert(message);
    }
}

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.app = new ChromaApp();
});
