#!/usr/bin/env python3
"""NewtonX ADKサイドカー（04_実装詳細.md §3.2）。

Rustバックエンドと標準入出力上のJSON Lines RPC（1行1リクエスト/1行1レスポンス）で通信する。
標準出力にはRPC応答以外を一切出力しない。ログは標準エラー出力にのみ書き出す（CLAUDE.md §5）。

起動方法:
    python newtonx_bridge.py <config_file_path>

<config_file_path> はProjectLens専用のADK設定ファイル（例: %APPDATA%/ProjectLens/newtonx_adk_config.json）。
ADK共通の ~/.newtonx/config.json とは分離する（04_実装詳細.md §3.4）。
"""

from __future__ import annotations

import json
import sys
import threading
import traceback
from typing import Any, Callable, Dict

import constants

try:
    from newtonx_adk import (
        APIError,
        AuthenticationError,
        AuthManager,
        ChatError,
        ConfigurationError,
        ConfigManager,
        FileUploadError,
        NewtonXClient,
        NewtonXError,
    )
except ImportError as e:  # pragma: no cover - 起動時に依存不足を即座に知らせる
    sys.stderr.write(f"newtonx_adk のインポートに失敗しました。依存関係を確認してください: {e}\n")
    sys.exit(1)


def log(message: str) -> None:
    sys.stderr.write(f"{message}\n")
    sys.stderr.flush()


def classify_error(exc: Exception) -> str:
    """例外を04_実装詳細.md §3.5のカテゴリへ分類する。"""
    if isinstance(exc, AuthenticationError):
        return constants.CATEGORY_AUTHENTICATION
    if isinstance(exc, ChatError):
        return constants.CATEGORY_CHAT
    if isinstance(exc, ConfigurationError):
        return constants.CATEGORY_CONFIGURATION
    if isinstance(exc, APIError):
        # ADKのAPIErrorはstatus_codeが空のことが多いため、メッセージ文字列で可能な範囲で判別する
        # （04_実装詳細.md §3.5脚注・§10残課題#3）。判別不能時はserver扱いとし、Rust側の
        # 指数バックオフに委ねる。
        message = str(exc).lower()
        if "429" in message or "rate limit" in message or "too many requests" in message:
            return constants.CATEGORY_RATE_LIMIT
        if "timeout" in message or "timed out" in message:
            return constants.CATEGORY_TIMEOUT
        return constants.CATEGORY_SERVER
    if isinstance(exc, (FileUploadError, NewtonXError)):
        return constants.CATEGORY_UNKNOWN
    if isinstance(exc, TimeoutError):
        return constants.CATEGORY_TIMEOUT
    return constants.CATEGORY_UNKNOWN


class Context:
    """RPCハンドラ間で共有する状態。

    注意: `main()` は各RPCリクエストを別スレッドで並行処理する（Rust側の
    concurrency設定（既定5）を活かすため。詳細は `main()` のコメント参照）。
    そのため `client` / `auth_manager` / `config_manager` は複数スレッドから
    同時に呼ばれる可能性がある。NewtonX ADKが内部で使うHTTPクライアントの
    厳密なスレッド安全性は未検証だが、I/Oバウンドな処理でスレッドを使う一般的な
    パターンとして許容する（残課題）。
    """

    def __init__(self, config_file_path: str) -> None:
        self.config_manager = ConfigManager(config_file_path)
        self.auth_manager = AuthManager(self.config_manager)
        self.client = NewtonXClient(self.config_manager)


def handle_auth_status(ctx: Context, params: Dict[str, Any]) -> Dict[str, Any]:
    return {"authenticated": ctx.auth_manager.is_authenticated()}


def handle_auth_save_credentials(ctx: Context, params: Dict[str, Any]) -> Dict[str, Any]:
    host = normalize_host(params["host"])
    pat = params["personalAccessToken"]
    # PAT方式（04_実装詳細.md §3.4）。auth_mode='none' はOAuthブラウザフローを使わない指定。
    ctx.config_manager.update_config(host=host, personal_access_token=pat, auth_mode="none")
    return {"success": True}


