use serde::Serialize;

use crate::data::{DecorEffect, GameData, SupportSkill};
use crate::model::{PurchaseKind, PurchasePlan, PurchaseTarget};
use crate::rules::Rules;
use crate::walltime::{WallRunOutcome, WallSimConfig, WallTimeSimulator};

#[derive(Clone, Debug)]
pub struct OptimizerConfig {
    pub runs: u32,
    pub seed: u64,
    pub beam_width: usize,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            runs: 100,
            seed: 42,
            beam_width: 10,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct OptimizerReport {
    pub dataset: &'static str,
    pub runs_per_plan: u32,
    pub evaluated_plans: usize,
    pub ranked_plans: Vec<PlanScore>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PlanScore {
    pub rank: usize,
    pub plan: String,
    pub targets: Vec<PurchaseTarget>,
    pub success_rate: f64,
    pub progress_score: f64,
    pub mean_days: f64,
    pub median_days: f64,
    pub p10_days: f64,
    pub p90_days: f64,
    pub mean_player_rank: f64,
    pub median_player_rank: f64,
    pub mean_league_progress: f64,
    pub mean_diamonds_unspent: f64,
    pub mean_diamonds_spent: f64,
    pub mean_first_purchase_day: Option<f64>,
}

pub fn optimize_purchase_plans<R: Rules + Clone>(
    rules: R,
    data: GameData,
    sim_config: WallSimConfig,
    config: OptimizerConfig,
) -> OptimizerReport {
    let candidates = data.purchase_candidates();
    let plans = if candidates.len() <= 9 {
        exhaustive_plans(candidates)
    } else {
        beam_plans(&data, candidates, config.beam_width.max(1))
    };
    let mut scores = Vec::with_capacity(plans.len());

    for plan in &plans {
        let simulator = WallTimeSimulator::new(rules.clone(), data.clone(), sim_config.clone());
        scores.push(score_plan(&simulator, plan.clone(), &config));
    }

    scores.sort_by(|a, b| {
        b.success_rate
            .total_cmp(&a.success_rate)
            .then_with(|| b.progress_score.total_cmp(&a.progress_score))
            .then_with(|| a.median_days.total_cmp(&b.median_days))
            .then_with(|| a.mean_days.total_cmp(&b.mean_days))
    });
    scores.truncate(config.beam_width.max(1));
    for (rank, score) in scores.iter_mut().enumerate() {
        score.rank = rank + 1;
    }

    OptimizerReport {
        dataset: data.name,
        runs_per_plan: config.runs,
        evaluated_plans: plans.len(),
        ranked_plans: scores,
        warnings: data.audit().warnings,
    }
}

fn score_plan<R: Rules>(
    simulator: &WallTimeSimulator<R>,
    plan: PurchasePlan,
    config: &OptimizerConfig,
) -> PlanScore {
    let mut days = Vec::with_capacity(config.runs as usize);
    let mut player_ranks = Vec::with_capacity(config.runs as usize);
    let mut successes = 0_u32;
    let mut diamonds_unspent = 0_f64;
    let mut diamonds_spent = 0_f64;
    let mut first_purchase_days = Vec::new();
    let mut league_progress = 0_f64;

    for i in 0..config.runs {
        let result = simulator.run(config.seed + i as u64, plan.clone());
        if matches!(result.outcome, WallRunOutcome::TargetReached) {
            successes += 1;
        }
        days.push(result.wall_days);
        player_ranks.push(result.final_state.player_rank as f64);
        league_progress +=
            result.final_state.league as f64 * 100.0 + result.final_state.competition as f64;
        diamonds_unspent += result.final_state.diamonds as f64;
        diamonds_spent += result
            .purchases
            .iter()
            .map(|purchase| purchase.price_diamonds as f64)
            .sum::<f64>();
        if let Some(first) = result.purchases.first() {
            first_purchase_days.push(first.minute as f64 / 1_440.0);
        }
    }

    days.sort_by(f64::total_cmp);
    player_ranks.sort_by(f64::total_cmp);
    let denom = config.runs.max(1) as f64;
    let success_rate = successes as f64 / denom;
    let mean_player_rank = player_ranks.iter().sum::<f64>() / denom;
    let median_player_rank = percentile(&player_ranks, 0.50);
    let mean_league_progress = league_progress / denom;
    let median_days = percentile(&days, 0.50);
    let mean_days = days.iter().sum::<f64>() / denom;
    let progress_score = success_rate * 1_000_000.0
        + median_player_rank * 10_000.0
        + mean_player_rank * 100.0
        + mean_league_progress
        - median_days / 1_000.0;
    PlanScore {
        rank: 0,
        plan: plan.name,
        targets: plan.targets,
        success_rate,
        progress_score,
        mean_days,
        median_days,
        p10_days: percentile(&days, 0.10),
        p90_days: percentile(&days, 0.90),
        mean_player_rank,
        median_player_rank,
        mean_league_progress,
        mean_diamonds_unspent: diamonds_unspent / denom,
        mean_diamonds_spent: diamonds_spent / denom,
        mean_first_purchase_day: (!first_purchase_days.is_empty())
            .then(|| first_purchase_days.iter().sum::<f64>() / first_purchase_days.len() as f64),
    }
}

fn percentile(values: &[f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let index = ((values.len() - 1) as f64 * p).round() as usize;
    values[index]
}

fn exhaustive_plans(candidates: Vec<PurchaseTarget>) -> Vec<PurchasePlan> {
    let mut plans = Vec::new();
    let mut current = Vec::new();
    let mut used = vec![false; candidates.len()];
    permute(&candidates, &mut used, &mut current, &mut plans);
    plans
}

fn permute(
    candidates: &[PurchaseTarget],
    used: &mut [bool],
    current: &mut Vec<PurchaseTarget>,
    plans: &mut Vec<PurchasePlan>,
) {
    if current.len() == candidates.len() {
        let name = current
            .iter()
            .map(|target| target.id.as_str())
            .collect::<Vec<_>>()
            .join(">");
        plans.push(PurchasePlan {
            name,
            targets: current.clone(),
        });
        return;
    }

    for i in 0..candidates.len() {
        if used[i] {
            continue;
        }
        used[i] = true;
        current.push(candidates[i].clone());
        permute(candidates, used, current, plans);
        current.pop();
        used[i] = false;
    }
}

fn beam_plans(
    data: &GameData,
    candidates: Vec<PurchaseTarget>,
    beam_width: usize,
) -> Vec<PurchasePlan> {
    let mut orders: Vec<(String, Vec<PurchaseTarget>)> = Vec::new();

    let mut cheapest = candidates.clone();
    cheapest.sort_by_key(|target| data.purchase_price(target));
    orders.push(("cheapest-first".to_string(), cheapest.clone()));

    let mut expensive = cheapest.clone();
    expensive.reverse();
    orders.push(("expensive-first".to_string(), expensive));

    let mut support_first = cheapest.clone();
    support_first.sort_by_key(|target| {
        (
            match target.kind {
                PurchaseKind::Support => 0,
                PurchaseKind::Decor => 1,
            },
            data.purchase_price(target),
        )
    });
    orders.push(("support-cheapest-first".to_string(), support_first));

    let mut decor_first = cheapest.clone();
    decor_first.sort_by_key(|target| {
        (
            match target.kind {
                PurchaseKind::Decor => 0,
                PurchaseKind::Support => 1,
            },
            data.purchase_price(target),
        )
    });
    orders.push(("decor-cheapest-first".to_string(), decor_first));

    let mut support_expensive = cheapest.clone();
    support_expensive.sort_by_key(|target| {
        (
            match target.kind {
                PurchaseKind::Support => 0,
                PurchaseKind::Decor => 1,
            },
            std::cmp::Reverse(data.purchase_price(target)),
        )
    });
    orders.push(("support-expensive-first".to_string(), support_expensive));

    let mut decor_expensive = cheapest.clone();
    decor_expensive.sort_by_key(|target| {
        (
            match target.kind {
                PurchaseKind::Decor => 0,
                PurchaseKind::Support => 1,
            },
            std::cmp::Reverse(data.purchase_price(target)),
        )
    });
    orders.push(("decor-expensive-first".to_string(), decor_expensive));

    let mut kp_first = cheapest.clone();
    kp_first.sort_by_key(|target| {
        (
            std::cmp::Reverse(kp_priority_score(data, target)),
            data.purchase_price(target),
        )
    });
    orders.push(("kp-first".to_string(), kp_first));

    let mut economy_first = cheapest;
    economy_first.sort_by_key(|target| {
        (
            std::cmp::Reverse(economy_priority_score(data, target)),
            data.purchase_price(target),
        )
    });
    orders.push(("economy-first".to_string(), economy_first));

    let mut expensive_tail = candidates.clone();
    expensive_tail.sort_by_key(|target| std::cmp::Reverse(data.purchase_price(target)));
    let mut first_candidates = candidates
        .iter()
        .filter(|target| matches!(target.kind, PurchaseKind::Support))
        .cloned()
        .collect::<Vec<_>>();
    let mut first_decors = candidates
        .iter()
        .filter(|target| matches!(target.kind, PurchaseKind::Decor))
        .cloned()
        .collect::<Vec<_>>();
    first_decors.sort_by_key(|target| {
        std::cmp::Reverse(
            kp_priority_score(data, target)
                .max(economy_priority_score(data, target))
                .max(data.purchase_price(target).saturating_mul(8)),
        )
    });
    first_decors.truncate(beam_width.max(1));
    for decor in first_decors {
        if !first_candidates.contains(&decor) {
            first_candidates.push(decor);
        }
    }
    for candidate in &first_candidates {
        let mut order = vec![candidate.clone()];
        order.extend(
            expensive_tail
                .iter()
                .filter(|target| *target != candidate)
                .cloned(),
        );
        orders.push((format!("first-{}-then-expensive", candidate.id), order));
    }
    if let Some(gardevoir) = candidates.iter().find(|target| target.id == "gardevoir") {
        for second in candidates.iter().filter(|target| target.id != "gardevoir") {
            let mut order = vec![gardevoir.clone(), second.clone()];
            order.extend(
                expensive_tail
                    .iter()
                    .filter(|target| target.id != "gardevoir" && target.id != second.id)
                    .cloned(),
            );
            orders.push((
                format!("gardevoir-then-{}-then-expensive", second.id),
                order,
            ));
        }
    }
    if let (Some(gardevoir), Some(rowlet)) = (
        candidates.iter().find(|target| target.id == "gardevoir"),
        candidates.iter().find(|target| target.id == "rowlet"),
    ) {
        for third in candidates
            .iter()
            .filter(|target| target.id != "gardevoir" && target.id != "rowlet")
        {
            let mut order = vec![gardevoir.clone(), rowlet.clone(), third.clone()];
            order.extend(
                expensive_tail
                    .iter()
                    .filter(|target| {
                        target.id != "gardevoir" && target.id != "rowlet" && target.id != third.id
                    })
                    .cloned(),
            );
            orders.push((format!("gardevoir-rowlet-then-{}", third.id), order));
        }
    }
    if let (Some(gardevoir), Some(rowlet), Some(slowpoke)) = (
        candidates.iter().find(|target| target.id == "gardevoir"),
        candidates.iter().find(|target| target.id == "rowlet"),
        candidates.iter().find(|target| target.id == "slowpoke"),
    ) {
        for fourth in candidates.iter().filter(|target| {
            target.id != "gardevoir" && target.id != "rowlet" && target.id != "slowpoke"
        }) {
            let mut order = vec![
                gardevoir.clone(),
                rowlet.clone(),
                slowpoke.clone(),
                fourth.clone(),
            ];
            order.extend(
                expensive_tail
                    .iter()
                    .filter(|target| {
                        target.id != "gardevoir"
                            && target.id != "rowlet"
                            && target.id != "slowpoke"
                            && target.id != fourth.id
                    })
                    .cloned(),
            );
            orders.push((
                format!("gardevoir-rowlet-slowpoke-then-{}", fourth.id),
                order,
            ));
        }
    }
    if let (Some(gardevoir), Some(rowlet), Some(slowpoke), Some(parasect)) = (
        candidates.iter().find(|target| target.id == "gardevoir"),
        candidates.iter().find(|target| target.id == "rowlet"),
        candidates.iter().find(|target| target.id == "slowpoke"),
        candidates
            .iter()
            .find(|target| target.id == "parasect_puffballs"),
    ) {
        for fifth in candidates.iter().filter(|target| {
            target.id != "gardevoir"
                && target.id != "rowlet"
                && target.id != "slowpoke"
                && target.id != "parasect_puffballs"
        }) {
            let mut order = vec![
                gardevoir.clone(),
                rowlet.clone(),
                slowpoke.clone(),
                parasect.clone(),
                fifth.clone(),
            ];
            order.extend(
                expensive_tail
                    .iter()
                    .filter(|target| {
                        target.id != "gardevoir"
                            && target.id != "rowlet"
                            && target.id != "slowpoke"
                            && target.id != "parasect_puffballs"
                            && target.id != fifth.id
                    })
                    .cloned(),
            );
            orders.push((
                format!("gardevoir-rowlet-slowpoke-parasect-then-{}", fifth.id),
                order,
            ));
        }
    }

    let prefix_lengths = [
        3_usize,
        5,
        beam_width.max(1),
        beam_width.saturating_mul(2),
        usize::MAX,
    ];
    let mut plans = Vec::new();
    for (name, order) in orders {
        for prefix in prefix_lengths {
            let take = prefix.min(order.len());
            if take == 0 {
                continue;
            }
            let targets = order.iter().take(take).cloned().collect::<Vec<_>>();
            let plan_name = if take == order.len() {
                name.clone()
            } else {
                format!("{name}-top{take}")
            };
            push_plan_if_new(&mut plans, plan_name, targets);
        }
    }
    let mut expensive_top = candidates.clone();
    expensive_top.sort_by_key(|target| std::cmp::Reverse(data.purchase_price(target)));
    expensive_top.truncate(5);
    push_permuted_plans("expensive-top5-permutation", &expensive_top, &mut plans);
    plans
}

fn push_plan_if_new(plans: &mut Vec<PurchasePlan>, name: String, targets: Vec<PurchaseTarget>) {
    if !plans
        .iter()
        .any(|plan: &PurchasePlan| plan.targets == targets)
    {
        plans.push(PurchasePlan { name, targets });
    }
}

fn push_permuted_plans(name: &str, candidates: &[PurchaseTarget], plans: &mut Vec<PurchasePlan>) {
    let mut used = vec![false; candidates.len()];
    let mut current = Vec::with_capacity(candidates.len());
    permute_prefixed(name, candidates, &mut used, &mut current, plans);
}

fn permute_prefixed(
    name: &str,
    candidates: &[PurchaseTarget],
    used: &mut [bool],
    current: &mut Vec<PurchaseTarget>,
    plans: &mut Vec<PurchasePlan>,
) {
    if current.len() == candidates.len() {
        let suffix = current
            .iter()
            .map(|target| target.id.as_str())
            .collect::<Vec<_>>()
            .join(">");
        push_plan_if_new(plans, format!("{name}:{suffix}"), current.clone());
        return;
    }
    for i in 0..candidates.len() {
        if used[i] {
            continue;
        }
        used[i] = true;
        current.push(candidates[i].clone());
        permute_prefixed(name, candidates, used, current, plans);
        current.pop();
        used[i] = false;
    }
}

fn kp_priority_score(data: &GameData, target: &PurchaseTarget) -> u32 {
    match target.kind {
        PurchaseKind::Support => data
            .supports
            .iter()
            .find(|support| support.id == target.id)
            .map(|support| match support.skill {
                SupportSkill::KpBoost { .. } => 10_000,
                SupportSkill::KpFlat { .. } => 8_000,
                SupportSkill::TrainingGreat => 6_500,
                SupportSkill::Food { .. } => 5_500,
                SupportSkill::RecoverSkills => 4_000,
                SupportSkill::Stamina { .. } => 3_500,
                _ => 0,
            })
            .unwrap_or(0),
        PurchaseKind::Decor => data
            .decors
            .iter()
            .find(|decor| decor.id == target.id)
            .map(|decor| match decor.effect {
                DecorEffect::KpPermyriad(value)
                | DecorEffect::TrainingPermyriad(value)
                | DecorEffect::SkillKpPermyriad(value) => value,
                DecorEffect::FoodCapacity(extra) => 4_000 + extra * 500,
                DecorEffect::TrainingEventPermyriad(value) => value / 2,
                _ => 0,
            })
            .unwrap_or(0),
    }
}

fn economy_priority_score(data: &GameData, target: &PurchaseTarget) -> u32 {
    match target.kind {
        PurchaseKind::Support => data
            .supports
            .iter()
            .find(|support| support.id == target.id)
            .map(|support| match support.skill {
                SupportSkill::Coins { .. } => 8_000,
                SupportSkill::Item { .. } => 5_000,
                SupportSkill::RecoverSkills => 3_500,
                _ => 0,
            })
            .unwrap_or(0),
        PurchaseKind::Decor => data
            .decors
            .iter()
            .find(|decor| decor.id == target.id)
            .map(|decor| match decor.effect {
                DecorEffect::CoinPermyriad(value)
                | DecorEffect::EventCoinPermyriad(value)
                | DecorEffect::TreasureCoinPermyriad(value)
                | DecorEffect::LevelUpCoinPermyriad(value) => value,
                DecorEffect::TrainerExpPermyriad(value) => value / 2,
                _ => 0,
            })
            .unwrap_or(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameData;
    use crate::rules::ApproxRules;

    #[test]
    fn optimizer_is_deterministic() {
        let data = GameData::approx_v1();
        let config = OptimizerConfig {
            runs: 2,
            seed: 99,
            beam_width: 2,
        };
        let a = optimize_purchase_plans(
            ApproxRules,
            data.clone(),
            WallSimConfig {
                max_wall_days: 20,
                max_actions: 10_000,
                ..WallSimConfig::default()
            },
            config.clone(),
        );
        let b = optimize_purchase_plans(
            ApproxRules,
            data,
            WallSimConfig {
                max_wall_days: 20,
                max_actions: 10_000,
                ..WallSimConfig::default()
            },
            config,
        );
        assert_eq!(a.ranked_plans[0].plan, b.ranked_plans[0].plan);
        assert_eq!(a.ranked_plans[0].median_days, b.ranked_plans[0].median_days);
    }
}
