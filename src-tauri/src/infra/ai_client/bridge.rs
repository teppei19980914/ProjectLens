//! NewtonX Pythonサイドカーとの通信（04_実装詳細.md §3.2）。
//! Rust ⇔ Python 間は標準入出力上の JSON Lines RPC（1行1リクエスト/1行1レスポンス）。
//! RPC処理をメソッドごとに複製せず、`call()` 1本に集約する（CLAUDE.md 原則2.2.1）。

use super::{AiClient, AssistantInfo};
use crate::models::constants::newtonx_rpc;
use crate::models::{AppError, AppResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

type PendingMap = Arc<std::sync::Mutex<HashMap<String, oneshot::Sender<Value>>>>;

/// NewtonX Pythonサイドカー（`newtonx_bridge.py`）を子プロセスとして起動し、
/// JSON Lines RPCで通信する `AiClient` 実装。
pub struct NewtonXBridge {
    stdin: Mutex<ChildStdin>,
    pending: PendingMap,
    _child: Mutex<Child>,
    assistant_uid: String,
    /// 1回のRPC往復あたりのタイムアウト（config.ai.timeoutSecs）。
    /// これがないとNewtonX側やネットワークが応答しない場合に解析が無期限にハングする
    /// （実際に発生した不具合。04_実装詳細.md §3.5「タイムアウト」に対応する安全装置）。
    timeout: Duration,
}

impl NewtonXBridge {
    /// 開発時: `python_exe` でインタプリタを起動し `script_path` を実行する。
    /// 配布時: PyInstaller単一exe化したサイドカーバイナリを直接起動する形に置き換える
    /// （残課題: 04_実装詳細.md §10 #4「サイドカー配布方式」）。
    pub async fn spawn(
        python_exe: &str,
        script_path: &std::path::Path,
        config_path: &std::path::Path,
        assistant_uid: String,
        timeout_secs: u64,
    ) -> AppResult<Self> {
        let mut child = Command::new(python_exe)
            .arg(script_path)
            .arg(config_path)
            .current_dir(script_path.parent().unwrap_or_else(|| std::path::Path::new(".")))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()) // ログはstderrのみに出す設計（CLAUDE.md §5）
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| AppError::Ai {
                message: format!("NewtonXサイドカーの起動に失敗しました: {e}"),
                recoverable: false,
                retryable: false,
            })?;

        let stdin = child.stdin.take().ok_or_else(|| AppError::Ai {
            message: "サイドカーのstdin取得に失敗しました".into(),
            recoverable: false,
            retryable: false,
        })?;
        let stdout = child.stdout.take().ok_or_else(|| AppError::Ai {
            message: "サイドカーのstdout取得に失敗しました".into(),
            recoverable: false,
            retryable: false,
        })?;
        let stderr = child.stderr.take();

        let pending: PendingMap = Arc::new(std::sync::Mutex::new(HashMap::new()));

        // stdout: 1行1レスポンス(JSON)を読み取り、idで対応するoneshotへ配送する
        let pending_reader = pending.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            loop {
                match lines.next_line().await {
                    Ok(Some(line)) => {
                        if line.trim().is_empty() {
                            continue;
                        }
                        if let Ok(value) = serde_json::from_str::<Value>(&line) {
                            if let Some(id) = value.get("id").and_then(|v| v.as_str()) {
                                if let Some(sender) = pending_reader.lock().unwrap().remove(id) {
                                    let _ = sender.send(value);
                                }
                            }
                        }
                        // パース不能行は無視する（stdoutにRPC応答以外を出力しない設計だが、
                        // 予期しない出力混入時にRPC全体を止めないための安全策）
                    }
                    _ => break,
                }
            }
        });

        // stderr: ログとして扱う（本バージョンでは破棄。将来的にファイル/コンソールへ転送）
        if let Some(stderr) = stderr {
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(_line)) = lines.next_line().await {
                    // TODO: 構造化ロギングへ接続する
                }
            });
        }

        Ok(Self {
            stdin: Mutex::new(stdin),
            pending,
            _child: Mutex::new(child),
            assistant_uid,
            timeout: Duration::from_secs(timeout_secs.max(1)),
        })
    }

    async fn call(&self, method: &str, params: Value) -> AppResult<Value> {
        let id = Uuid::new_v4().to_string();
        let (tx, rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(id.clone(), tx);

        let request = json!({ "id": id, "method": method, "params": params });
        let mut line = serde_json::to_string(&request).map_err(|e| AppError::Ai {
            message: format!("RPCリクエストのシリアライズに失敗しました: {e}"),
            recoverable: true,
            retryable: false,
        })?;
        line.push('\n');

        {
            let mut stdin = self.stdin.lock().await;
            stdin.write_all(line.as_bytes()).await.map_err(|e| AppError::Ai {
                message: format!("サイドカーへの送信に失敗しました: {e}"),
                recoverable: true,
                retryable: true,
            })?;
            stdin.flush().await.ok();
        }

        let response = match tokio::time::timeout(self.timeout, rx).await {
            Ok(Ok(response)) => response,
            Ok(Err(_)) => {
                self.pending.lock().unwrap().remove(&id);
                return Err(AppError::Ai {
                    message: "サイドカーからの応答を受信できませんでした（プロセス終了の可能性）".into(),
                    recoverable: false,
                    retryable: false,
                });
            }
            Err(_elapsed) => {
                // config.ai.timeoutSecs 超過。孤立したpendingエントリを掃除してからリトライ可能なエラーとして返す
                // （04_実装詳細.md §3.5「タイムアウト」）。この安全装置がないと解析が無期限にハングする。
                self.pending.lock().unwrap().remove(&id);
                return Err(AppError::Ai {
                    message: format!(
                        "NewtonXサイドカーからの応答がタイムアウトしました（{}秒、method={method}）",
                        self.timeout.as_secs()
                    ),
                    recoverable: true,
                    retryable: true,
                });
            }
        };

        if let Some(error) = response.get("error") {
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("不明なサイドカーエラー")
                .to_string();
            let category = error.get("category").and_then(|c| c.as_str()).unwrap_or("");
            return Err(map_bridge_error(category, message));
        }

        Ok(response.get("result").cloned().unwrap_or(Value::Null))
    }
}

