//! 背压控制

use tokio::sync::broadcast;
use once_cell::sync::OnceCell;

/// 控制消息
#[derive(Debug, Clone)]
pub enum ControlMsg {
    /// 暂停生产者
    Pause,
    /// 恢复生产者
    Resume,
    /// 排空缓冲区
    Drain,
}

/// 全局控制通道
static CONTROL_TX: OnceCell<broadcast::Sender<ControlMsg>> = OnceCell::new();

/// 初始化控制通道
pub fn init_control() {
    let (tx, _) = broadcast::channel(64);
    let _ = CONTROL_TX.set(tx);
}

/// 获取控制发送端
pub fn get_control_sender() -> Option<broadcast::Sender<ControlMsg>> {
    CONTROL_TX.get().cloned()
}

/// 订阅控制消息
pub fn subscribe_control() -> Option<broadcast::Receiver<ControlMsg>> {
    CONTROL_TX.get().map(|tx| tx.subscribe())
}

/// 发送暂停信号
pub fn send_pause() {
    if let Some(tx) = get_control_sender() {
        let _ = tx.send(ControlMsg::Pause);
    }
}

/// 发送恢复信号
pub fn send_resume() {
    if let Some(tx) = get_control_sender() {
        let _ = tx.send(ControlMsg::Resume);
    }
}