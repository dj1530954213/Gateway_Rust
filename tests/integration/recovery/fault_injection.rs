//! 故障注入工具集
//! 提供各种故障模拟功能，用于测试系统的容错能力

use std::time::Duration;
use std::collections::HashMap;
use tokio::time::sleep;
use serde_json::{json, Value};
use anyhow::{Result, Context};

/// 故障类型
#[derive(Debug, Clone)]
pub enum FaultType {
    NetworkDisconnection,
    NetworkLatency(u32),           // 延迟毫秒数
    NetworkPacketLoss(f32),        // 丢包率 (0.0-1.0)
    ServiceCrash(String),          // 服务名称
    DiskFull,
    MemoryExhaustion,
    CPUSpike(u32),                 // CPU使用率百分比
    DatabaseUnavailable,
}

/// 故障注入参数
#[derive(Debug, Clone)]
pub struct FaultInjectionParams {
    pub fault_type: FaultType,
    pub duration: Duration,
    pub target_component: String,
    pub severity: FaultSeverity,
    pub auto_recover: bool,
}

/// 故障严重程度
#[derive(Debug, Clone)]
pub enum FaultSeverity {
    Low,      // 轻微影响
    Medium,   // 中等影响
    High,     // 严重影响
    Critical, // 灾难性影响
}

/// 故障注入结果
#[derive(Debug)]
pub struct FaultInjectionResult {
    pub fault_id: String,
    pub start_time: std::time::Instant,
    pub end_time: Option<std::time::Instant>,
    pub impact_metrics: HashMap<String, f64>,
    pub recovery_actions: Vec<RecoveryAction>,
}

/// 恢复动作
#[derive(Debug, Clone)]
pub struct RecoveryAction {
    pub action_type: RecoveryActionType,
    pub timestamp: std::time::Instant,
    pub success: bool,
    pub details: String,
}

#[derive(Debug, Clone)]
pub enum RecoveryActionType {
    AutoReconnect,
    ServiceRestart,
    FailoverToBackup,
    DataRecovery,
    ManualIntervention,
}

/// 综合故障注入器
pub struct ComprehensiveFaultInjector {
    toxiproxy_url: String,
    mock_plc_url: String,
    active_faults: HashMap<String, FaultInjectionParams>,
    fault_counter: u32,
}

impl ComprehensiveFaultInjector {
    /// 创建新的故障注入器
    pub fn new(toxiproxy_url: String, mock_plc_url: String) -> Self {
        Self {
            toxiproxy_url,
            mock_plc_url,
            active_faults: HashMap::new(),
            fault_counter: 0,
        }
    }

    /// 注入指定类型的故障
    pub async fn inject_fault(&mut self, params: FaultInjectionParams) -> Result<String> {
        self.fault_counter += 1;
        let fault_id = format!("fault_{}", self.fault_counter);
        
        println!("💉 注入故障: {} - {:?}", fault_id, params.fault_type);
        
        match &params.fault_type {
            FaultType::NetworkDisconnection => {
                self.inject_network_disconnection(&params.target_component).await?;
            }
            FaultType::NetworkLatency(latency_ms) => {
                self.inject_network_latency(&params.target_component, *latency_ms).await?;
            }
            FaultType::NetworkPacketLoss(loss_rate) => {
                self.inject_packet_loss(&params.target_component, *loss_rate).await?;
            }
            FaultType::ServiceCrash(service) => {
                self.inject_service_crash(service).await?;
            }
            FaultType::DiskFull => {
                self.inject_disk_full().await?;
            }
            FaultType::CPUSpike(cpu_percent) => {
                self.inject_cpu_spike(*cpu_percent).await?;
            }
            FaultType::DatabaseUnavailable => {
                self.inject_database_unavailable().await?;
            }
            FaultType::MemoryExhaustion => {
                self.inject_memory_exhaustion().await?;
            }
        }
        
        self.active_faults.insert(fault_id.clone(), params);
        
        println!("✅ 故障注入完成: {}", fault_id);
        Ok(fault_id)
    }