/// Pythonサイドカー側で判別したエラーカテゴリをAppErrorへ変換する（04_実装詳細.md §3.5）。
fn map_bridge_error(category: &str, message: String) -> AppError {
    match category {
        "authentication" => AppError::Ai {
            message,
            recoverable: false, // PAT方式のため自動再認証はできない（04_実装詳細.md §3.5）
            retryable: false,
        },
        "rate_limit" | "server" | "timeout" => AppError::Ai {
            message,
            recoverable: true,
            retryable: true,
        },
        "chat" | "configuration" => AppError::Ai {
            message,
            recoverable: false,
            retryable: false,
        },
        _ => AppError::Ai {
            message,
            recoverable: true,
            retryable: true,
        },
    }
}

#[async_trait]
impl AiClient for NewtonXBridge {
    async fn auth_status(&self) -> AppResult<bool> {
        let result = self.call(newtonx_rpc::AUTH_STATUS, json!({})).await?;
        Ok(result.get("authenticated").and_then(|v| v.as_bool()).unwrap_or(false))
    }

    async fn save_credentials(&self, host: &str, personal_access_token: &str) -> AppResult<()> {
        self.call(
            newtonx_rpc::AUTH_SAVE_CREDENTIALS,
            json!({ "host": host, "personalAccessToken": personal_access_token }),
        )
        .await?;
        Ok(())
    }

    async fn clear_credentials(&self) -> AppResult<()> {
        self.call(newtonx_rpc::AUTH_CLEAR_CREDENTIALS, json!({})).await?;
        Ok(())
    }

    async fn test_connection(&self) -> AppResult<String> {
        let result = self.call(newtonx_rpc::AI_TEST, json!({})).await?;
        Ok(result
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("OK")
            .to_string())
    }

    async fn list_assistants(&self) -> AppResult<Vec<AssistantInfo>> {
        let result = self.call(newtonx_rpc::ASSISTANTS_LIST, json!({})).await?;
        serde_json::from_value(result).map_err(|e| AppError::Ai {
            message: format!("アシスタント一覧の解析に失敗しました: {e}"),
            recoverable: true,
            retryable: false,
        })
    }

    async fn open_session(&self, title: &str) -> AppResult<String> {
        let result = self
            .call(
                newtonx_rpc::SESSION_OPEN,
                json!({ "title": title, "assistantUid": self.assistant_uid }),
            )
            .await?;
        result
            .get("chatUid")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::Ai {
                message: "session.openの応答にchatUidがありません".into(),
                recoverable: false,
                retryable: false,
            })
    }

    async fn close_session(&self, chat_uid: &str) -> AppResult<()> {
        self.call(newtonx_rpc::SESSION_CLOSE, json!({ "chatUid": chat_uid })).await?;
        Ok(())
    }

    async fn analyze(&self, chat_uid: &str, prompt: &str) -> AppResult<Option<String>> {
        // web_search=false / knowledge_search=false は必ず明示指定する（04_実装詳細.md §3.3、CLAUDE.md §3）
        let result = self
            .call(
                newtonx_rpc::AI_ANALYZE,
                json!({
                    "chatUid": chat_uid,
                    "message": prompt,
                    "webSearch": false,
                    "knowledgeSearch": false,
                }),
            )
            .await?;
        Ok(result.get("response").and_then(|v| v.as_str()).map(|s| s.to_string()))
    }
}
