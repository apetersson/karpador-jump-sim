use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

use crate::model::{Action, GameState, Kp};
use crate::rules::Rules;
use crate::strategy::Strategy;

#[derive(Clone, Copy, Debug)]
pub struct SimConfig {
    pub max_actions: u32,
    pub target_league: u32,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            max_actions: 1_000,
            target_league: 3,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub enum RunOutcome {
    TargetReached,
    ActionLimit,
}

#[derive(Clone, Debug, Serialize)]
pub struct SimResult {
    pub seed: u64,
    pub strategy: &'static str,
    pub rules: &'static str,
    pub outcome: RunOutcome,
    pub final_state: GameState,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExperimentReport {
    pub strategy: &'static str,
    pub rules: &'static str,
    pub runs: u32,
    pub target_league: u32,
    pub success_rate: f64,
    pub avg_actions: f64,
    pub avg_minutes: f64,
    pub avg_generation: f64,
    pub avg_rank: f64,
    pub avg_coins: f64,
    pub avg_kp: f64,
}

pub struct Simulator<R> {
    rules: R,
    config: SimConfig,
}

impl<R: Rules> Simulator<R> {
    pub fn new(rules: R, config: SimConfig) -> Self {
        Self { rules, config }
    }

    pub fn run<S: Strategy>(&self, seed: u64, mut strategy: S) -> SimResult {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut state = self.rules.initial_state();
        state.magikarp = self.rules.new_magikarp(&state, &mut rng);

        while state.action_count < self.config.max_actions
            && state.league < self.config.target_league
        {
            let action = strategy.decide(&state, &self.rules);
            self.apply_action(&mut state, action, &mut rng);
            state.action_count += 1;
            state.magikarp.level = self
                .rules
                .level_for_kp(state.magikarp.kp, state.magikarp.max_level);
        }

        let outcome = if state.league >= self.config.target_league {
            RunOutcome::TargetReached
        } else {
            RunOutcome::ActionLimit
        };

        SimResult {
            seed,
            strategy: strategy.name(),
            rules: self.rules.name(),
            outcome,
            final_state: state,
        }
    }

    pub fn experiment<S, F>(&self, runs: u32, seed: u64, mut make_strategy: F) -> ExperimentReport
    where
        S: Strategy,
        F: FnMut() -> S,
    {
        let mut successes = 0_u32;
        let mut total_actions = 0_f64;
        let mut total_minutes = 0_f64;
        let mut total_generation = 0_f64;
        let mut total_rank = 0_f64;
        let mut total_coins = 0_f64;
        let mut total_kp = 0_f64;
        let mut strategy_name = "";

        for i in 0..runs {
            let result = self.run(seed + i as u64, make_strategy());
            strategy_name = result.strategy;
            if matches!(result.outcome, RunOutcome::TargetReached) {
                successes += 1;
            }
            total_actions += result.final_state.action_count as f64;
            total_minutes += result.final_state.elapsed_minutes as f64;
            total_generation += result.final_state.generation as f64;
            total_rank += result.final_state.player_rank as f64;
            total_coins += result.final_state.coins as f64;
            total_kp += result.final_state.magikarp.kp as f64;
        }

        let denom = runs.max(1) as f64;
        ExperimentReport {
            strategy: strategy_name,
            rules: self.rules.name(),
            runs,
            target_league: self.config.target_league,
            success_rate: successes as f64 / denom,
            avg_actions: total_actions / denom,
            avg_minutes: total_minutes / denom,
            avg_generation: total_generation / denom,
            avg_rank: total_rank / denom,
            avg_coins: total_coins / denom,
            avg_kp: total_kp / denom,
        }
    }

    fn apply_action(&self, state: &mut GameState, action: Action, rng: &mut impl rand::Rng) {
        match action {
            Action::FeedAll => self.feed_all(state),
            Action::Train => self.train(state, rng),
            Action::Compete => self.compete(state, rng),
            Action::StartSession
            | Action::CollectGold
            | Action::ClaimAchievement
            | Action::UseSupportSkill(_)
            | Action::BuySupport(_)
            | Action::BuyDecor(_)
            | Action::UpgradeBerry(_)
            | Action::EatBerry(_, _)
            | Action::LeagueFight { .. }
            | Action::WaitUntil(_)
            | Action::EndSession => self.rest(state, 1),
            Action::BuyFoodUpgrade => {
                let cost = self.rules.food_upgrade_cost(state);
                if state.coins >= cost {
                    state.coins -= cost;
                    state.food_level += 1;
                } else {
                    self.rest(state, 5);
                }
            }
            Action::BuyTrainingUpgrade => {
                let cost = self.rules.training_upgrade_cost(state);
                if state.coins >= cost {
                    state.coins -= cost;
                    state.training_level += 1;
                } else {
                    self.rest(state, 5);
                }
            }
            Action::Rest { minutes } => self.rest(state, minutes),
            Action::RetireAndFish => self.retire_and_fish(state, rng),
        }
    }

