#!/usr/bin/env python3
"""
COOP/COEP ヘッダー付き HTTP サーバー

SharedArrayBuffer を有効にするためのヘッダーを追加します。
これにより、ブラウザで最高速の SharedArrayBuffer モードが利用可能になります。

使用方法:
    # ローカルのみ (localhost)
    python3 scripts/serve-with-coop-coep.py
    
    # ポート指定
    python3 scripts/serve-with-coop-coep.py -p 8080
    python3 scripts/serve-with-coop-coep.py --port 3000
    
    # 外部公開 (0.0.0.0)
    python3 scripts/serve-with-coop-coep.py --public
    python3 scripts/serve-with-coop-coep.py -P
    
    # ホストIP指定
    python3 scripts/serve-with-coop-coep.py --host 192.168.1.100
    python3 scripts/serve-with-coop-coep.py -H 0.0.0.0
    
    # 組み合わせ
    python3 scripts/serve-with-coop-coep.py --public -p 3000
    python3 scripts/serve-with-coop-coep.py -H 192.168.1.100 -p 8080
"""

import sys
import os
import socket
import argparse
from http.server import SimpleHTTPRequestHandler, HTTPServer

class COOPCOEPHandler(SimpleHTTPRequestHandler):
    """COOP/COEP ヘッダーを追加するハンドラ"""
    
    def end_headers(self):
        # SharedArrayBuffer を有効にするためのヘッダー
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        # CORSヘッダー（外部アクセス用）
        self.send_header('Access-Control-Allow-Origin', '*')
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

def get_local_ip():
    """ローカルIPアドレスを取得"""
    try:
        # 外部に接続を試みてローカルIPを取得
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.connect(("8.8.8.8", 80))
        ip = s.getsockname()[0]
        s.close()
        return ip
    except Exception:
        return "127.0.0.1"

def parse_args():
    """コマンドライン引数をパース"""
    parser = argparse.ArgumentParser(
        description='COOP/COEP ヘッダー付き HTTP サーバー (SharedArrayBuffer対応)',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
例:
  %(prog)s                    # localhost:8080 で起動
  %(prog)s -p 3000            # localhost:3000 で起動
  %(prog)s --public           # 0.0.0.0:8080 で起動（外部公開）
  %(prog)s -P -p 3000         # 0.0.0.0:3000 で起動（外部公開）
  %(prog)s -H 192.168.1.100   # 指定IPで起動
        """
    )
    
    parser.add_argument(
        '-p', '--port',
        type=int,
        default=8080,
        help='ポート番号 (デフォルト: 8080)'
    )
    
    parser.add_argument(
        '-H', '--host',
        type=str,
        default='127.0.0.1',
        help='バインドするホストアドレス (デフォルト: 127.0.0.1)'
    )
    
    parser.add_argument(
        '-P', '--public',
        action='store_true',
        help='外部公開モード (0.0.0.0にバインド)'
    )
    
    parser.add_argument(
        '-d', '--directory',
        type=str,
        default=None,
        help='サーブするディレクトリ (デフォルト: static/)'
    )
    
    return parser.parse_args()

def main():
    args = parse_args()
    
    # ホスト設定
    host = '0.0.0.0' if args.public else args.host
    port = args.port
    
    # ディレクトリ設定
    if args.directory:
        static_dir = os.path.abspath(args.directory)
    else:
        script_dir = os.path.dirname(os.path.abspath(__file__))
        project_dir = os.path.dirname(script_dir)
        static_dir = os.path.join(project_dir, 'static')
    
    if not os.path.exists(static_dir):
        print(f"\033[31mError: directory not found: {static_dir}\033[0m")
        print("Run ./scripts/build-wasm.sh first")
        sys.exit(1)
    
    os.chdir(static_dir)
    
    # ローカルIPを取得
    local_ip = get_local_ip()
    
    # サーバー起動
    server = HTTPServer((host, port), COOPCOEPHandler)
    
    # 接続情報を表示
    urls = []
    if host == '0.0.0.0':
        urls.append(f"http://localhost:{port}")
        urls.append(f"http://{local_ip}:{port}")
        urls.append(f"http://0.0.0.0:{port}")
    elif host == '127.0.0.1':
        urls.append(f"http://localhost:{port}")
    else:
        urls.append(f"http://{host}:{port}")
    
    print(f"""
╔══════════════════════════════════════════════════════════════════╗
║  chroma-transparent WASM Server (SharedArrayBuffer enabled)      ║
╠══════════════════════════════════════════════════════════════════╣
║                                                                  ║
║  URLs:                                                           ║""")
    
    for url in urls:
        print(f"║    {url:<56} ║")
    
    print(f"""║                                                                  ║
║  Directory: {static_dir[:50]:<50} ║
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
        print("\n\033[33mServer stopped\033[0m")
        server.shutdown()

if __name__ == '__main__':
    main()
