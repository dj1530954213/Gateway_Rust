//! db_test.rs —— 数据库连接与配置验证（占位）

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 使用库暴露的加载配置与简要输出，避免真实连接数据库
    let cfg = web_gw_api::load_config()?;
    println!("db_test config: {:?}", cfg.redacted());

    // 仅验证配置逻辑，不触发真实连接
    // 如需实际校验连接，请在本地设置正确的 WEBGW_PG_DSN 后，
    // 启用如下代码：
    // let mgr = web_gw_api::DatabasePoolManager::new(&cfg.pg_dsn, cfg.database_pool.clone()).await?;
    // println!("health = {:?}", mgr.health_check().await?);

    Ok(())
}