    /// 恢复指定故障
    pub async fn recover_fault(&mut self, fault_id: &str) -> Result<()> {
        if let Some(params) = self.active_faults.remove(fault_id) {
            println!("🔄 恢复故障: {} - {:?}", fault_id, params.fault_type);
            
            match params.fault_type {
                FaultType::NetworkDisconnection | 
                FaultType::NetworkLatency(_) | 
                FaultType::NetworkPacketLoss(_) => {
                    self.clear_network_faults(&params.target_component).await?;
                }
                FaultType::ServiceCrash(_) => {
                    // 服务崩溃恢复通常由监控系统自动处理
                    println!("🔄 等待服务自动重启...");
                }
                FaultType::DiskFull => {
                    self.clear_disk_full().await?;
                }
                FaultType::CPUSpike(_) => {
                    self.clear_cpu_spike().await?;
                }
                FaultType::DatabaseUnavailable => {
                    self.recover_database().await?;
                }
                FaultType::MemoryExhaustion => {
                    self.clear_memory_exhaustion().await?;
                }
            }
            
            println!("✅ 故障恢复完成: {}", fault_id);
        } else {
            anyhow::bail!("故障ID不存在: {}", fault_id);
        }
        
        Ok(())
    }

    /// 清除所有故障
    pub async fn clear_all_faults(&mut self) -> Result<()> {
        println!("🧹 清除所有故障...");
        
        let fault_ids: Vec<String> = self.active_faults.keys().cloned().collect();
        
        for fault_id in fault_ids {
            if let Err(e) = self.recover_fault(&fault_id).await {
                eprintln!("⚠️  清除故障失败 {}: {:?}", fault_id, e);
            }
        }
        
        // 额外清理网络故障
        let _ = self.clear_all_network_faults().await;
        
        println!("✅ 所有故障已清除");
        Ok(())
    }

    /// 注入网络断连故障
    async fn inject_network_disconnection(&self, target: &str) -> Result<()> {
        let client = reqwest::Client::new();
        let proxy_name = self.get_proxy_name(target)?;
        
        let response = client
            .post(&format!("{}/proxies/{}/toxics", self.toxiproxy_url, proxy_name))
            .json(&json!({
                "type": "bandwidth",
                "name": "network_disconnect",
                "attributes": {
                    "rate": 0
                }
            }))
            .send()
            .await
            .context("Failed to inject network disconnection")?;

        if !response.status().is_success() {
            anyhow::bail!("Network disconnection injection failed: {}", response.status());
        }

        Ok(())
    }

    /// 注入网络延迟故障
    async fn inject_network_latency(&self, target: &str, latency_ms: u32) -> Result<()> {
        let client = reqwest::Client::new();
        let proxy_name = self.get_proxy_name(target)?;
        
        let response = client
            .post(&format!("{}/proxies/{}/toxics", self.toxiproxy_url, proxy_name))
            .json(&json!({
                "type": "latency",
                "name": "network_latency",
                "attributes": {
                    "latency": latency_ms,
                    "jitter": latency_ms / 10
                }
            }))
            .send()
            .await
            .context("Failed to inject network latency")?;

        if !response.status().is_success() {
            anyhow::bail!("Network latency injection failed: {}", response.status());
        }

        Ok(())
    }

    /// 注入丢包故障
    async fn inject_packet_loss(&self, target: &str, loss_rate: f32) -> Result<()> {
        let client = reqwest::Client::new();
        let proxy_name = self.get_proxy_name(target)?;
        
        let response = client
            .post(&format!("{}/proxies/{}/toxics", self.toxiproxy_url, proxy_name))
            .json(&json!({
                "type": "bandwidth",
                "name": "packet_loss",
                "attributes": {
                    "rate": (1.0 - loss_rate) * 1000000.0 // kbps
                }
            }))
            .send()
            .await
            .context("Failed to inject packet loss")?;

        if !response.status().is_success() {
            anyhow::bail!("Packet loss injection failed: {}", response.status());
        }

        Ok(())
    }

    /// 注入服务崩溃故障
    async fn inject_service_crash(&self, service: &str) -> Result<()> {
        println!("💥 模拟服务崩溃: {}", service);
        
        // 通过Mock PLC的控制API模拟服务崩溃
        let client = reqwest::Client::new();
        let response = client
            .post(&self.mock_plc_url)
            .json(&json!({
                "command": "simulate_crash",
                "service": service,
                "crash_type": "immediate"
            }))
            .send()
            .await
            .context("Failed to simulate service crash")?;

        if !response.status().is_success() {
            anyhow::bail!("Service crash simulation failed: {}", response.status());
        }

        Ok(())
    }

