//! 控制通道和背压管理

use tokio::sync::broadcast;
use once_cell::sync::OnceCell;
use crate::{NormalizedUrl, EndpointError};

/// 控制消息类型
#[derive(Debug, Clone)]
pub enum ControlMsg {
    /// 暂停指定URL的连接获取
    Pause(NormalizedUrl),
    /// 恢复指定URL的连接获取  
    Resume(NormalizedUrl),
    /// 排空指定URL的连接池
    Drain(NormalizedUrl),
}

/// 全局控制通道发送端
static CONTROL_TX: OnceCell<broadcast::Sender<ControlMsg>> = OnceCell::new();

/// 初始化控制通道
pub fn init_control_channel() {
    let (tx, _) = broadcast::channel(1024);
    let _ = CONTROL_TX.set(tx);
}

/// 获取控制通道发送端
pub fn get_control_sender() -> Result<broadcast::Sender<ControlMsg>, EndpointError> {
    CONTROL_TX.get()
        .ok_or_else(|| EndpointError::Pool("Control channel not initialized".to_string()))
        .cloned()
}

/// 创建控制通道接收端
pub fn subscribe_control() -> Result<broadcast::Receiver<ControlMsg>, EndpointError> {
    let tx = get_control_sender()?;
    Ok(tx.subscribe())
}

/// 发送暂停信号
pub fn send_pause(url: NormalizedUrl) -> Result<(), EndpointError> {
    let tx = get_control_sender()?;
    tx.send(ControlMsg::Pause(url))
        .map_err(|e| EndpointError::Pool(format!("Failed to send pause: {}", e)))?;
    Ok(())
}

/// 发送恢复信号
pub fn send_resume(url: NormalizedUrl) -> Result<(), EndpointError> {
    let tx = get_control_sender()?;
    tx.send(ControlMsg::Resume(url))
        .map_err(|e| EndpointError::Pool(format!("Failed to send resume: {}", e)))?;
    Ok(())
}

/// 发送排空信号
pub fn send_drain(url: NormalizedUrl) -> Result<(), EndpointError> {
    let tx = get_control_sender()?;
    tx.send(ControlMsg::Drain(url))
        .map_err(|e| EndpointError::Pool(format!("Failed to send drain: {}", e)))?;
    Ok(())
}