use crate::model::{Action, GameState};
use crate::rules::Rules;

pub trait Strategy {
    fn name(&self) -> &'static str;
    fn decide(&mut self, state: &GameState, rules: &impl Rules) -> Action;
}

#[derive(Clone, Debug, Default)]
pub struct GreedyKpStrategy;

impl Strategy for GreedyKpStrategy {
    fn name(&self) -> &'static str {
        "greedy-kp"
    }

    fn decide(&mut self, state: &GameState, _rules: &impl Rules) -> Action {
        if state.food_available > 0 {
            Action::FeedAll
        } else if state.stamina > 0 {
            Action::Train
        } else if state.is_magikarp_maxed() {
            Action::Compete
        } else {
            Action::Rest { minutes: 10 }
        }
    }
}

#[derive(Clone, Debug)]
pub struct EarlyCompeteStrategy {
    pub safety_margin_permyriad: u32,
}

impl Default for EarlyCompeteStrategy {
    fn default() -> Self {
        Self {
            safety_margin_permyriad: 11_000,
        }
    }
}

impl Strategy for EarlyCompeteStrategy {
    fn name(&self) -> &'static str {
        "early-compete"
    }

    fn decide(&mut self, state: &GameState, rules: &impl Rules) -> Action {
        let own = rules.jump_height_cm(state.magikarp.kp);
        let opponent = rules.opponent_jump_cm(state.league, state.competition);
        if own * 10_000 >= opponent * self.safety_margin_permyriad as u64 {
            Action::Compete
        } else if state.food_available > 0 {
            Action::FeedAll
        } else if state.stamina > 0 {
            Action::Train
        } else {
            Action::Rest { minutes: 10 }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ShopRoiStrategy;

impl Strategy for ShopRoiStrategy {
    fn name(&self) -> &'static str {
        "shop-roi"
    }

    fn decide(&mut self, state: &GameState, rules: &impl Rules) -> Action {
        let own = rules.jump_height_cm(state.magikarp.kp);
        let opponent = rules.opponent_jump_cm(state.league, state.competition);
        if own * 10_000 >= opponent * 10_800 {
            return Action::Compete;
        }

        let food_cost = rules.food_upgrade_cost(state);
        let training_cost = rules.training_upgrade_cost(state);
        let food_roi = rules.food_kp(state) as f64 / food_cost.max(1) as f64;
        let train_roi = rules.training_kp(state, crate::rules::TrainingResult::Normal) as f64
            / training_cost.max(1) as f64;

        if state.coins >= food_cost || state.coins >= training_cost {
            if food_roi >= train_roi && state.coins >= food_cost {
                return Action::BuyFoodUpgrade;
            }
            if state.coins >= training_cost {
                return Action::BuyTrainingUpgrade;
            }
        }

        if state.food_available > 0 {
            Action::FeedAll
        } else if state.stamina > 0 {
            Action::Train
        } else {
            Action::Rest { minutes: 10 }
        }
    }
}
