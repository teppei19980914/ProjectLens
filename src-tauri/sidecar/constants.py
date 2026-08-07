"""RPCメソッド名等の定数（Rust側 src-tauri/src/models/constants.rs とペアで保守する）。
CLAUDE.md 原則2.1.3: 文字列リテラルはRust側/Python側それぞれ1箇所に集約し、コピペ多重定義を禁止する。
"""

# NewtonXサイドカー RPCメソッド名（04_実装詳細.md §3.2）
AUTH_STATUS = "auth.status"
AUTH_SAVE_CREDENTIALS = "auth.save_credentials"
AUTH_CLEAR_CREDENTIALS = "auth.clear_credentials"
AI_TEST = "ai.test"
AI_ANALYZE = "ai.analyze"
SESSION_OPEN = "session.open"
SESSION_CLOSE = "session.close"
ASSISTANTS_LIST = "assistants.list"

# NewtonX解析用チャットの専用フォルダ名（04_実装詳細.md §3.3）
NEWTONX_CHAT_FOLDER_NAME = "ProjectLens"

# エラーカテゴリ（04_実装詳細.md §3.5。Rust側 bridge.rs の map_bridge_error と対応）
CATEGORY_AUTHENTICATION = "authentication"
CATEGORY_RATE_LIMIT = "rate_limit"
CATEGORY_SERVER = "server"
CATEGORY_TIMEOUT = "timeout"
CATEGORY_CHAT = "chat"
CATEGORY_CONFIGURATION = "configuration"
CATEGORY_UNKNOWN = "unknown"
