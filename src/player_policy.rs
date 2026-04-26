use crate::model::{GameState, PurchasePlan};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WallSessionAction {
    CollectGold,
    ClaimAchievement,
    HomeRandomEvent,
    UpgradeSupport(usize),
    UseSupport(usize),
    UseTrainingSoda,
    UseSkillHerb,
    BuyNextPlanItem,
    UpgradeBerry(usize),
    Train,
    EatBerries { index: usize, count: u32 },
    LeagueFight { intentional_loss: bool },
}

#[derive(Clone, Debug, Default)]
pub struct PolicySessionState {
    pub berries_eaten_before_fight: u32,
    pub ate_rest_after_block: bool,
}

pub struct PolicyDecisionView<'a> {
    pub state: &'a GameState,
    pub plan: &'a PurchasePlan,
    pub plan_cursor: usize,
    pub session: &'a PolicySessionState,
    pub home_treasure_available: bool,
    pub home_random_event_ready: bool,
    pub kp_gain_buff_active: bool,
    pub next_support_upgrade: Option<usize>,
    pub ready_support: Option<usize>,
    pub support_on_cooldown: bool,
    pub can_buy_next_plan_item: bool,
    pub next_equal_berry_upgrade: Option<usize>,
    pub next_min_berries_to_eat: Option<(usize, u32)>,
    pub next_rest_berries_to_eat: Option<(usize, u32)>,
    pub should_take_intentional_loss: bool,
    pub expected_jump_clears_current_opponent: bool,
    pub forced_league_progress_after_max: bool,
}

pub trait WallTimePolicy {
    fn name(&self) -> &'static str;

    fn begin_session(&mut self) -> PolicySessionState {
        PolicySessionState::default()
    }

    fn decide(&mut self, view: &PolicyDecisionView<'_>) -> Option<WallSessionAction>;
}

#[derive(Clone, Debug, Default)]
pub struct ActivePlayerPolicy;

impl WallTimePolicy for ActivePlayerPolicy {
    fn name(&self) -> &'static str {
        "active-player-v1"
    }

    fn decide(&mut self, view: &PolicyDecisionView<'_>) -> Option<WallSessionAction> {
        if view.home_treasure_available {
            return Some(WallSessionAction::CollectGold);
        }
        if view.state.pending_achievement_claims > 0 {
            return Some(WallSessionAction::ClaimAchievement);
        }
        if view.home_random_event_ready {
            return Some(WallSessionAction::HomeRandomEvent);
        }
        if view.kp_gain_buff_active {
            if view.state.stamina > 0 {
                return Some(WallSessionAction::Train);
            }
            if let Some((index, count)) = view.next_rest_berries_to_eat {
                return Some(WallSessionAction::EatBerries { index, count });
            }
        }
        if let Some(index) = view.next_support_upgrade {
            return Some(WallSessionAction::UpgradeSupport(index));
        }
        if let Some(index) = view.ready_support {
            return Some(WallSessionAction::UseSupport(index));
        }
        if view.state.skill_herbs > 0 && view.support_on_cooldown {
            return Some(WallSessionAction::UseSkillHerb);
        }
        if view.can_buy_next_plan_item {
            return Some(WallSessionAction::BuyNextPlanItem);
        }
        if let Some(index) = view.next_equal_berry_upgrade {
            return Some(WallSessionAction::UpgradeBerry(index));
        }
        if view.state.stamina > 0 {
            return Some(WallSessionAction::Train);
        }
        if view.state.training_sodas > 0 && view.state.stamina < view.state.max_stamina {
            return Some(WallSessionAction::UseTrainingSoda);
        }
        if view.session.berries_eaten_before_fight < 3 {
            if let Some((index, count)) = view.next_min_berries_to_eat {
                return Some(WallSessionAction::EatBerries { index, count });
            }
        }

        if view.should_take_intentional_loss {
            return Some(WallSessionAction::LeagueFight {
                intentional_loss: true,
            });
        }
        if view.expected_jump_clears_current_opponent || view.forced_league_progress_after_max {
            return Some(WallSessionAction::LeagueFight {
                intentional_loss: false,
            });
        }
        if !view.session.ate_rest_after_block {
            if let Some((index, count)) = view.next_rest_berries_to_eat {
                return Some(WallSessionAction::EatBerries { index, count });
            }
        }
        None
    }
}
