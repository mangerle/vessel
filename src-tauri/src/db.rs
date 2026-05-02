// src-tauri/src/db.rs
use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use std::sync::OnceLock;

/// 全局连接池单例
static POOL: OnceLock<Pool<Sqlite>> = OnceLock::new();

/// 初始化数据库连接池并运行迁移
pub async fn init_db(database_url: &str) -> anyhow::Result<()> {
    if POOL.get().is_some() {
        return Ok(());
    }

    let pool = SqlitePool::connect(database_url).await?;
    // 运行 migrations 目录下的迁移脚本
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    if POOL.set(pool).is_err() {
        // 如果在检查后仍被设置，说明并发初始化，通常在 tauri setup 中不会发生，
        // 但为了严谨性进行处理
        return Ok(());
    }
    
    Ok(())
}

/// 获取全局连接池引用
pub fn get_pool() -> &'static Pool<Sqlite> {
    POOL.get().expect("数据库尚未初始化")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_db_init_and_schema() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            // 使用内存数据库进行测试
            let db_url = "sqlite::memory:";
            init_db(db_url).await.expect("测试数据库初始化失败");
            
            let pool = get_pool();
            
            // 验证 connections 表是否存在
            let (table_exists,): (bool,) = sqlx::query_as(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='connections')"
            )
            .fetch_one(pool)
            .await
            .expect("查询表状态失败");
            
            assert!(table_exists, "connections 表应该已创建");

            // 尝试向 connections 表插入数据以验证其结构
            sqlx::query("INSERT INTO connections (id, name, driver, host) VALUES (?, ?, ?, ?)")
                .bind(uuid::Uuid::new_v4().to_string())
                .bind("Test connection")
                .bind("local")
                .bind("localhost")
                .execute(pool)
                .await
                .expect("插入数据失败");
        });
    }
}
