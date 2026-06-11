//! 云端 CRUD 同步客户端
//! 支持 GET/PUT/DELETE /sync/data 端点

use reqwest::Client;
use serde::{Deserialize, Serialize};

/// 云同步配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub server_url: String,
    pub auth_token: Option<String>,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            server_url: "https://sync.stock-monitor.app".to_string(),
            auth_token: None,
        }
    }
}

/// 云同步客户端
pub struct SyncClient {
    client: Client,
    config: SyncConfig,
}

impl SyncClient {
    pub fn new(config: SyncConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to build sync HTTP client");
        Self { client, config }
    }

    /// 构建 Authorization header
    fn auth_header(&self) -> Option<String> {
        self.config.auth_token.as_ref().map(|t| format!("Bearer {}", t))
    }

    /// 上传数据到云端
    pub async fn push_data(&self, data: &str) -> Result<(), String> {
        let mut req = self.client.put(&format!("{}/sync/data", self.config.server_url));
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        req = req.header("Content-Type", "application/json");
        req = req.body(data.to_string());

        let resp = req.send().await.map_err(|e| format!("推送失败: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("推送失败: HTTP {}", resp.status()));
        }
        Ok(())
    }

    /// 从云端拉取数据
    pub async fn pull_data(&self) -> Result<String, String> {
        let mut req = self.client.get(&format!("{}/sync/data", self.config.server_url));
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let resp = req.send().await.map_err(|e| format!("拉取失败: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("拉取失败: HTTP {}", resp.status()));
        }
        resp.text().await.map_err(|e| format!("读取响应失败: {}", e))
    }

    /// 删除云端数据
    pub async fn delete_data(&self) -> Result<(), String> {
        let mut req = self.client.delete(&format!("{}/sync/data", self.config.server_url));
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let resp = req.send().await.map_err(|e| format!("删除失败: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("删除失败: HTTP {}", resp.status()));
        }
        Ok(())
    }
}

/// 同步到云端（简化入口）
pub async fn sync_to_cloud(config: &SyncConfig, data: &str) -> Result<(), String> {
    let client = SyncClient::new(config.clone());
    client.push_data(data).await
}