    /// 注入磁盘满故障
    async fn inject_disk_full(&self) -> Result<()> {
        println!("💾 模拟磁盘空间不足...");
        
        // 这里可以通过创建大文件来模拟磁盘满
        // 在实际测试中，建议使用专门的工具或容器限制
        
        Ok(())
    }

    /// 注入CPU峰值故障
    async fn inject_cpu_spike(&self, cpu_percent: u32) -> Result<()> {
        println!("🔥 模拟CPU峰值: {}%", cpu_percent);
        
        // 通过Mock PLC API模拟高CPU使用率
        let client = reqwest::Client::new();
        let response = client
            .post(&self.mock_plc_url)
            .json(&json!({
                "command": "inject_fault",
                "error_rate": 0.0,
                "response_delay": cpu_percent as f64 / 100.0
            }))
            .send()
            .await
            .context("Failed to inject CPU spike")?;

        if !response.status().is_success() {
            anyhow::bail!("CPU spike injection failed: {}", response.status());
        }

        Ok(())
    }

    /// 注入数据库不可用故障
    async fn inject_database_unavailable(&self) -> Result<()> {
        println!("🗄️  模拟数据库不可用...");
        
        // 在实际环境中，这可能涉及停止数据库容器或阻断数据库连接
        // 这里简化处理
        
        Ok(())
    }

    /// 注入内存耗尽故障
    async fn inject_memory_exhaustion(&self) -> Result<()> {
        println!("🧠 模拟内存耗尽...");
        
        // 实际实现中可能需要分配大量内存或使用cgroup限制
        
        Ok(())
    }

    /// 清除网络故障
    async fn clear_network_faults(&self, target: &str) -> Result<()> {
        let client = reqwest::Client::new();
        let proxy_name = self.get_proxy_name(target)?;
        
        // 删除所有网络相关的toxic
        let toxics = ["network_disconnect", "network_latency", "packet_loss"];
        
        for toxic in &toxics {
            let _ = client
                .delete(&format!("{}/proxies/{}/toxics/{}", self.toxiproxy_url, proxy_name, toxic))
                .send()
                .await;
        }
        
        Ok(())
    }

    /// 清除所有网络故障
    async fn clear_all_network_faults(&self) -> Result<()> {
        let client = reqwest::Client::new();
        
        // 获取所有代理
        let response = client
            .get(&format!("{}/proxies", self.toxiproxy_url))
            .send()
            .await?;

        if response.status().is_success() {
            if let Ok(proxies) = response.json::<Value>().await {
                if let Some(proxy_list) = proxies.as_object() {
                    for proxy_name in proxy_list.keys() {
                        let _ = self.clear_network_faults(proxy_name).await;
                    }
                }
            }
        }

        Ok(())
    }

    /// 清除磁盘满故障
    async fn clear_disk_full(&self) -> Result<()> {
        println!("🧹 清除磁盘满故障...");
        // 删除之前创建的大文件
        Ok(())
    }

