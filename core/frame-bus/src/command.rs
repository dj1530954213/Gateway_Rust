/*!
# Command Routing and Processing

Provides command routing, validation, and execution management for the frame bus.
*/

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex};
use tokio::time::timeout;
use anyhow::{Context, Result};
use tracing::{info, warn, error, debug};

use crate::envelope::{CmdFrame, CmdAckFrame, FrameEnvelope, Value};
use crate::ring::FrameSender;

/// Command priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Emergency = 3,
}

impl From<i32> for CommandPriority {
    fn from(value: i32) -> Self {
        match value {
            0 => CommandPriority::Low,
            1 => CommandPriority::Normal,
            2 => CommandPriority::High,
            3 => CommandPriority::Emergency,
            _ => CommandPriority::Normal,
        }
    }
}

/// Command execution status
#[derive(Debug, Clone, PartialEq)]
pub enum CommandStatus {
    Pending,
    InProgress,
    Completed(CommandResult),
    TimedOut,
    Failed(String),
}

/// Command execution result
#[derive(Debug, Clone, PartialEq)]
pub struct CommandResult {
    pub success: bool,
    pub actual_value: Option<Value>,
    pub error_message: Option<String>,
    pub execution_time: Duration,
}

/// Pending command information
#[derive(Debug, Clone)]
pub struct PendingCommand {
    pub frame: CmdFrame,
    pub status: CommandStatus,
    pub submitted_at: Instant,
    pub timeout_at: Instant,
    pub retry_count: u32,
    pub max_retries: u32,
}

impl PendingCommand {
    /// Create new pending command
    pub fn new(frame: CmdFrame) -> Self {
        let now = Instant::now();
        let timeout_duration = Duration::from_millis(frame.timeout_ms as u64);
        
        Self {
            frame,
            status: CommandStatus::Pending,
            submitted_at: now,
            timeout_at: now + timeout_duration,
            retry_count: 0,
            max_retries: 3,
        }
    }
    
    /// Check if command has timed out
    pub fn is_timed_out(&self) -> bool {
        Instant::now() > self.timeout_at
    }
    
    /// Check if command can be retried
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }
    
    /// Mark for retry
    pub fn retry(&mut self) {
        self.retry_count += 1;
        self.status = CommandStatus::Pending;
    }
    
    /// Get priority
    pub fn priority(&self) -> CommandPriority {
        CommandPriority::from(self.frame.priority)
    }
}

/// Command queue that supports priority scheduling
pub struct CommandQueue {
    /// Priority queues (indexed by priority level)
    queues: [VecDeque<PendingCommand>; 4],
    
    /// Maximum queue sizes by priority
    max_sizes: [usize; 4],
    
    /// Current total size
    total_size: usize,
}

impl CommandQueue {
    /// Create new command queue
    pub fn new() -> Self {
        Self {
            queues: [
                VecDeque::new(), // Low
                VecDeque::new(), // Normal
                VecDeque::new(), // High
                VecDeque::new(), // Emergency
            ],
            max_sizes: [100, 500, 200, 50], // Conservative limits
            total_size: 0,
        }
    }
    
    /// Add command to queue
    pub fn push(&mut self, command: PendingCommand) -> Result<()> {
        let priority = command.priority();
        let queue_index = priority as usize;
        
        // Check queue limits
        if self.queues[queue_index].len() >= self.max_sizes[queue_index] {
            return Err(anyhow::anyhow!("Command queue full for priority {:?}", priority));
        }
        
        self.queues[queue_index].push_back(command);
        self.total_size += 1;
        
        Ok(())
    }
    
    /// Pop highest priority command
    pub fn pop(&mut self) -> Option<PendingCommand> {
        // Check emergency queue first, then high, normal, low
        for queue in self.queues.iter_mut().rev() {
            if let Some(command) = queue.pop_front() {
                self.total_size -= 1;
                return Some(command);
            }
        }
        None
    }
    
    /// Get total queue size
    pub fn len(&self) -> usize {
        self.total_size
    }
    
    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.total_size == 0
    }
    
    /// Get size by priority
    pub fn size_by_priority(&self, priority: CommandPriority) -> usize {
        self.queues[priority as usize].len()
    }
    
    /// Remove timed out commands
    pub fn remove_timed_out(&mut self) -> Vec<PendingCommand> {
        let mut timed_out = Vec::new();
        
        for queue in &mut self.queues {
            let mut i = 0;
            while i < queue.len() {
                if queue[i].is_timed_out() {
                    if let Some(mut cmd) = queue.remove(i) {
                        cmd.status = CommandStatus::TimedOut;
                        timed_out.push(cmd);
                        self.total_size -= 1;
                    }
                } else {
                    i += 1;
                }
            }
        }
        
        timed_out
    }
}