def normalize_host(raw_host: str) -> str:
    """ConfigManager._derive_from_host() はhostを完全なドメイン
    （例: "seraku.newton-x.net"）として扱い、"."を含まない値は
    api_base_url = "https://<値>/api" のように誤って組み立ててしまう
    （実機で確認済みの不具合。全APIリクエストが到達不能になり401→
    ADK内部のMSAL再認証フォールバックが例外を送出する）。
    UI上は`newtonx_adk/docs/auth_setup_guide.md`に合わせて「会社サブドメイン」
    （例: seraku）の入力を案内しているため、ここで`setup_config.py`と同じ
    "<subdomain>.newton-x.net" への変換を行う。ユーザーが誤ってフルドメインを
    入力した場合（"."を含む場合）はそのまま使う。
    """
    host = (raw_host or "").strip()
    if not host:
        return host
    if "." in host:
        return host
    return f"{host}.newton-x.net"


def handle_auth_clear_credentials(ctx: Context, params: Dict[str, Any]) -> Dict[str, Any]:
    ctx.config_manager.update_config(personal_access_token="")
    try:
        # 検証済みの既知の挙動: auth_manager.logout() はADK内部で「ログアウトしました。」を
        # 標準出力に直接printする（stderrではない）。本来「stdoutにはRPC応答以外を出力しない」
        # 設計（CLAUDE.md §5）に反するが、Rust側bridge.rsはJSONとしてパースできない行を
        # 無視する安全策を備えているため、RPC通信自体は壊れない。将来的にはこの呼び出し中のみ
        # sys.stdoutを一時的にリダイレクトすることも検討する。
        ctx.auth_manager.logout()
    except Exception:  # ログアウト処理自体の失敗はPAT消去の成否に影響させない
        log("auth_manager.logout() failed (ignored)")
    return {}


def handle_ai_test(ctx: Context, params: Dict[str, Any]) -> Dict[str, Any]:
    status = ctx.client.get_model_status()
    available = sum(1 for v in status.values() if v)
    total = len(status)
    return {"ok": available > 0, "message": f"利用可能なモデル: {available}/{total}"}


def handle_assistants_list(ctx: Context, params: Dict[str, Any]) -> list[Dict[str, Any]]:
    assistants = ctx.client.get_assistants()
    return [{"uid": a["uid"], "name": a["name"]} for a in assistants]


def handle_session_open(ctx: Context, params: Dict[str, Any]) -> Dict[str, Any]:
    assistant_uid = params["assistantUid"]
    title = params.get("title") or "解析セッション"

    # 専用フォルダへの配置はチャット一覧を汚さないための付加価値であり、解析機能そのものの
    # 前提条件ではない（04_実装詳細.md §3.3）。フォルダ関連API（/folders）がPAT運用や
    # ADK側の401時フォールバック処理（実機で問題を確認済み。フォルダ一覧取得時の401リトライが
    # OAuth/MSAL経路に入り、tenant_id未設定のPAT運用では認証URLの構築に失敗して例外になる）で
    # 失敗しても、フォルダなしの通常チャットにフォールバックして解析自体は継続できるようにする。
    try:
        chat_uid = ctx.client.create_chat_in_folder_by_name(
            assistant_uid=assistant_uid,
            folder_name=constants.NEWTONX_CHAT_FOLDER_NAME,
            title=title,
            create_if_missing=True,
        )
        if chat_uid:
            return {"chatUid": chat_uid}
        log("create_chat_in_folder_by_name returned None. falling back to a plain chat without a folder.")
    except Exception as exc:
        log(f"create_chat_in_folder_by_name failed ({exc}); falling back to a plain chat without a folder.")

    chat_uid = ctx.client.create_chat(assistant_uid=assistant_uid, title=title)
    if not chat_uid:
        raise ChatError("チャットの作成に失敗しました（フォルダ配置あり/なし双方で失敗）")
    return {"chatUid": chat_uid}


def handle_session_close(ctx: Context, params: Dict[str, Any]) -> Dict[str, Any]:
    ctx.client.delete_chat(params["chatUid"])
    return {}


def handle_ai_analyze(ctx: Context, params: Dict[str, Any]) -> Dict[str, Any]:
    # web_search=False / knowledge_search=False を必ず明示指定する（CLAUDE.md §3、04_実装詳細.md §3.3）
    response = ctx.client.send_message(
        params["chatUid"],
        params["message"],
        web_search=params.get("webSearch", False),
        knowledge_search=params.get("knowledgeSearch", False),
    )
    return {"response": response}


