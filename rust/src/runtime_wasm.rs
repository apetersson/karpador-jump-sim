use crate::data::GameData;
use crate::player_policy::ActivePlayerPolicy;
use crate::rules::ApkRules;
use crate::start_config::{PolicyConfig, SimulationConfigFile};
use crate::walltime::{WallSimConfig, WallTimeSimulator};
use serde_json::json;
use wasm_bindgen::prelude::*;
use std::collections::BTreeMap;

const MASTER_LEAGUE_INDEX: u32 = 10;
const MINUTES_PER_DAY: f64 = 1440.0;

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
    let diamond_spending = summarize_diamond_spending(&value);
    let summary = json!({
        "plan": value.get("plan"),
        "outcome": value.get("outcome"),
        "wall_days": value.get("wall_days"),
        "sessions": value.get("sessions"),
        "warnings": value.get("warnings"),
        "league": value.pointer("/final_state/league"),
        "diamonds": value.pointer("/final_state/diamonds"),
        "days_to_master_league": derive_days_to_master_league(&value),
        "diamonds_spent_total": diamond_spending.0,
        "diamond_spending_by_kind": diamond_spending.1,
        "diamond_spending_by_item": diamond_spending.2,
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

fn summarize_diamond_spending(
    value: &serde_json::Value,
) -> (u32, Vec<serde_json::Value>, Vec<serde_json::Value>) {
    let purchases = value.get("purchases").and_then(|entry| entry.as_array());
    let mut total = 0_u32;
    let mut by_kind: BTreeMap<String, u32> = BTreeMap::new();
    let mut by_item: BTreeMap<(String, String), u32> = BTreeMap::new();

    if let Some(purchases) = purchases {
        for purchase in purchases {
            let kind = purchase
                .get("kind")
                .and_then(|value| value.as_str())
                .unwrap_or("support_or_decor")
                .to_string();
            let id = purchase
                .get("id")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown")
                .to_string();
            let Some(price) = purchase
                .get("price_diamonds")
                .and_then(|value| value.as_u64())
                .and_then(|value| u32::try_from(value).ok())
            else {
                continue;
            };
            total = total.saturating_add(price);
            *by_kind.entry(kind.clone()).or_insert(0) += price;
            *by_item.entry((kind, id)).or_insert(0) += price;
        }
    }

    let diamond_spending_by_kind = by_kind
        .into_iter()
        .map(|(kind, amount)| json!({ "kind": kind, "amount": amount }))
        .collect::<Vec<_>>();
    let diamond_spending_by_item = by_item
        .into_iter()
        .map(|((kind, id), amount)| {
            json!({
                "kind": kind,
                "id": id,
                "amount": amount
            })
        })
        .collect::<Vec<_>>();

    (total, diamond_spending_by_kind, diamond_spending_by_item)
}

fn derive_days_to_master_league(value: &serde_json::Value) -> Option<f64> {
    let final_league = value
        .pointer("/final_state/league")
        .and_then(|value| value.as_u64())?;
    let target_league = u64::from(MASTER_LEAGUE_INDEX);
    if final_league >= target_league {
        return value.get("wall_days").and_then(|value| value.as_f64());
    }

    let Some(action_log) = value.pointer("/action_log").and_then(|entry| entry.as_array()) else {
        return None;
    };

    for entry in action_log {
        if entry.get("event").and_then(|value| value.as_str()) != Some("league_fight") {
            continue;
        }
        let Some(detail) = entry.get("detail").and_then(|value| value.as_str()) else {
            continue;
        };
        let Some((_before, after)) = detail.split_once(" -> ") else {
            continue;
        };
        let Some((to_league, _)) = after.split_once('-') else {
            continue;
        };
        let Ok(to_league) = to_league.trim().parse::<u32>() else {
            continue;
        };
        if u64::from(to_league).saturating_sub(1) < target_league {
            continue;
        }
        let minute = entry.get("minute").and_then(|value| value.as_f64())?;
        return Some(minute / MINUTES_PER_DAY);
    }

    None
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
        assert!(parsed.get("diamonds_spent_total").is_some());
        assert!(parsed.get("days_to_master_league").is_some());
        assert!(parsed.get("diamond_spending_by_kind").is_some());
        assert_eq!(
            parsed.pointer("/diamond_spending_by_kind")
                .and_then(|value| value.as_array())
                .is_some(),
            true
        );
        assert!(
            parsed
                .get("diamond_spending_by_item")
                .and_then(|value| value.as_array())
                .is_some()
        );
    }
}
