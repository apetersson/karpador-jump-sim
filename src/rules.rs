use rand::Rng;

use std::sync::Arc;

use crate::data::GameData;
use crate::model::{GameState, Kp, MagikarpState, WallClock};

#[derive(Clone, Copy, Debug)]
pub enum TrainingResult {
    Normal,
    Good,
    Great,
}

pub trait Rules {
    fn name(&self) -> &'static str;
    fn initial_state(&self) -> GameState;
    fn new_magikarp(&self, state: &GameState, rng: &mut impl Rng) -> MagikarpState;
    fn max_level_for_rank(&self, player_rank: u32) -> u32;
    fn player_rank_for_exp(&self, trainer_exp: u128) -> u32;
    fn need_kp_for_level(&self, level: u32) -> Kp;
    fn level_for_kp(&self, kp: Kp, max_level: u32) -> u32;
    fn jump_height_cm(&self, kp: Kp) -> u64;
    fn food_kp(&self, state: &GameState) -> Kp;
    fn training_result(&self, rng: &mut impl Rng) -> TrainingResult;
    fn training_kp(&self, state: &GameState, result: TrainingResult) -> Kp;
    fn food_upgrade_cost(&self, state: &GameState) -> u64;
    fn training_upgrade_cost(&self, state: &GameState) -> u64;
    fn food_respawn_minutes(&self, state: &GameState) -> u32;
    fn stamina_respawn_minutes(&self) -> u32;
    fn competitions_per_league(&self) -> u32;
    fn opponent_jump_cm(&self, league: u32, competition: u32) -> u64;
    fn competition_reward_coins(&self, league: u32, competition: u32) -> u64;
    fn retirement_rank_xp(&self, state: &GameState) -> u32;
}

#[derive(Clone, Debug, Default)]
pub struct ApproxRules;

impl ApproxRules {
    fn apply_bonus(kp: Kp, bonus_permyriad: u32) -> Kp {
        kp.saturating_mul(10_000 + bonus_permyriad as Kp) / 10_000
    }
}

