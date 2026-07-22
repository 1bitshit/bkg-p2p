use crate::safety::types::*;
use anyhow::Result;

/// Task safety scanner
pub struct TaskScanner {
    config: SafetyConfig,
}

impl TaskScanner {
    pub fn new(config: SafetyConfig) -> Self {
        Self { config }
    }

    /// Scan a task for safety issues
    pub async fn scan(&self, task: &TaskDefinition) -> Result<ScanResult> {
        let mut issues = Vec::new();

        // Check for dangerous operations
        if let Some(ops) = &task.operations {
            for op in ops {
                if let Some(issue) = self.scan_operation(op) {
                    issues.push(issue);
                }
            }
        }

        // Check for resource limits
        if let Some(resources) = &task.resources {
            if resources.max_memory_mb > self.config.max_memory_mb {
                issues.push(SafetyIssue {
                    severity: Severity::Warning,
                    category: IssueCategory::ResourceLimit,
                    description: format!("Task requests {}MB memory, limit is {}MB",
                        resources.max_memory_mb, self.config.max_memory_mb),
                    recommendation: Some("Reduce memory request".to_string()),
                });
            }
        }

        Ok(ScanResult {
            safe: issues.is_empty(),
            issues,
            scan_timestamp: chrono::Utc::now(),
        })
    }

    /// Scan a single operation
    fn scan_operation(&self, op: &str) -> Option<SafetyIssue> {
        let dangerous_patterns = vec![
            "rm -rf",
            "sudo",
            "chmod 777",
            "curl | sh",
            "wget | sh",
        ];

        for pattern in dangerous_patterns {
            if op.contains(pattern) {
                return Some(SafetyIssue {
                    severity: Severity::Critical,
                    category: IssueCategory::DangerousOperation,
                    description: format!("Operation contains dangerous pattern: {}", pattern),
                    recommendation: Some("Remove or replace dangerous operation".to_string()),
                });
            }
        }

        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub safe: bool,
    pub issues: Vec<SafetyIssue>,
    pub scan_timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyIssue {
    pub severity: Severity,
    pub category: IssueCategory,
    pub description: String,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueCategory {
    DangerousOperation,
    ResourceLimit,
    PermissionEscalation,
    DataExfiltration,
    PolicyViolation,
}
