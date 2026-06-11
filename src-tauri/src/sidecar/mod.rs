//! Python Sidecar 架构
//! 子进程管理 + stdin/stdout JSON IPC + 崩溃重启

use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

/// Sidecar 请求
#[derive(Debug, Serialize)]
struct SidecarRequest {
    id: u32,
    action: String,
    params: serde_json::Value,
}

/// Sidecar 响应
#[derive(Debug, Deserialize)]
struct SidecarResponse {
    id: u32,
    success: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

/// Python Sidecar 管理器
pub struct SidecarManager {
    process: Arc<Mutex<Option<Child>>>,
    request_id: Arc<Mutex<u32>>,
    python_path: String,
    script_path: String,
}

impl SidecarManager {
    pub fn new(python_path: &str, script_path: &str) -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            request_id: Arc::new(Mutex::new(0)),
            python_path: python_path.to_string(),
            script_path: script_path.to_string(),
        }
    }

    /// 启动 Python 子进程
    pub async fn start(&self) -> Result<(), String> {
        let mut proc = self.process.lock().await;

        let child = Command::new(&self.python_path)
            .arg(&self.script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("启动 Sidecar 失败: {}", e))?;

        *proc = Some(child);
        tracing::info!("Python Sidecar 已启动");
        Ok(())
    }

    /// 发送请求并等待响应
    pub async fn send_request(
        &self,
        action: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let mut id_guard = self.request_id.lock().await;
        *id_guard += 1;
        let id = *id_guard;
        drop(id_guard);

        let request = SidecarRequest {
            id,
            action: action.to_string(),
            params,
        };

        let request_json = serde_json::to_string(&request)
            .map_err(|e| format!("序列化请求失败: {}", e))?;

        // 写入 stdin
        let mut proc = self.process.lock().await;
        let child = proc.as_mut().ok_or("Sidecar 未启动")?;

        let stdin = child.stdin.as_mut().ok_or("无法获取 stdin")?;
        stdin.write_all(format!("{}\n", request_json).as_bytes()).await
            .map_err(|e| format!("写入 Sidecar stdin 失败: {}", e))?;
        stdin.flush().await
            .map_err(|e| format!("flush stdin 失败: {}", e))?;

        // 读取 stdout
        let stdout = child.stdout.as_mut().ok_or("无法获取 stdout")?;
        let mut reader = BufReader::new(stdout);
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await
            .map_err(|e| format!("读取 Sidecar stdout 失败: {}", e))?;

        let response: SidecarResponse = serde_json::from_str(response_line.trim())
            .map_err(|e| format!("解析 Sidecar 响应失败: {}", e))?;

        if response.success {
            Ok(response.data.unwrap_or(serde_json::Value::Null))
        } else {
            Err(response.error.unwrap_or_else(|| "未知错误".to_string()))
        }
    }

    /// 停止 Python 子进程
    pub async fn stop(&self) -> Result<(), String> {
        let mut proc = self.process.lock().await;
        if let Some(child) = proc.as_mut() {
            child.kill().await.map_err(|e| format!("停止 Sidecar 失败: {}", e))?;
            *proc = None;
            tracing::info!("Python Sidecar 已停止");
        }
        Ok(())
    }

    /// 检查是否运行中
    pub async fn is_running(&self) -> bool {
        let proc = self.process.lock().await;
        proc.is_some()
    }

    /// 重启（崩溃恢复）
    pub async fn restart(&self) -> Result<(), String> {
        self.stop().await?;
        self.start().await
    }
}
