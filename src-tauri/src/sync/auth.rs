//! JWT 认证 + Token 密钥链存储
//! 使用 keyring crate 安全存储 token

use serde::{Deserialize, Serialize};

/// JWT Token 验证
pub fn verify_token(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    // 简单格式校验：JWT 应为三段 base64
    let parts: Vec<&str> = token.split('.').collect();
    parts.len() == 3
}

/// 认证状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub is_authenticated: bool,
    pub user_id: Option<String>,
    pub expires_at: Option<i64>,
}

/// Token 存储操作
/// 使用操作系统密钥链（keyring crate）
pub struct TokenStore {
    service_name: String,
}

impl TokenStore {
    pub fn new() -> Self {
        Self {
            service_name: "com.tickpulse.app".to_string(),
        }
    }

    /// 存储 token 到密钥链
    #[cfg(feature = "keyring")]
    pub fn store_token(&self, username: &str, token: &str) -> Result<(), String> {
        let entry = keyring::Entry::new(&self.service_name, username)
            .map_err(|e| format!("密钥链创建失败: {}", e))?;
        entry.set_password(token).map_err(|e| format!("Token存储失败: {}", e))
    }

    /// 从密钥链获取 token
    #[cfg(feature = "keyring")]
    pub fn get_token(&self, username: &str) -> Result<String, String> {
        let entry = keyring::Entry::new(&self.service_name, username)
            .map_err(|e| format!("密钥链创建失败: {}", e))?;
        entry.get_password().map_err(|e| format!("Token获取失败: {}", e))
    }

    /// 从密钥链删除 token
    #[cfg(feature = "keyring")]
    pub fn delete_token(&self, username: &str) -> Result<(), String> {
        let entry = keyring::Entry::new(&self.service_name, username)
            .map_err(|e| format!("密钥链创建失败: {}", e))?;
        entry.delete_credential().map_err(|e| format!("Token删除失败: {}", e))
    }

    // 无 keyring feature 时的 fallback
    #[cfg(not(feature = "keyring"))]
    pub fn store_token(&self, _username: &str, _token: &str) -> Result<(), String> {
        Err("keyring feature 未启用".to_string())
    }
    #[cfg(not(feature = "keyring"))]
    pub fn get_token(&self, _username: &str) -> Result<String, String> {
        Err("keyring feature 未启用".to_string())
    }
    #[cfg(not(feature = "keyring"))]
    pub fn delete_token(&self, _username: &str) -> Result<(), String> {
        Err("keyring feature 未启用".to_string())
    }
}

impl Default for TokenStore {
    fn default() -> Self {
        Self::new()
    }
}
