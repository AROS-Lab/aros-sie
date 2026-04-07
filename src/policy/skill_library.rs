//! Two-tier skill library: task-skills (domain-specific) + meta-skills (domain-general).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::SkillTier;

/// A learned skill entry in the library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Unique name for this skill.
    pub name: String,
    /// Which tier this skill belongs to.
    pub tier: SkillTier,
    /// Human-readable description.
    pub description: String,
    /// When this skill was learned/last updated.
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Quality score from evaluation (higher is better).
    pub score: f64,
    /// Serialized strategy/template data.
    pub data: serde_json::Value,
}

/// Two-tier skill library maintaining task-skills and meta-skills separately.
pub struct SkillLibrary {
    skills: HashMap<String, Skill>,
}

impl SkillLibrary {
    /// Create a new empty skill library.
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    /// Add or update a skill.
    pub fn add_skill(&mut self, skill: Skill) {
        self.skills.insert(skill.name.clone(), skill);
    }

    /// Remove a skill by name.
    pub fn remove_skill(&mut self, name: &str) -> Option<Skill> {
        self.skills.remove(name)
    }

    /// Get a skill by name.
    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    /// List all skills of a given tier.
    pub fn by_tier(&self, tier: SkillTier) -> Vec<&Skill> {
        self.skills.values().filter(|s| s.tier == tier).collect()
    }

    /// Get the top k skills by score within a tier.
    pub fn top_k(&self, tier: SkillTier, k: usize) -> Vec<&Skill> {
        let mut skills: Vec<&Skill> = self.by_tier(tier);
        skills.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        skills.truncate(k);
        skills
    }

    /// Total number of skills.
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    /// Check if the library is empty.
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

impl Default for SkillLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_skill(name: &str, tier: SkillTier, score: f64) -> Skill {
        Skill {
            name: name.to_string(),
            tier,
            description: format!("Test skill: {}", name),
            updated_at: chrono::Utc::now(),
            score,
            data: serde_json::json!({}),
        }
    }

    #[test]
    fn test_add_and_get() {
        let mut lib = SkillLibrary::new();
        lib.add_skill(make_skill("rust_debugging", SkillTier::TaskSkill, 0.8));
        assert!(lib.get("rust_debugging").is_some());
        assert_eq!(lib.len(), 1);
    }

    #[test]
    fn test_by_tier() {
        let mut lib = SkillLibrary::new();
        lib.add_skill(make_skill("rust_debug", SkillTier::TaskSkill, 0.8));
        lib.add_skill(make_skill("decomposition", SkillTier::MetaSkill, 0.9));
        lib.add_skill(make_skill("sql_opt", SkillTier::TaskSkill, 0.7));

        assert_eq!(lib.by_tier(SkillTier::TaskSkill).len(), 2);
        assert_eq!(lib.by_tier(SkillTier::MetaSkill).len(), 1);
    }

    #[test]
    fn test_top_k() {
        let mut lib = SkillLibrary::new();
        lib.add_skill(make_skill("a", SkillTier::TaskSkill, 0.3));
        lib.add_skill(make_skill("b", SkillTier::TaskSkill, 0.9));
        lib.add_skill(make_skill("c", SkillTier::TaskSkill, 0.6));

        let top = lib.top_k(SkillTier::TaskSkill, 2);
        assert_eq!(top.len(), 2);
        assert!((top[0].score - 0.9).abs() < f64::EPSILON);
        assert!((top[1].score - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn test_remove() {
        let mut lib = SkillLibrary::new();
        lib.add_skill(make_skill("temp", SkillTier::TaskSkill, 0.5));
        assert!(lib.remove_skill("temp").is_some());
        assert!(lib.is_empty());
    }
}
