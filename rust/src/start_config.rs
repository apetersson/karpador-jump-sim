use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SimulationConfigFile {
    #[serde(default)]
    pub start_state: Option<StartStateConfig>,
    #[serde(default)]
    pub policy: Option<PolicyConfig>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct StartStateConfig {
    #[serde(default)]
    pub player_rank: Option<u32>,
    #[serde(default, alias = "coins")]
    pub gold: Option<u64>,
    #[serde(default)]
    pub diamonds: Option<u32>,
    #[serde(default)]
    pub league: Option<u32>,
    #[serde(default)]
    pub competition: Option<u32>,
    #[serde(default)]
    pub generation: Option<u32>,
    #[serde(default)]
    pub retirements: Option<u32>,
    #[serde(default)]
    pub magikarp_level: Option<u32>,
    #[serde(default)]
    pub magikarp_kp: Option<u128>,
    #[serde(default)]
    pub candy: Option<u32>,
    #[serde(default)]
    pub training_sodas: Option<u32>,
    #[serde(default)]
    pub skill_herbs: Option<u32>,
    #[serde(default)]
    pub league_aids: Option<u32>,
    #[serde(default)]
    pub owned_supports: Vec<String>,
    #[serde(default)]
    pub owned_decors: Vec<String>,
    #[serde(default)]
    pub berry_levels: BTreeMap<String, u32>,
    #[serde(default)]
    pub training_levels: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct PolicyConfig {
    #[serde(default)]
    pub purchase_plan: Option<String>,
    #[serde(default)]
    pub allow_training_sodas: Option<bool>,
    #[serde(default)]
    pub allow_skill_herbs: Option<bool>,
    #[serde(default)]
    pub allow_support_upgrades: Option<bool>,
    #[serde(default)]
    pub training_upgrade_share: Option<u32>,
    #[serde(default)]
    pub allowed_berry_upgrades: Option<Vec<String>>,
    #[serde(default)]
    pub allowed_training_upgrades: Option<Vec<String>>,
    #[serde(default)]
    pub karpador_loss_risk_max_level_percent: Option<u32>,
    #[serde(default)]
    pub sessions_per_day: Option<u8>,
}
