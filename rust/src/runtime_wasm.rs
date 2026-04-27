use crate::data::GameData;
use crate::player_policy::ActivePlayerPolicy;
use crate::rules::ApkRules;
use crate::start_config::{PolicyConfig, SimulationConfigFile};
use crate::walltime::{WallSimConfig, WallTimeSimulator};
use serde_json::json;
use wasm_bindgen::prelude::*;

/// Run the wall-time simulator in the browser from a JSON start configuration.
#[wasm_bindgen]
pub fn run_wall_time_simulation(
    start_config_json: &str,
    seed: u64,
    max_actions: u32,
    max_days: u32,
    sessions_per_day: u8,
    target_league: u32,
) -> Result<String, String> {
    let data = GameData::apk_master();
    let simulation_config = parse_start_config(start_config_json)?;
    let policy_config = simulation_config
        .as_ref()
        .and_then(|config| config.policy.clone())
        .unwrap_or_default();
    validate_policy_config(&data, &policy_config)?;

    let mut plan_name = policy_config
        .purchase_plan
        .clone()
        .unwrap_or_else(|| "balanced".to_string());
    if plan_name.trim().is_empty() {
        plan_name = "balanced".to_string();
    }
    validate_purchase_plan(&data, &plan_name)?;

    let start_state = simulation_config
        .as_ref()
        .and_then(|config| config.start_state.as_ref());

    let plan = data.preset_plan(&plan_name);
    let mut policy = ActivePlayerPolicy::with_purchase_plan_and_config(
        plan,
        Some(&PolicyConfig {
            purchase_plan: None,
            ..policy_config.clone()
        }),
    );

    let sim = WallTimeSimulator::new(
        ApkRules::new(&data),
        data,
        WallSimConfig {
            max_actions,
            max_wall_days: max_days,
            max_sessions_per_day: sessions_per_day,
            target_league,
            karpador_loss_risk_max_level_percent: policy_config
                .karpador_loss_risk_max_level_percent
                .map(|value| value.min(100)),
        },
    );

    let result = sim
        .run_with_policy_from_config(seed, &mut policy, start_state)
        .map_err(|error| format!("simulation failed: {error}"))?;

    serde_json::to_string(&result).map_err(|error| format!("failed serializing simulation result: {error}"))
}

/// Run simulation and return a compact JSON summary for lightweight browser displays.
#[wasm_bindgen]
pub fn run_wall_time_simulation_summary(
    start_config_json: &str,
    seed: u64,
    max_actions: u32,
    max_days: u32,
    sessions_per_day: u8,
    target_league: u32,
) -> Result<String, String> {
    let result_json = run_wall_time_simulation(
        start_config_json,
        seed,
        max_actions,
        max_days,
        sessions_per_day,
        target_league,
    )?;
    let value: serde_json::Value = serde_json::from_str(&result_json)
        .map_err(|error| format!("invalid simulation payload: {error}"))?;
    let summary = json!({
        "plan": value.get("plan"),
        "outcome": value.get("outcome"),
        "wall_days": value.get("wall_days"),
        "sessions": value.get("sessions"),
        "warnings": value.get("warnings"),
        "league": value.pointer("/final_state/league"),
        "diamonds": value.pointer("/final_state/diamonds"),
    });
    serde_json::to_string_pretty(&summary)
        .map_err(|error| format!("failed serializing simulation summary: {error}"))
}

fn parse_start_config(start_config_json: &str) -> Result<Option<SimulationConfigFile>, String> {
    if start_config_json.trim().is_empty() {
        return Ok(None);
    }
    serde_json::from_str::<SimulationConfigFile>(start_config_json)
        .map_err(|error| format!("invalid start config json: {error}"))
        .map(Some)
}

fn validate_purchase_plan(data: &GameData, plan_name: &str) -> Result<(), String> {
    if !plan_name.trim_start().starts_with('[') {
        return Ok(());
    }
    let ids = serde_json::from_str::<Vec<String>>(plan_name)
        .map_err(|error| format!("invalid purchase_plan json: {error}"))?;
    for id in ids {
        if data.target_by_id(&id).is_none() {
            return Err(format!("unknown purchase_plan id in start_config: {id}"));
        }
    }
    Ok(())
}

fn validate_policy_config(data: &GameData, config: &PolicyConfig) -> Result<(), String> {
    if let Some(ids) = &config.allowed_berry_upgrades {
        for id in ids {
            if !data.berries.iter().any(|berry| berry.id == id) {
                return Err(format!("unknown allowed_berry_upgrades id in start_config: {id}"));
            }
        }
    }

    if let Some(ids) = &config.allowed_training_upgrades {
        for id in ids {
            if !data.trainings.iter().any(|training| training.id == id) {
                return Err(format!(
                    "unknown allowed_training_upgrades id in start_config: {id}"
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_wall_time_simulation_returns_result_json() {
        let payload = run_wall_time_simulation("{}", 42, 1000, 1, 4, 10)
            .expect("simulation should produce output");
        let parsed: serde_json::Value = serde_json::from_str(&payload)
            .expect("payload should be valid JSON");
        assert_eq!(parsed.get("plan"), Some(&serde_json::Value::String("balanced".to_string())));
        assert!(parsed.get("outcome").is_some());
    }

    #[test]
    fn run_wall_time_simulation_rejects_unknown_purchase_plan_ids() {
        let config = serde_json::json!({
            "policy": {
                "purchase_plan": "[\"not-existing\"]"
            }
        })
        .to_string();
        let error = run_wall_time_simulation(&config, 42, 1000, 1, 4, 10)
            .expect_err("should reject invalid plan");
        assert!(error.contains("unknown purchase_plan id"));
    }

    #[test]
    fn run_wall_time_simulation_summary_returns_summary_json() {
        let payload = run_wall_time_simulation_summary("{}", 42, 100_000, 1, 4, 10)
            .expect("simulation summary should be produced");
        let parsed: serde_json::Value = serde_json::from_str(&payload)
            .expect("summary payload should be valid JSON");
        assert!(parsed.get("plan").is_some());
        assert!(parsed.get("outcome").is_some());
        assert!(parsed.get("wall_days").is_some());
        assert!(parsed.get("sessions").is_some());
    }
}
