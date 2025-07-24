//! 集成测试环境管理工具

use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;
use anyhow::{Result, Context};

/// 集成测试环境配置
#[derive(Debug, Clone)]
pub struct IntegrationTestEnv {
    pub mqtt_port: u16,
    pub influxdb_port: u16,
    pub mock_plc_port: u16,
    pub docker_compose_file: String,
}

impl Default for IntegrationTestEnv {
    fn default() -> Self {
        Self {
            mqtt_port: 1883,
            influxdb_port: 8086,
            mock_plc_port: 1502,
            docker_compose_file: "tests/integration/docker-compose.yml".to_string(),
        }
    }
}

impl IntegrationTestEnv {
    /// 启动测试环境
    pub async fn setup(&self) -> Result<()> {
        println!("🚀 启动集成测试环境...");
        
        // 停止可能存在的容器
        self.cleanup().await.ok();
        
        // 启动Docker Compose
        let output = Command::new("docker-compose")
            .args(&["-f", &self.docker_compose_file, "up", "-d"])
            .output()
            .context("Failed to start docker-compose")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Docker compose failed: {}", stderr);
        }

        // 等待服务启动
        self.wait_for_services().await?;
        
        println!("✅ 集成测试环境启动完成");
        Ok(())
    }

    /// 清理测试环境
    pub async fn cleanup(&self) -> Result<()> {
        println!("🧹 清理集成测试环境...");
        
        let output = Command::new("docker-compose")
            .args(&["-f", &self.docker_compose_file, "down", "-v"])
            .output()
            .context("Failed to stop docker-compose")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("Warning: Docker cleanup failed: {}", stderr);
        }

        println!("✅ 集成测试环境清理完成");
        Ok(())
    }

    /// 等待服务启动
    async fn wait_for_services(&self) -> Result<()> {
        println!("⏳ 等待服务启动...");
        
        // 等待MQTT Broker
        self.wait_for_port(self.mqtt_port, "MQTT Broker").await?;
        
        // 等待InfluxDB
        self.wait_for_port(self.influxdb_port, "InfluxDB").await?;
        
        // 等待Mock PLC
        self.wait_for_port(self.mock_plc_port, "Mock PLC").await?;
        
        // 额外等待2秒确保服务完全就绪
        sleep(Duration::from_secs(2)).await;
        
        Ok(())
    }

    /// 等待指定端口可用
    async fn wait_for_port(&self, port: u16, service_name: &str) -> Result<()> {
        use std::net::TcpStream;
        use std::time::Instant;
        
        let start = Instant::now();
        let timeout = Duration::from_secs(30);
        
        loop {
            if start.elapsed() > timeout {
                anyhow::bail!("Timeout waiting for {} on port {}", service_name, port);
            }
            
            if TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok() {
                println!("✅ {} is ready on port {}", service_name, port);
                break;
            }
            
            sleep(Duration::from_millis(500)).await;
        }
        
        Ok(())
    }

    /// 获取MQTT连接URL
    pub fn mqtt_url(&self) -> String {
        format!("tcp://127.0.0.1:{}", self.mqtt_port)
    }

    /// 获取InfluxDB URL
    pub fn influxdb_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.influxdb_port)
    }

    /// 获取Mock PLC URL
    pub fn mock_plc_url(&self) -> String {
        format!("tcp://127.0.0.1:{}", self.mock_plc_port)
    }
}

/// 测试环境RAII包装器
pub struct TestEnvironment {
    env: IntegrationTestEnv,
}

impl TestEnvironment {
    /// 创建并启动测试环境
    pub async fn new() -> Result<Self> {
        let env = IntegrationTestEnv::default();
        env.setup().await?;
        Ok(Self { env })
    }

    /// 获取环境配置
    pub fn config(&self) -> &IntegrationTestEnv {
        &self.env
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        // 在测试结束时异步清理环境
        tokio::spawn(async move {
            let env = IntegrationTestEnv::default();
            let _ = env.cleanup().await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_environment_setup_cleanup() {
        let env = IntegrationTestEnv::default();
        
        // 测试启动和清理
        env.setup().await.expect("Failed to setup test environment");
        env.cleanup().await.expect("Failed to cleanup test environment");
    }
}