HANDLERS: Dict[str, Callable[[Context, Dict[str, Any]], Any]] = {
    constants.AUTH_STATUS: handle_auth_status,
    constants.AUTH_SAVE_CREDENTIALS: handle_auth_save_credentials,
    constants.AUTH_CLEAR_CREDENTIALS: handle_auth_clear_credentials,
    constants.AI_TEST: handle_ai_test,
    constants.AI_ANALYZE: handle_ai_analyze,
    constants.SESSION_OPEN: handle_session_open,
    constants.SESSION_CLOSE: handle_session_close,
    constants.ASSISTANTS_LIST: handle_assistants_list,
}


_stdout_lock = threading.Lock()


def write_response(payload: Dict[str, Any]) -> None:
    # 複数スレッドから同時に呼ばれるため、1行分の書き込みが割り込まれないようロックする
    # （JSON Lines RPCは「1行1レスポンス」が前提。行の途中に別スレッドの出力が混ざると
    # Rust側でパースできなくなる）。
    with _stdout_lock:
        sys.stdout.write(json.dumps(payload, ensure_ascii=False) + "\n")
        sys.stdout.flush()


def dispatch(ctx: Context, request: Dict[str, Any]) -> None:
    request_id = request.get("id")
    method = request.get("method")
    params = request.get("params") or {}

    handler = HANDLERS.get(method)
    if handler is None:
        write_response({"id": request_id, "error": {"category": constants.CATEGORY_UNKNOWN, "message": f"未知のRPCメソッドです: {method}"}})
        return

    try:
        result = handler(ctx, params)
        write_response({"id": request_id, "result": result})
    except Exception as exc:  # RPC1件ごとに例外を捕捉し、プロセス全体を落とさない
        log(f"RPC error [{method}]: {exc}\n{traceback.format_exc()}")
        write_response({"id": request_id, "error": {"category": classify_error(exc), "message": str(exc)}})


_active_threads: list[threading.Thread] = []
_active_threads_lock = threading.Lock()


def spawn_dispatch(ctx: Context, request: Dict[str, Any]) -> None:
    thread = threading.Thread(target=dispatch, args=(ctx, request), daemon=False)
    with _active_threads_lock:
        # 完了済みスレッドを掃除してリストが際限なく肥大化しないようにする
        _active_threads[:] = [t for t in _active_threads if t.is_alive()]
        _active_threads.append(thread)
    thread.start()


def main() -> int:
    # Windows環境ではstdio既定エンコーディングがコンソールのコードページになる場合があるため、
    # JSON Lines RPCが常にUTF-8で送受信されるよう明示的に固定する。
    for stream in (sys.stdin, sys.stdout, sys.stderr):
        if hasattr(stream, "reconfigure"):
            stream.reconfigure(encoding="utf-8")

    if len(sys.argv) < 2:
        log("使用方法: python newtonx_bridge.py <config_file_path>")
        return 1

    ctx = Context(sys.argv[1])
    log("newtonx_bridge started")

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = json.loads(line)
        except json.JSONDecodeError:
            log(f"不正なRPCリクエスト行を無視しました: {line[:200]}")
            continue
        # 重要: dispatch()を同期呼び出しすると、1件のNewtonXリクエストが遅延/ハングした際に
        # この標準入力読み取りループ自体が止まり、Rust側が並行送信した他の全リクエスト
        # （config.ai.concurrency、既定5）も未読のまま止まってしまう（実際に発生した不具合）。
        # 各リクエストを別スレッドで処理し、次の行をすぐ読み続けられるようにする。
        spawn_dispatch(ctx, request)

    # 標準入力がEOFになった後（Rust側がstdinを閉じた等）も、処理中のスレッドが
    # write_response()で応答を書き終えるまで待ってから終了する。daemon=Trueのまま
    # 即座にプロセスを終了させると、処理中だったRPCの応答が失われる不具合があった
    # （実機で確認済み: auth.save_credentialsの応答が届かないケース）。
    with _active_threads_lock:
        remaining = list(_active_threads)
    for t in remaining:
        t.join(timeout=30)
        if t.is_alive():
            log("スレッドが30秒以内に終了しませんでした（応答が失われた可能性があります）")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
