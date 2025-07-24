/*!
# Automation Controller

Intelligent automation and rule-based control systems.
*/

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{ComponentStatus, HealthLevel};

/// Automation controller
pub struct AutomationController {
    rules: Arc<RwLock<HashMap<String, AutomationRule>>>,
    status: Arc<RwLock<ComponentStatus>>,
}

/// Automation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub conditions: Vec<RuleCondition>,
    pub actions: Vec<RuleAction>,
    pub priority: u32,
    pub metadata: HashMap<String, String>,
}

/// Rule condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    pub field: String,
    pub operator: ComparisonOperator,
    pub value: serde_json::Value,
}

/// Comparison operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    Contains,
    StartsWith,
    EndsWith,
}

/// Rule action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleAction {
    pub action_type: ActionType,
    pub target: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Action type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    SendCommand,
    SetValue,
    SendAlert,
    LogEvent,
    ExecuteScript,
    TriggerWorkflow,
}

/// Rule execution context
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub data: HashMap<String, serde_json::Value>,
    pub timestamp: std::time::SystemTime,
    pub source: String,
}

impl AutomationController {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            status: Arc::new(RwLock::new(ComponentStatus::default())),
        })
    }

    pub async fn start(&self) -> Result<()> {
        let mut status = self.status.write().await;
        status.running = true;
        status.health = HealthLevel::Good;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut status = self.status.write().await;
        status.running = false;
        Ok(())
    }

    pub async fn add_rule(&self, rule: AutomationRule) -> Result<()> {
        let mut rules = self.rules.write().await;
        rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    pub async fn evaluate_rules(&self, context: &ExecutionContext) -> Result<Vec<RuleAction>> {
        let rules = self.rules.read().await;
        let mut triggered_actions = Vec::new();

        for rule in rules.values() {
            if rule.enabled && self.evaluate_conditions(rule, context) {
                triggered_actions.extend(rule.actions.clone());
            }
        }

        Ok(triggered_actions)
    }

    pub async fn get_status(&self) -> Result<ComponentStatus> {
        let status = self.status.read().await;
        Ok(status.clone())
    }

    fn evaluate_conditions(&self, rule: &AutomationRule, context: &ExecutionContext) -> bool {
        rule.conditions.iter().all(|condition| {
            if let Some(field_value) = context.data.get(&condition.field) {
                self.compare_values(field_value, &condition.operator, &condition.value)
            } else {
                false
            }
        })
    }

    fn compare_values(
        &self,
        left: &serde_json::Value,
        operator: &ComparisonOperator,
        right: &serde_json::Value,
    ) -> bool {
        match operator {
            ComparisonOperator::Equal => left == right,
            ComparisonOperator::NotEqual => left != right,
            ComparisonOperator::GreaterThan => {
                if let (Some(l), Some(r)) = (left.as_f64(), right.as_f64()) {
                    l > r
                } else {
                    false
                }
            }
            ComparisonOperator::LessThan => {
                if let (Some(l), Some(r)) = (left.as_f64(), right.as_f64()) {
                    l < r
                } else {
                    false
                }
            }
            // Add other operators as needed
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_automation_controller() {
        let controller = AutomationController::new().await.unwrap();
        assert!(controller.start().await.is_ok());
    }
}