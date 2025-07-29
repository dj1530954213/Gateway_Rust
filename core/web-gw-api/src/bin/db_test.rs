//! db_test.rs —— 数据库连接测试工具
//!
//! 用于验证PostgreSQL连接配置和数据库迁移

use anyhow::Result;
use sqlx::{Pool, Postgres};
use web_gw_api::config::{load_config, ApiConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 加载配置
    let config = load_config()?;
    
    println!("🔧 正在测试数据库连接...");
    println!("📍 DSN: {}", config.pg_dsn);
    
    // 尝试连接数据库
    match sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.pg_dsn)
        .await
    {
        Ok(pool) => {
            println!("✅ 数据库连接成功！");
            
            // 测试基本查询
            match sqlx::query("SELECT version()").fetch_one(&pool).await {
                Ok(row) => {
                    let version: String = row.get(0);
                    println!("📊 PostgreSQL版本: {}", version);
                }
                Err(e) => {
                    println!("❌ 查询版本失败: {}", e);
                }
            }
            
            // 尝试运行迁移
            println!("🔄 正在运行数据库迁移...");
            match sqlx::migrate!("../../../schema/migrations").run(&pool).await {
                Ok(_) => {
                    println!("✅ 数据库迁移成功！");
                    
                    // 检查表是否存在
                    check_tables(&pool).await?;
                }
                Err(e) => {
                    println!("❌ 数据库迁移失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ 数据库连接失败: {}", e);
            println!("💡 请确保:");
            println!("   1. PostgreSQL服务正在运行");
            println!("   2. 数据库 'iot' 已创建");
            println!("   3. 连接参数正确");
            
            return Err(e.into());
        }
    }
    
    Ok(())
}

async fn check_tables(pool: &Pool<Postgres>) -> Result<()> {
    let tables = vec!["devices", "tags", "driver_registry", "alert_rules"];
    
    println!("🔍 检查数据表...");
    
    for table in tables {
        let query = format!(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = '{}')",
            table
        );
        
        match sqlx::query(&query).fetch_one(pool).await {
            Ok(row) => {
                let exists: bool = row.get(0);
                if exists {
                    println!("  ✅ 表 '{}' 存在", table);
                } else {
                    println!("  ❌ 表 '{}' 不存在", table);
                }
            }
            Err(e) => {
                println!("  ❓ 检查表 '{}' 时出错: {}", table, e);
            }
        }
    }
    
    Ok(())
}