impl Rules for ApproxRules {
    fn name(&self) -> &'static str {
        "approx-v0"
    }

    fn initial_state(&self) -> GameState {
        let max_level = self.max_level_for_rank(1);
        GameState {
            elapsed_minutes: 0,
            action_count: 0,
            clock: WallClock::default(),
            trainer_exp: 0,
            player_rank: 1,
            coins: 120,
            diamonds: 0,
            diamond_ledger: Vec::new(),
            candy: 0,
            training_sodas: 0,
            skill_herbs: 0,
            league_aids: 0,
            stamina: 3,
            max_stamina: 3,
            food_level: 1,
            training_level: 1,
            food_available: 3,
            max_food: 3,
            league: 0,
            competition: 0,
            generation: 1,
            retirements: 0,
            berries: Vec::new(),
            trainings: Vec::new(),
            supports: Vec::new(),
            decors: Vec::new(),
            pending_achievement_claims: 0,
            pending_diamond_rewards: Vec::new(),
            pending_coin_rewards: Vec::new(),
            pending_candy_rewards: Vec::new(),
            diamond_achievement_keys: Vec::new(),
            league_wins_total: 0,
            support_skill_uses: 0,
            items_used: 0,
            random_events_seen: 0,
            random_event_ids_seen: Vec::new(),
            random_event_retirements: 0,
            random_event_day: 0,
            training_random_events_today: 0,
            league_win_random_events_today: 0,
            league_loss_random_events_today: 0,
            discovered_patterns: 0,
            login_days_claimed: 0,
            home_treasure_ready_at: WallClock::SESSION_START as u64,
            home_random_event_ready_at: WallClock::SESSION_START as u64,
            next_food_spawn_at: WallClock::SESSION_START as u64 + 8,
            next_stamina_at: WallClock::SESSION_START as u64
                + self.stamina_respawn_minutes() as u64,
            kp_gain_buff_until: 0,
            kp_gain_buff_permyriad: 10_000,
            magikarp: MagikarpState {
                level: 1,
                max_level,
                kp: 0,
                individual_bonus_permyriad: 0,
                pattern_rarity: 1,
                foods_eaten: 0,
                trainings_done: 0,
                wins: 0,
                level_coin_bonus_claimed_to: 1,
            },
        }
    }

    fn new_magikarp(&self, state: &GameState, rng: &mut impl Rng) -> MagikarpState {
        let bonus_roll = rng.random_range(0..100);
        let individual_bonus_permyriad = match bonus_roll {
            0..=54 => 0,
            55..=84 => 500,
            85..=96 => 1_000,
            _ => 2_000,
        };
        let pattern_rarity = match rng.random_range(0..100) {
            0..=69 => 1,
            70..=89 => 2,
            90..=97 => 3,
            _ => 4,
        };

        MagikarpState {
            level: 1,
            max_level: self.max_level_for_rank(state.player_rank),
            kp: 0,
            individual_bonus_permyriad,
            pattern_rarity,
            foods_eaten: 0,
            trainings_done: 0,
            wins: 0,
            level_coin_bonus_claimed_to: 1,
        }
    }

    fn max_level_for_rank(&self, player_rank: u32) -> u32 {
        (player_rank + 10).min(100)
    }

    fn player_rank_for_exp(&self, trainer_exp: u128) -> u32 {
        1 + (trainer_exp / 8).min(99) as u32
    }

    fn need_kp_for_level(&self, level: u32) -> Kp {
        let l = level as Kp;
        // Placeholder curve. Replace with MagicarpData::getNeedPower once recovered.
        20 * l * l + 15 * l
    }

    fn level_for_kp(&self, kp: Kp, max_level: u32) -> u32 {
        (1..=max_level)
            .take_while(|level| kp >= self.need_kp_for_level(*level))
            .last()
            .unwrap_or(1)
    }

    fn jump_height_cm(&self, kp: Kp) -> u64 {
        // Placeholder for MagicarpData::calculateJumpHeight.
        ((kp as f64).sqrt() * 18.0 + 60.0).round() as u64
    }

    fn food_kp(&self, state: &GameState) -> Kp {
        let base = 14 + 8 * state.food_level as Kp;
        let rank = 1 + state.player_rank as Kp;
        let rarity_bonus = 1 + state.magikarp.pattern_rarity as Kp / 2;
        Self::apply_bonus(
            base * rank * rarity_bonus,
            state.magikarp.individual_bonus_permyriad,
        )
    }

    fn training_result(&self, rng: &mut impl Rng) -> TrainingResult {
        match rng.random_range(0..100) {
            0..=64 => TrainingResult::Normal,
            65..=91 => TrainingResult::Good,
            _ => TrainingResult::Great,
        }
    }

    fn training_kp(&self, state: &GameState, result: TrainingResult) -> Kp {
        let mult = match result {
            TrainingResult::Normal => 100,
            TrainingResult::Good => 160,
            TrainingResult::Great => 260,
        };
        let base = (80 + 42 * state.training_level as Kp) * (1 + state.player_rank as Kp);
        Self::apply_bonus(base * mult / 100, state.magikarp.individual_bonus_permyriad)
    }

    fn food_upgrade_cost(&self, state: &GameState) -> u64 {
        90 * 3_u64.pow(state.food_level.saturating_sub(1).min(8))
    }

    fn training_upgrade_cost(&self, state: &GameState) -> u64 {
        140 * 3_u64.pow(state.training_level.saturating_sub(1).min(8))
    }

    fn food_respawn_minutes(&self, state: &GameState) -> u32 {
        8_u32.saturating_sub(state.player_rank / 8).max(3)
    }

    fn stamina_respawn_minutes(&self) -> u32 {
        30
    }

    fn competitions_per_league(&self) -> u32 {
        5
    }

    fn opponent_jump_cm(&self, league: u32, competition: u32) -> u64 {
        let step = league * self.competitions_per_league() + competition + 1;
        120 + (step as u64 * step as u64 * 24)
    }

    fn competition_reward_coins(&self, league: u32, competition: u32) -> u64 {
        45 + (league as u64 * 70) + (competition as u64 * 22)
    }

    fn retirement_rank_xp(&self, state: &GameState) -> u32 {
        1 + state.magikarp.level / 4 + state.magikarp.wins / 2
    }
}

#[derive(Clone, Debug)]
pub struct ApkRules {
    data: Arc<GameData>,
}

impl ApkRules {
    pub fn new(data: &GameData) -> Self {
        Self {
            data: Arc::new(data.clone()),
        }
    }