/// Command router and processor
pub struct CommandProcessor {
    /// Command queue
    queue: Arc<Mutex<CommandQueue>>,
    
    /// Pending commands (by command ID)
    pending_commands: Arc<RwLock<HashMap<u64, PendingCommand>>>,
    
    /// Command acknowledgment receivers
    ack_receivers: Arc<RwLock<HashMap<u64, mpsc::UnboundedSender<CmdAckFrame>>>>,
    
    /// Frame sender for publishing commands
    frame_sender: FrameSender,
    
    /// Command statistics
    stats: Arc<RwLock<CommandStats>>,
    
    /// Whether processor is running
    running: Arc<RwLock<bool>>,
}

impl CommandProcessor {
    /// Create new command processor
    pub fn new(frame_sender: FrameSender) -> Self {
        Self {
            queue: Arc::new(Mutex::new(CommandQueue::new())),
            pending_commands: Arc::new(RwLock::new(HashMap::new())),
            ack_receivers: Arc::new(RwLock::new(HashMap::new())),
            frame_sender,
            stats: Arc::new(RwLock::new(CommandStats::default())),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Start the command processor
    pub async fn start(&self) -> Result<()> {
        {
            let mut running = self.running.write().unwrap();
            if *running {
                return Ok(());
            }
            *running = true;
        }
        
        info!("Starting command processor");
        
        // Start processing loop
        let queue = self.queue.clone();
        let frame_sender = self.frame_sender.clone();
        let stats = self.stats.clone();
        let running = self.running.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(10));
            
            while *running.read().unwrap() {
                interval.tick().await;
                
                // Process commands from queue
                let mut queue_guard = queue.lock().await;
                
                // Remove timed out commands
                let timed_out = queue_guard.remove_timed_out();
                for cmd in timed_out {
                    warn!("Command timed out: {} for tag {}", cmd.frame.cmd_id, cmd.frame.tag);
                    stats.write().unwrap().timeouts += 1;
                }
                
                // Process next command
                if let Some(mut command) = queue_guard.pop() {
                    drop(queue_guard);
                    
                    command.status = CommandStatus::InProgress;
                    debug!("Processing command: {} for tag {}", 
                          command.frame.cmd_id, command.frame.tag);
                    
                    // Send command to frame bus
                    match FrameEnvelope::wrap_cmd(0, command.frame.clone()) {
                        Ok(envelope) => {
                            if frame_sender.send(envelope).is_ok() {
                                stats.write().unwrap().sent += 1;
                            } else {
                                stats.write().unwrap().failed += 1;
                                error!("Failed to send command to frame bus");
                            }
                        }
                        Err(e) => {
                            stats.write().unwrap().failed += 1;
                            error!("Failed to encode command: {}", e);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop the command processor
    pub fn stop(&self) {
        let mut running = self.running.write().unwrap();
        *running = false;
        info!("Command processor stopped");
    }
    
    /// Submit a command for execution
    pub async fn submit_command(&self, frame: CmdFrame) -> Result<mpsc::UnboundedReceiver<CmdAckFrame>> {
        let command = PendingCommand::new(frame.clone());
        let cmd_id = frame.cmd_id;
        
        // Create acknowledgment channel
        let (ack_tx, ack_rx) = mpsc::unbounded_channel();
        
        // Store acknowledgment receiver
        {
            let mut ack_receivers = self.ack_receivers.write().unwrap();
            ack_receivers.insert(cmd_id, ack_tx);
        }
        
        // Add to pending commands
        {
            let mut pending = self.pending_commands.write().unwrap();
            pending.insert(cmd_id, command.clone());
        }
        
        // Add to queue
        {
            let mut queue = self.queue.lock().await;
            queue.push(command).context("Failed to queue command")?;
        }
        
        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.submitted += 1;
        }
        
        info!("Command submitted: {} for tag {}", cmd_id, frame.tag);
        Ok(ack_rx)
    }
    
    /// Handle command acknowledgment
    pub fn handle_ack(&self, ack: CmdAckFrame) {
        let cmd_id = ack.cmd_id;
        
        debug!("Received command acknowledgment: {}", cmd_id);
        
        // Update pending command
        {
            let mut pending = self.pending_commands.write().unwrap();
            if let Some(mut command) = pending.remove(&cmd_id) {
                let result = CommandResult {
                    success: ack.success,
                    actual_value: ack.actual_value.clone(),
                    error_message: if ack.error_msg.is_empty() { 
                        None 
                    } else { 
                        Some(ack.error_msg.clone()) 
                    },
                    execution_time: Duration::from_nanos(
                        ack.timestamp.saturating_sub(command.frame.timestamp)
                    ),
                };
                
                command.status = CommandStatus::Completed(result);
                
                // Update statistics
                {
                    let mut stats = self.stats.write().unwrap();
                    if ack.success {
                        stats.succeeded += 1;
                    } else {
                        stats.failed += 1;
                    }
                }
            }
        }
        
        // Send acknowledgment to waiting receiver
        {
            let mut ack_receivers = self.ack_receivers.write().unwrap();
            if let Some(sender) = ack_receivers.remove(&cmd_id) {
                if sender.send(ack).is_err() {
                    debug!("Command acknowledgment receiver dropped: {}", cmd_id);
                }
            }
        }
    }
    
    /// Get command statistics
    pub fn stats(&self) -> CommandStats {
        self.stats.read().unwrap().clone()
    }
    
    /// Get queue statistics
    pub async fn queue_stats(&self) -> QueueStats {
        let queue = self.queue.lock().await;
        QueueStats {
            total: queue.len(),
            low: queue.size_by_priority(CommandPriority::Low),
            normal: queue.size_by_priority(CommandPriority::Normal),
            high: queue.size_by_priority(CommandPriority::High),
            emergency: queue.size_by_priority(CommandPriority::Emergency),
        }
    }
    
    /// Execute a command with timeout
    pub async fn execute_command_with_timeout(
        &self,
        frame: CmdFrame,
        timeout_duration: Duration,
    ) -> Result<CmdAckFrame> {
        let mut ack_rx = self.submit_command(frame.clone()).await?;
        
        match timeout(timeout_duration, ack_rx.recv()).await {
            Ok(Some(ack)) => Ok(ack),
            Ok(None) => Err(anyhow::anyhow!("Command acknowledgment channel closed")),
            Err(_) => Err(anyhow::anyhow!("Command execution timed out")),
        }
    }
}

/// Command processing statistics
#[derive(Debug, Clone, Default)]
pub struct CommandStats {
    pub submitted: u64,
    pub sent: u64,
    pub succeeded: u64,
    pub failed: u64,
    pub timeouts: u64,
}

/// Queue statistics
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub total: usize,
    pub low: usize,
    pub normal: usize,
    pub high: usize,
    pub emergency: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::Value;

    #[test]
    fn test_command_priority() {
        assert_eq!(CommandPriority::from(0), CommandPriority::Low);
        assert_eq!(CommandPriority::from(3), CommandPriority::Emergency);
        assert_eq!(CommandPriority::from(99), CommandPriority::Normal);
    }
    
    #[test]
    fn test_command_queue() {
        let mut queue = CommandQueue::new();
        assert!(queue.is_empty());
        
        // Create commands with different priorities
        let low_cmd = PendingCommand::new(
            CmdFrame::new("test.tag", Value::int(1), "test")
                .with_priority(CommandPriority::Low as i32)
        );
        
        let high_cmd = PendingCommand::new(
            CmdFrame::new("test.tag", Value::int(2), "test")
                .with_priority(CommandPriority::High as i32)
        );
        
        let emergency_cmd = PendingCommand::new(
            CmdFrame::new("test.tag", Value::int(3), "test")
                .with_priority(CommandPriority::Emergency as i32)
        );
        
        // Add commands
        queue.push(low_cmd).unwrap();
        queue.push(high_cmd).unwrap();
        queue.push(emergency_cmd).unwrap();
        
        assert_eq!(queue.len(), 3);
        
        // Should pop in priority order (Emergency, High, Low)
        let cmd1 = queue.pop().unwrap();
        assert_eq!(cmd1.frame.value.unwrap().to_i64(), Some(3));
        
        let cmd2 = queue.pop().unwrap();
        assert_eq!(cmd2.frame.value.unwrap().to_i64(), Some(2));
        
        let cmd3 = queue.pop().unwrap();
        assert_eq!(cmd3.frame.value.unwrap().to_i64(), Some(1));
        
        assert!(queue.is_empty());
    }
    
    #[test]
    fn test_pending_command() {
        let frame = CmdFrame::new("test.tag", Value::int(42), "test")
            .with_timeout_ms(1000);
        
        let mut cmd = PendingCommand::new(frame);
        assert_eq!(cmd.status, CommandStatus::Pending);
        assert!(!cmd.is_timed_out());
        assert!(cmd.can_retry());
        
        cmd.retry();
        assert_eq!(cmd.retry_count, 1);
    }
    
    #[test]
    fn test_command_result() {
        let result = CommandResult {
            success: true,
            actual_value: Some(Value::int(42)),
            error_message: None,
            execution_time: Duration::from_millis(100),
        };
        
        assert!(result.success);
        assert!(result.actual_value.is_some());
        assert!(result.error_message.is_none());
    }
}