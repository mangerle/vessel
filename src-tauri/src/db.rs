// src-tauri/src/db.rs
use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use std::sync::OnceLock;

/// 全局连接池单例
static POOL: OnceLock<Pool<Sqlite>> = OnceLock::new();

/// 初始化数据库连接池并运行迁移
pub async fn init_db(database_url: &str) -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect(database_url).await?;
    // 运行 migrations 目录下的迁移脚本
    sqlx::migrate!("./migrations").run(&pool).await?;
    let _ = POOL.set(pool);
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
    fn test_db_init_and_pool() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            // 使用内存数据库进行测试
            let db_url = "sqlite::memory:";
            init_db(db_url).await.expect("测试数据库初始化失败");
            
            let pool = get_pool();
            let row: (i32,) = sqlx::query_as("SELECT 1")
                .fetch_one(pool)
                .await
                .expect("查询失败");
            
            assert_eq!(row.0, 1);
        });
    }
}