    fn training_multiplier(result: TrainingResult) -> u128 {
        match result {
            TrainingResult::Normal => 100,
            TrainingResult::Good => 150,
            TrainingResult::Great => 350,
        }
    }
}

impl Rules for ApkRules {
    fn name(&self) -> &'static str {
        "apk-masterdata-v1"
    }

    fn initial_state(&self) -> GameState {
        let max_level = self.max_level_for_rank(1);
        GameState {
            elapsed_minutes: 0,
            action_count: 0,
            clock: WallClock::default(),
            trainer_exp: 0,
            player_rank: 1,
            coins: 50,
            diamonds: 0,
            diamond_ledger: Vec::new(),
            candy: 0,
            training_sodas: 0,
            skill_herbs: 0,
            league_aids: 0,
            stamina: 3,
            max_stamina: 3,
            food_level: 1,
            training_level: 1,
            food_available: 0,
            max_food: self.data.economy.home_food_max.value,
            league: 0,
            competition: 0,
            generation: 1,
            retirements: 0,
            berries: Vec::new(),
            trainings: Vec::new(),
            supports: Vec::new(),
            decors: Vec::new(),
            pending_achievement_claims: 0,
            pending_diamond_rewards: Vec::new(),
            pending_coin_rewards: Vec::new(),
            pending_candy_rewards: Vec::new(),
            diamond_achievement_keys: Vec::new(),
            league_wins_total: 0,
            support_skill_uses: 0,
            items_used: 0,
            random_events_seen: 0,
            random_event_ids_seen: Vec::new(),
            random_event_retirements: 0,
            random_event_day: 0,
            training_random_events_today: 0,
            league_win_random_events_today: 0,
            league_loss_random_events_today: 0,
            discovered_patterns: 0,
            login_days_claimed: 0,
            home_treasure_ready_at: WallClock::SESSION_START as u64,
            home_random_event_ready_at: WallClock::SESSION_START as u64,
            next_food_spawn_at: WallClock::SESSION_START as u64
                + self.data.economy.food_respawn_minutes.value as u64,
            next_stamina_at: WallClock::SESSION_START as u64
                + self.stamina_respawn_minutes() as u64,
            kp_gain_buff_until: 0,
            kp_gain_buff_permyriad: 10_000,
            magikarp: MagikarpState {
                level: 1,
                max_level,
                kp: 0,
                individual_bonus_permyriad: 0,
                pattern_rarity: 1,
                foods_eaten: 0,
                trainings_done: 0,
                wins: 0,
                level_coin_bonus_claimed_to: 1,
            },
        }
    }

    fn new_magikarp(&self, state: &GameState, rng: &mut impl Rng) -> MagikarpState {
        let bonus_roll = rng.random_range(0..100);
        let individual_bonus_permyriad = match bonus_roll {
            0..=54 => 0,
            55..=84 => 500,
            85..=96 => 1_000,
            _ => 2_000,
        };
        let pattern_rarity = match rng.random_range(0..100) {
            0..=69 => 1,
            70..=89 => 2,
            90..=97 => 3,
            _ => 4,
        };

        MagikarpState {
            level: 1,
            max_level: self.max_level_for_rank(state.player_rank),
            kp: 0,
            individual_bonus_permyriad,
            pattern_rarity,
            foods_eaten: 0,
            trainings_done: 0,
            wins: 0,
            level_coin_bonus_claimed_to: 1,
        }
    }

    fn max_level_for_rank(&self, player_rank: u32) -> u32 {
        self.data
            .breeder_ranks
            .iter()
            .find(|row| row.rank == player_rank)
            .or_else(|| self.data.breeder_ranks.last())
            .map(|row| row.magikarp_max_rank.value)
            .unwrap_or_else(|| (player_rank + 10).min(100))
    }

    fn player_rank_for_exp(&self, trainer_exp: u128) -> u32 {
        self.data
            .breeder_ranks
            .iter()
            .take_while(|row| trainer_exp >= row.need_exp.value)
            .last()
            .map(|row| row.rank)
            .unwrap_or(1)
    }

    fn need_kp_for_level(&self, level: u32) -> Kp {
        self.data
            .magikarp_ranks
            .iter()
            .find(|row| row.rank == level)
            .or_else(|| self.data.magikarp_ranks.last())
            .map(|row| row.need_kp.value)
            .unwrap_or(0)
    }

    fn level_for_kp(&self, kp: Kp, max_level: u32) -> u32 {
        self.data
            .magikarp_ranks
            .iter()
            .take_while(|row| row.rank <= max_level && kp >= row.need_kp.value)
            .last()
            .map(|row| row.rank)
            .unwrap_or(1)
    }

    fn jump_height_cm(&self, kp: Kp) -> u64 {
        let Some(first) = self.data.jump_curve.first() else {
            return kp.min(u64::MAX as u128) as u64;
        };
        if kp <= first.need_kp.value {
            return first.height.value;
        }
        for pair in self.data.jump_curve.windows(2) {
            let a = &pair[0];
            let b = &pair[1];
            if kp <= b.need_kp.value {
                let span = b.need_kp.value.saturating_sub(a.need_kp.value).max(1);
                let offset = kp.saturating_sub(a.need_kp.value);
                return a.height.value
                    + ((b.height.value.saturating_sub(a.height.value)) as u128 * offset / span)
                        as u64;
            }
        }
        self.data
            .jump_curve
            .last()
            .map(|point| point.height.value)
            .unwrap_or(0)
    }

    fn food_kp(&self, state: &GameState) -> Kp {
        self.data.berry_jp("food_1", state.food_level).unwrap_or(2)
    }

    fn training_result(&self, rng: &mut impl Rng) -> TrainingResult {
        match rng.random_range(0..100) {
            0..=70 => TrainingResult::Normal,
            71..=94 => TrainingResult::Good,
            _ => TrainingResult::Great,
        }
    }

    fn training_kp(&self, state: &GameState, result: TrainingResult) -> Kp {
        self.data
            .training_jp("training_1", state.training_level)
            .unwrap_or(35)
            * Self::training_multiplier(result)
            / 100
    }

    fn food_upgrade_cost(&self, state: &GameState) -> u64 {
        self.data
            .food_upgrade_costs
            .get(state.food_level.saturating_sub(1) as usize)
            .and_then(|row| row.first())
            .map(|cost| cost.value)
            .unwrap_or(u64::MAX)
    }

    fn training_upgrade_cost(&self, state: &GameState) -> u64 {
        self.data
            .training_upgrade_costs
            .get(state.training_level.saturating_sub(1) as usize)
            .and_then(|row| row.first())
            .map(|cost| cost.value)
            .unwrap_or(u64::MAX)
    }

    fn food_respawn_minutes(&self, _state: &GameState) -> u32 {
        self.data.economy.food_respawn_minutes.value
    }

    fn stamina_respawn_minutes(&self) -> u32 {
        self.data.economy.stamina_respawn_minutes.value
    }

    fn competitions_per_league(&self) -> u32 {
        self.data
            .leagues
            .first()
            .map(|league| league.competitions.len() as u32)
            .unwrap_or(5)
    }

    fn opponent_jump_cm(&self, league: u32, competition: u32) -> u64 {
        self.data
            .leagues
            .get(league as usize)
            .and_then(|league| league.competitions.get(competition as usize))
            .map(|competition| competition.opponent_jump_cm.value)
            .unwrap_or(u64::MAX)
    }

    fn competition_reward_coins(&self, league: u32, competition: u32) -> u64 {
        self.data
            .leagues
            .get(league as usize)
            .and_then(|league| league.competitions.get(competition as usize))
            .map(|competition| competition.win_reward_coins.value)
            .unwrap_or(0)
    }

    fn retirement_rank_xp(&self, state: &GameState) -> u32 {
        self.data
            .magikarp_ranks
            .iter()
            .find(|row| row.rank == state.magikarp.level)
            .or_else(|| self.data.magikarp_ranks.last())
            .map(|row| row.retirement_breeder_exp.value.min(u32::MAX as u128) as u32)
            .unwrap_or(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magikarp_max_level_is_trainer_rank_plus_ten_capped_at_100() {
        let rules = ApproxRules;
        assert_eq!(rules.max_level_for_rank(1), 11);
        assert_eq!(rules.max_level_for_rank(15), 25);
        assert_eq!(rules.max_level_for_rank(90), 100);
        assert_eq!(rules.max_level_for_rank(100), 100);
    }
}
