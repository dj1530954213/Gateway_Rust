//! Docker容器管理辅助工具

use std::process::Command;
use anyhow::{Result, Context};
use serde_json::Value;

/// Docker容器信息
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub ports: Vec<String>,
}

/// Docker辅助工具
pub struct DockerHelper;

impl DockerHelper {
    /// 检查Docker是否运行
    pub fn is_docker_running() -> bool {
        Command::new("docker")
            .args(&["info"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// 检查docker-compose是否可用
    pub fn is_compose_available() -> bool {
        Command::new("docker-compose")
            .args(&["--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// 获取容器状态
    pub fn get_container_status(container_name: &str) -> Result<Option<ContainerInfo>> {
        let output = Command::new("docker")
            .args(&["inspect", container_name])
            .output()
            .context("Failed to inspect container")?;

        if !output.status.success() {
            return Ok(None);
        }

        let json_str = String::from_utf8(output.stdout)?;
        let json: Value = serde_json::from_str(&json_str)?;
        
        if let Some(container) = json.as_array().and_then(|arr| arr.first()) {
            let info = ContainerInfo {
                id: container["Id"].as_str().unwrap_or("").to_string(),
                name: container["Name"].as_str().unwrap_or("").to_string(),
                status: container["State"]["Status"].as_str().unwrap_or("").to_string(),
                ports: Vec::new(), // 简化版本，实际可以解析端口信息
            };
            Ok(Some(info))
        } else {
            Ok(None)
        }
    }

    /// 等待容器启动
    pub async fn wait_for_container_healthy(container_name: &str, timeout_secs: u64) -> Result<()> {
        use tokio::time::{sleep, Duration, Instant};
        
        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);
        
        loop {
            if start.elapsed() > timeout {
                anyhow::bail!("Timeout waiting for container {} to be healthy", container_name);
            }
            
            if let Some(info) = Self::get_container_status(container_name)? {
                if info.status == "running" {
                    return Ok(());
                }
            }
            
            sleep(Duration::from_millis(1000)).await;
        }
    }

    /// 获取容器日志
    pub fn get_container_logs(container_name: &str, lines: Option<u32>) -> Result<String> {
        let mut args = vec!["logs"];
        
        if let Some(n) = lines {
            args.push("--tail");
            args.push(&n.to_string());
        }
        
        args.push(container_name);
        
        let output = Command::new("docker")
            .args(&args)
            .output()
            .context("Failed to get container logs")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// 在容器中执行命令
    pub fn exec_in_container(container_name: &str, command: &[&str]) -> Result<String> {
        let mut args = vec!["exec", container_name];
        args.extend_from_slice(command);
        
        let output = Command::new("docker")
            .args(&args)
            .output()
            .context("Failed to exec in container")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Command failed: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// 复制文件到容器
    pub fn copy_to_container(container_name: &str, src: &str, dest: &str) -> Result<()> {
        let output = Command::new("docker")
            .args(&["cp", src, &format!("{}:{}", container_name, dest)])
            .output()
            .context("Failed to copy file to container")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Copy failed: {}", stderr);
        }

        Ok(())
    }

    /// 从容器复制文件
    pub fn copy_from_container(container_name: &str, src: &str, dest: &str) -> Result<()> {
        let output = Command::new("docker")
            .args(&["cp", &format!("{}:{}", container_name, src), dest])
            .output()
            .context("Failed to copy file from container")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Copy failed: {}", stderr);
        }

        Ok(())
    }

    /// 检查必需的工具是否可用
    pub fn check_requirements() -> Result<()> {
        if !Self::is_docker_running() {
            anyhow::bail!("Docker is not running or not available");
        }

        if !Self::is_compose_available() {
            anyhow::bail!("docker-compose is not available");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_requirements() {
        // 这个测试只在有Docker环境时运行
        if DockerHelper::is_docker_running() {
            assert!(DockerHelper::check_requirements().is_ok());
        }
    }
}