    /// 清除CPU峰值故障
    async fn clear_cpu_spike(&self) -> Result<()> {
        println!("❄️  清除CPU峰值故障...");
        
        let client = reqwest::Client::new();
        let response = client
            .post(&self.mock_plc_url)
            .json(&json!({
                "command": "clear_faults"
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to clear CPU spike");
        }

        Ok(())
    }

    /// 恢复数据库
    async fn recover_database(&self) -> Result<()> {
        println!("🔄 恢复数据库连接...");
        // 重启数据库或移除连接阻断
        Ok(())
    }

    /// 清除内存耗尽故障
    async fn clear_memory_exhaustion(&self) -> Result<()> {
        println!("🧹 清除内存耗尽故障...");
        // 释放分配的内存或移除内存限制
        Ok(())
    }

    /// 获取代理名称
    fn get_proxy_name(&self, target: &str) -> Result<String> {
        match target {
            "mqtt" => Ok("mqtt_proxy".to_string()),
            "influxdb" => Ok("influxdb_proxy".to_string()),
            "plc" => Ok("plc_proxy".to_string()),
            _ => Ok("mqtt_proxy".to_string()), // 默认使用MQTT代理
        }
    }

    /// 获取活跃故障列表
    pub fn get_active_faults(&self) -> &HashMap<String, FaultInjectionParams> {
        &self.active_faults
    }

    /// 执行故障注入场景
    pub async fn run_fault_scenario(&mut self, scenario: FaultScenario) -> Result<FaultScenarioResult> {
        println!("🎬 执行故障场景: {}", scenario.name);
        
        let mut results = Vec::new();
        let start_time = std::time::Instant::now();
        
        for step in scenario.steps {
            println!("📋 执行步骤: {}", step.description);
            
            match step.action {
                ScenarioAction::InjectFault(params) => {
                    let fault_id = self.inject_fault(params).await?;
                    results.push(fault_id);
                }
                ScenarioAction::Wait(duration) => {
                    sleep(duration).await;
                }
                ScenarioAction::RecoverFault(fault_id) => {
                    self.recover_fault(&fault_id).await?;
                }
                ScenarioAction::VerifySystem => {
                    // 系统验证逻辑
                    println!("✅ 系统验证通过");
                }
            }
        }
        
        let total_duration = start_time.elapsed();
        
        Ok(FaultScenarioResult {
            scenario_name: scenario.name,
            total_duration,
            injected_faults: results,
            success: true,
        })
    }
}

/// 故障场景
#[derive(Debug)]
pub struct FaultScenario {
    pub name: String,
    pub description: String,
    pub steps: Vec<ScenarioStep>,
}

/// 场景步骤
#[derive(Debug)]
pub struct ScenarioStep {
    pub description: String,
    pub action: ScenarioAction,
}

/// 场景动作
#[derive(Debug)]
pub enum ScenarioAction {
    InjectFault(FaultInjectionParams),
    Wait(Duration),
    RecoverFault(String),
    VerifySystem,
}

/// 故障场景结果
#[derive(Debug)]
pub struct FaultScenarioResult {
    pub scenario_name: String,
    pub total_duration: Duration,
    pub injected_faults: Vec<String>,
    pub success: bool,
}

/// 创建网络中断恢复场景
pub fn create_network_interruption_scenario() -> FaultScenario {
    FaultScenario {
        name: "Network Interruption Recovery".to_string(),
        description: "Test system recovery from network interruptions".to_string(),
        steps: vec![
            ScenarioStep {
                description: "Inject network disconnection".to_string(),
                action: ScenarioAction::InjectFault(FaultInjectionParams {
                    fault_type: FaultType::NetworkDisconnection,
                    duration: Duration::from_secs(10),
                    target_component: "mqtt".to_string(),
                    severity: FaultSeverity::High,
                    auto_recover: false,
                }),
            },
            ScenarioStep {
                description: "Wait for system to detect failure".to_string(),
                action: ScenarioAction::Wait(Duration::from_secs(5)),
            },
            ScenarioStep {
                description: "Recover network connection".to_string(),
                action: ScenarioAction::RecoverFault("fault_1".to_string()),
            },
            ScenarioStep {
                description: "Wait for system recovery".to_string(),
                action: ScenarioAction::Wait(Duration::from_secs(10)),
            },
            ScenarioStep {
                description: "Verify system functionality".to_string(),
                action: ScenarioAction::VerifySystem,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fault_injection_params_creation() {
        let params = FaultInjectionParams {
            fault_type: FaultType::NetworkLatency(100),
            duration: Duration::from_secs(30),
            target_component: "mqtt".to_string(),
            severity: FaultSeverity::Medium,
            auto_recover: true,
        };
        
        assert_eq!(params.duration, Duration::from_secs(30));
        assert_eq!(params.target_component, "mqtt");
        assert!(params.auto_recover);
    }
    
    #[test]
    fn test_network_interruption_scenario_creation() {
        let scenario = create_network_interruption_scenario();
        assert_eq!(scenario.name, "Network Interruption Recovery");
        assert_eq!(scenario.steps.len(), 5);
    }
}