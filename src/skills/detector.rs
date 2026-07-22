use crate::skills::types::*;
use anyhow::Result;

/// Skill detector for automatic skill discovery
pub struct SkillDetector {
    config: SkillConfig,
}

impl SkillDetector {
    pub fn new(config: SkillConfig) -> Self {
        Self { config }
    }

    /// Detect skills from a task description
    pub async fn detect(&self, task: &str) -> Result<Vec<SkillMatch>> {
        let mut matches = Vec::new();

        // Simple keyword-based detection (TODO: Use embeddings for semantic matching)
        for skill in &self.config.available_skills {
            let score = self.match_score(task, &skill.keywords);
            if score > self.config.detection_threshold {
                matches.push(SkillMatch {
                    skill: skill.clone(),
                    confidence: score,
                    matched_keywords: self.matched_keywords(task, &skill.keywords),
                });
            }
        }

        // Sort by confidence descending
        matches.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        Ok(matches)
    }

    /// Calculate match score between task and keywords
    fn match_score(&self, task: &str, keywords: &[String]) -> f64 {
        let task_lower = task.to_lowercase();
        let mut matches = 0;

        for keyword in keywords {
            if task_lower.contains(&keyword.to_lowercase()) {
                matches += 1;
            }
        }

        if keywords.is_empty() {
            0.0
        } else {
            matches as f64 / keywords.len() as f64
        }
    }

    /// Get matched keywords
    fn matched_keywords(&self, task: &str, keywords: &[String]) -> Vec<String> {
        let task_lower = task.to_lowercase();
        keywords.iter()
            .filter(|k| task_lower.contains(&k.to_lowercase()))
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMatch {
    pub skill: Skill,
    pub confidence: f64,
    pub matched_keywords: Vec<String>,
}