    fn feed_all(&self, state: &mut GameState) {
        if state.food_available == 0 {
            self.rest(state, self.rules.food_respawn_minutes(state));
            return;
        }
        let food_count = state.food_available;
        let gained = self.rules.food_kp(state) * food_count as Kp;
        state.magikarp.kp = state.magikarp.kp.saturating_add(gained);
        state.magikarp.foods_eaten += food_count;
        state.food_available = 0;
        state.elapsed_minutes += 1;
    }

    fn train(&self, state: &mut GameState, rng: &mut impl rand::Rng) {
        if state.stamina == 0 {
            self.rest(state, self.rules.stamina_respawn_minutes());
            return;
        }
        state.stamina -= 1;
        let result = self.rules.training_result(rng);
        let gained = self.rules.training_kp(state, result);
        state.magikarp.kp = state.magikarp.kp.saturating_add(gained);
        state.magikarp.trainings_done += 1;
        state.elapsed_minutes += 4;
    }

    fn compete(&self, state: &mut GameState, rng: &mut impl rand::Rng) {
        let cheer_permyriad = match rng.random_range(0..100) {
            0..=74 => 10_000_u64,
            75..=94 => 11_500_u64,
            _ => 13_000_u64,
        };
        let own_jump = self.rules.jump_height_cm(state.magikarp.kp) * cheer_permyriad / 10_000;
        let opponent_jump = self.rules.opponent_jump_cm(state.league, state.competition);
        state.elapsed_minutes += 6;

        if own_jump >= opponent_jump {
            state.magikarp.wins += 1;
            state.coins += self
                .rules
                .competition_reward_coins(state.league, state.competition);
            state.competition += 1;
            if state.competition >= self.rules.competitions_per_league() {
                state.league += 1;
                state.competition = 0;
                state.player_rank += 1;
                state.max_stamina = state.max_stamina.max(3 + state.player_rank / 2);
                state.stamina = state.max_stamina;
            }
        } else if state.is_magikarp_maxed() {
            self.retire_and_fish(state, rng);
        } else {
            self.rest(state, 10);
        }
    }

    fn retire_and_fish(&self, state: &mut GameState, rng: &mut impl rand::Rng) {
        let xp = self.rules.retirement_rank_xp(state);
        state.trainer_exp = state.trainer_exp.saturating_add(xp as u128);
        state.player_rank = self.rules.player_rank_for_exp(state.trainer_exp);
        state.retirements += 1;
        state.generation += 1;
        state.magikarp = self.rules.new_magikarp(state, rng);
        state.food_available = state.max_food;
        state.stamina = state.max_stamina;
        state.elapsed_minutes += 12;
    }

    fn rest(&self, state: &mut GameState, minutes: u32) {
        state.elapsed_minutes += minutes as u64;
        let food_ticks = minutes / self.rules.food_respawn_minutes(state).max(1);
        state.food_available = (state.food_available + food_ticks).min(state.max_food);
        let stamina_ticks = minutes / self.rules.stamina_respawn_minutes().max(1);
        state.stamina = (state.stamina + stamina_ticks).min(state.max_stamina);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::ApproxRules;
    use crate::strategy::{EarlyCompeteStrategy, GreedyKpStrategy};

    #[test]
    fn greedy_run_is_deterministic_for_seed() {
        let sim = Simulator::new(ApproxRules, SimConfig::default());
        let a = sim.run(7, GreedyKpStrategy);
        let b = sim.run(7, GreedyKpStrategy);
        assert_eq!(a.final_state.magikarp.kp, b.final_state.magikarp.kp);
        assert_eq!(a.final_state.league, b.final_state.league);
    }

    #[test]
    fn early_compete_can_progress() {
        let sim = Simulator::new(
            ApproxRules,
            SimConfig {
                max_actions: 400,
                target_league: 1,
            },
        );
        let result = sim.run(42, EarlyCompeteStrategy::default());
        assert!(result.final_state.action_count <= 400);
    }
}
