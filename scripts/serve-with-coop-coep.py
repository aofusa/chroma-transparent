#!/usr/bin/env python3
"""
COOP/COEP ヘッダー付き HTTP サーバー

SharedArrayBuffer を有効にするためのヘッダーを追加します。
これにより、ブラウザで最高速の SharedArrayBuffer モードが利用可能になります。

使用方法:
    python3 scripts/serve-with-coop-coep.py [port]
    python3 scripts/serve-with-coop-coep.py 8080
"""

import sys
import os
from http.server import SimpleHTTPRequestHandler, HTTPServer

class COOPCOEPHandler(SimpleHTTPRequestHandler):
    """COOP/COEP ヘッダーを追加するハンドラ"""
    
    def end_headers(self):
        # SharedArrayBuffer を有効にするためのヘッダー
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        super().end_headers()
    
    def log_message(self, format, *args):
        # カラフルなログ出力
        message = format % args
        if '200' in message or '304' in message:
            print(f"\033[32m{message}\033[0m")  # 緑
        elif '404' in message:
            print(f"\033[31m{message}\033[0m")  # 赤
        else:
            print(message)

def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
    
    # static ディレクトリに移動
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_dir = os.path.dirname(script_dir)
    static_dir = os.path.join(project_dir, 'static')
    
    if not os.path.exists(static_dir):
        print(f"Error: static directory not found: {static_dir}")
        print("Run ./scripts/build-wasm.sh first")
        sys.exit(1)
    
    os.chdir(static_dir)
    
    server = HTTPServer(('', port), COOPCOEPHandler)
    
    print(f"""
╔══════════════════════════════════════════════════════════════════╗
║  chroma-transparent WASM Server (SharedArrayBuffer enabled)      ║
╠══════════════════════════════════════════════════════════════════╣
║                                                                  ║
║  URL: http://localhost:{port:<5}                                   ║
║                                                                  ║
║  Headers:                                                        ║
║    Cross-Origin-Opener-Policy: same-origin                       ║
║    Cross-Origin-Embedder-Policy: require-corp                    ║
║                                                                  ║
║  Mode: SharedArrayBuffer (fastest)                               ║
║                                                                  ║
║  Press Ctrl+C to stop                                            ║
╚══════════════════════════════════════════════════════════════════╝
""")
    
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nServer stopped")

if __name__ == '__main__':
    main()

