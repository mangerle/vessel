// src-tauri/src/connection/manager.rs
use sqlx::Row;
use serde::{Deserialize, Serialize};
use crate::db::get_pool;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Connection {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub host: String,
    pub auth_config: Option<String>,
}

/// 获取所有已保存的连接
pub async fn get_all_connections() -> anyhow::Result<Vec<Connection>> {
    let pool = get_pool();
    let rows = sqlx::query_as::<_, Connection>(
        "SELECT id, name, driver, host, auth_config FROM connections"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 添加新连接
pub async fn add_connection(conn: Connection) -> anyhow::Result<()> {
    let pool = get_pool();
    sqlx::query(
        "INSERT INTO connections (id, name, driver, host, auth_config) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(conn.id)
    .bind(conn.name)
    .bind(conn.driver)
    .bind(conn.host)
    .bind(conn.auth_config)
    .execute(pool)
    .await?;
    Ok(())
}

/// 删除连接
pub async fn delete_connection(id: &str) -> anyhow::Result<()> {
    let pool = get_pool();
    sqlx::query("DELETE FROM connections WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    
    // 如果删除的是当前活动连接，则清空设置并清除缓存
    let active_id = get_active_connection_id().await?;
    if let Some(aid) = active_id {
        if aid == id {
            set_active_connection_id("").await?;
            super::clear_client_cache().await;
        }
    }
    
    Ok(())
}

/// 获取当前活动连接 ID
pub async fn get_active_connection_id() -> anyhow::Result<Option<String>> {
    let pool = get_pool();
    let row = sqlx::query("SELECT value FROM settings WHERE key = 'active_connection_id'")
        .fetch_optional(pool)
        .await?;
    
    match row {
        Some(r) => {
            let val: String = r.get(0);
            if val.is_empty() { Ok(None) } else { Ok(Some(val)) }
        },
        None => Ok(None)
    }
}

/// 设置当前活动连接 ID
pub async fn set_active_connection_id(id: &str) -> anyhow::Result<()> {
    let pool = get_pool();
    sqlx::query("UPDATE settings SET value = ? WHERE key = 'active_connection_id'")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
