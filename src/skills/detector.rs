use serde::{Deserialize, Serialize};

use crate::skills::SkillManifest;

/// Skill detector for automatic skill discovery based on task descriptions.
pub struct SkillDetector {
    available_skills: Vec<SkillManifest>,
    detection_threshold: f64,
}

impl SkillDetector {
    pub fn new(available_skills: Vec<SkillManifest>, detection_threshold: f64) -> Self {
        Self {
            available_skills,
            detection_threshold,
        }
    }

    /// Detect skills that match a task description.
    pub async fn detect(&self, task: &str) -> Vec<SkillMatch> {
        let mut matches = Vec::new();

        for skill in &self.available_skills {
            let keywords: Vec<String> = skill
                .activation
                .tags
                .iter()
                .map(|t| t.to_lowercase())
                .chain(skill.name.split(|c: char| !c.is_alphanumeric()).map(|s| s.to_lowercase()))
                .filter(|s| !s.is_empty())
                .collect();

            let score = self.match_score(task, &keywords);
            if score > self.detection_threshold {
                let matched = self.matched_keywords(task, &keywords);
                matches.push(SkillMatch {
                    name: skill.name.clone(),
                    description: skill.description.clone(),
                    confidence: score,
                    matched_keywords: matched,
                });
            }
        }

        matches.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matches
    }

    fn match_score(&self, task: &str, keywords: &[String]) -> f64 {
        if keywords.is_empty() {
            return 0.0;
        }
        let task_lower = task.to_lowercase();
        let hits = keywords
            .iter()
            .filter(|k| task_lower.contains(k.as_str()))
            .count();
        hits as f64 / keywords.len() as f64
    }

    fn matched_keywords(&self, task: &str, keywords: &[String]) -> Vec<String> {
        let task_lower = task.to_lowercase();
        keywords
            .iter()
            .filter(|k| task_lower.contains(k.as_str()))
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMatch {
    pub name: String,
    pub description: String,
    pub confidence: f64,
    pub matched_keywords: Vec<String>,
}
