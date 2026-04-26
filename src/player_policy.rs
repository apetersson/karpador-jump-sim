use crate::model::{GameState, PurchaseKind, PurchasePlan, PurchaseTarget};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum LeagueFightIntent {
    TryWin,
    IntentionallyLose,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub enum WallAction {
    CollectHomeTreasure,
    ClaimAchievements,
    TriggerHomeEvent,
    UseSupport { support_id: String },
    UpgradeSupport { support_id: String },
    UseTrainingSoda,
    UseSkillHerb,
    BuyShopItem { target: PurchaseTarget },
    UpgradeBerry { berry_id: String },
    UpgradeTraining { training_id: String },
    Train,
    EatBerries { berry_id: String, count: u32 },
    LeagueFight { intent: LeagueFightIntent },
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct AvailableAction {
    pub action: WallAction,
    pub reason: &'static str,
    pub cost_coins: Option<u64>,
}

pub struct DecisionContext<'a> {
    pub state: &'a GameState,
    pub available_actions: &'a [AvailableAction],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyDecision {
    Execute(WallAction),
    EndSession,
}

pub trait WallTimePolicy {
    fn name(&self) -> &'static str;

    fn result_plan_name(&self) -> String {
        self.name().to_string()
    }

    fn begin_session(&mut self) {}

    fn choose_action(&mut self, context: &DecisionContext<'_>) -> PolicyDecision;

    fn observe_action(&mut self, _before: &GameState, _action: &WallAction, _after: &GameState) {}
}

#[derive(Clone, Debug)]
pub struct ActivePlayerPolicy {
    purchase_plan: PurchasePlan,
    training_upgrade_share_permyriad: u32,
    berry_upgrade_coins_spent: u64,
    training_upgrade_coins_spent: u64,
    plan_cursor: usize,
    berries_eaten_before_fight: u32,
    ate_rest_after_block: bool,
    league_loss_done: Vec<bool>,
    must_max_after_intentional_loss: bool,
}

impl Default for ActivePlayerPolicy {
    fn default() -> Self {
        Self::with_purchase_plan(PurchasePlan {
            name: "none".to_string(),
            targets: Vec::new(),
        })
    }
}

impl ActivePlayerPolicy {
    pub fn with_purchase_plan(purchase_plan: PurchasePlan) -> Self {
        Self {
            purchase_plan,
            training_upgrade_share_permyriad: 0,
            berry_upgrade_coins_spent: 0,
            training_upgrade_coins_spent: 0,
            plan_cursor: 0,
            berries_eaten_before_fight: 0,
            ate_rest_after_block: false,
            league_loss_done: Vec::new(),
            must_max_after_intentional_loss: false,
        }
    }

    pub fn with_purchase_plan_and_training_share(
        purchase_plan: PurchasePlan,
        training_upgrade_share_permyriad: u32,
    ) -> Self {
        Self {
            training_upgrade_share_permyriad: training_upgrade_share_permyriad.min(10_000),
            ..Self::with_purchase_plan(purchase_plan)
        }
    }

    fn find_action(
        actions: &[AvailableAction],
        predicate: impl Fn(&WallAction) -> bool,
    ) -> Option<WallAction> {
        actions
            .iter()
            .find(|available| predicate(&available.action))
            .map(|available| available.action.clone())
    }

    fn find_available(
        actions: &[AvailableAction],
        predicate: impl Fn(&AvailableAction) -> bool,
    ) -> Option<WallAction> {
        actions
            .iter()
            .find(|available| predicate(available))
            .map(|available| available.action.clone())
    }

    fn buy_next_plan_item(
        &mut self,
        state: &GameState,
        actions: &[AvailableAction],
    ) -> Option<WallAction> {
        while let Some(target) = self.purchase_plan.targets.get(self.plan_cursor) {
            if target_owned(state, target) {
                self.plan_cursor += 1;
                continue;
            }
            return Self::find_action(
                actions,
                |action| matches!(action, WallAction::BuyShopItem { target: available } if available == target),
            );
        }
        None
    }

    fn eat_berries_action(actions: &[AvailableAction], wanted: u32) -> Option<WallAction> {
        actions
            .iter()
            .filter_map(|available| match &available.action {
                WallAction::EatBerries { berry_id, count } if *count <= wanted => {
                    Some((berry_id, *count, available.action.clone()))
                }
                _ => None,
            })
            .max_by_key(|(berry_id, count, _)| (*count, std::cmp::Reverse(berry_id.as_str())))
            .map(|(_, _, action)| action)
    }

    fn eat_all_berries_action(actions: &[AvailableAction]) -> Option<WallAction> {
        actions
            .iter()
            .filter_map(|available| match &available.action {
                WallAction::EatBerries { berry_id, count } => {
                    Some((berry_id, *count, available.action.clone()))
                }
                _ => None,
            })
            .max_by_key(|(berry_id, count, _)| (*count, std::cmp::Reverse(berry_id.as_str())))
            .map(|(_, _, action)| action)
    }

    fn equal_berry_upgrade_action(
        state: &GameState,
        actions: &[AvailableAction],
    ) -> Option<WallAction> {
        actions
            .iter()
            .filter_map(|available| match &available.action {
                WallAction::UpgradeBerry { berry_id } => state
                    .berries
                    .iter()
                    .find(|berry| berry.id == berry_id && berry.pair_group == "primary_equal")
                    .map(|berry| (berry.level, berry.id, available.action.clone())),
                _ => None,
            })
            .min_by_key(|(level, id, _)| (*level, *id))
            .map(|(_, _, action)| action)
    }

    fn starter_training_upgrade_action(actions: &[AvailableAction]) -> Option<WallAction> {
        actions
            .iter()
            .filter_map(|available| match &available.action {
                WallAction::UpgradeTraining { training_id } => {
                    Some((training_id.as_str(), available.action.clone()))
                }
                _ => None,
            })
            .min_by_key(|(id, _)| *id)
            .map(|(_, action)| action)
    }

    fn upgrade_with_ratio(
        &self,
        state: &GameState,
        actions: &[AvailableAction],
    ) -> Option<WallAction> {
        let berry = Self::equal_berry_upgrade_action(state, actions);
        let training = Self::starter_training_upgrade_action(actions);
        match (berry, training) {
            (Some(berry), Some(training)) => self.best_ratio_action(actions, berry, training),
            (Some(berry), None) => Some(berry),
            (None, Some(training)) if self.training_upgrade_share_permyriad > 0 => Some(training),
            (None, _) => None,
        }
    }

    fn best_ratio_action(
        &self,
        actions: &[AvailableAction],
        berry: WallAction,
        training: WallAction,
    ) -> Option<WallAction> {
        let berry_cost = action_coin_cost(actions, &berry)?;
        let training_cost = action_coin_cost(actions, &training)?;
        let target = self.training_upgrade_share_permyriad.min(10_000) as u128;
        let berry_error = ratio_error(
            self.training_upgrade_coins_spent as u128,
            self.berry_upgrade_coins_spent.saturating_add(berry_cost) as u128,
            target,
        );
        let training_error = ratio_error(
            self.training_upgrade_coins_spent
                .saturating_add(training_cost) as u128,
            self.berry_upgrade_coins_spent as u128,
            target,
        );
        if training_error < berry_error {
            Some(training)
        } else {
            Some(berry)
        }
    }

    fn ensure_league_slots(&mut self, league: u32) {
        let len = league as usize + 1;
        if self.league_loss_done.len() < len {
            self.league_loss_done.resize(len, false);
        }
    }

    fn should_take_intentional_loss(&mut self, state: &GameState) -> bool {
        self.ensure_league_slots(state.league);
        !self.league_loss_done[state.league as usize] && !self.must_max_after_intentional_loss
    }
}

impl WallTimePolicy for ActivePlayerPolicy {
    fn name(&self) -> &'static str {
        "active-player-v1"
    }

    fn result_plan_name(&self) -> String {
        self.purchase_plan.name.clone()
    }

    fn begin_session(&mut self) {
        self.berries_eaten_before_fight = 0;
        self.ate_rest_after_block = false;
    }

    fn choose_action(&mut self, context: &DecisionContext<'_>) -> PolicyDecision {
        let actions = context.available_actions;
        let state = context.state;

        for required in [
            WallAction::CollectHomeTreasure,
            WallAction::ClaimAchievements,
            WallAction::TriggerHomeEvent,
        ] {
            if let Some(action) = Self::find_action(actions, |action| action == &required) {
                return PolicyDecision::Execute(action);
            }
        }

        if state.kp_gain_buff_permyriad > 10_000 && state.kp_gain_buff_until > state.now() {
            if let Some(action) =
                Self::find_action(actions, |action| matches!(action, WallAction::Train))
            {
                return PolicyDecision::Execute(action);
            }
            if let Some(action) = Self::eat_all_berries_action(actions) {
                return PolicyDecision::Execute(action);
            }
        }

        for predicate in [
            |action: &WallAction| matches!(action, WallAction::UpgradeSupport { .. }),
            |action: &WallAction| matches!(action, WallAction::UseSupport { .. }),
            |action: &WallAction| matches!(action, WallAction::UseSkillHerb),
        ] {
            if let Some(action) = Self::find_action(actions, predicate) {
                return PolicyDecision::Execute(action);
            }
        }

        if let Some(action) = self.buy_next_plan_item(state, actions) {
            return PolicyDecision::Execute(action);
        }
        if let Some(action) = self.upgrade_with_ratio(state, actions) {
            return PolicyDecision::Execute(action);
        }
        if let Some(action) =
            Self::find_action(actions, |action| matches!(action, WallAction::Train))
        {
            return PolicyDecision::Execute(action);
        }
        if let Some(action) = Self::find_action(actions, |action| {
            matches!(action, WallAction::UseTrainingSoda)
        }) {
            return PolicyDecision::Execute(action);
        }
        if self.berries_eaten_before_fight < 3 {
            if let Some(action) =
                Self::eat_berries_action(actions, 3 - self.berries_eaten_before_fight)
            {
                return PolicyDecision::Execute(action);
            }
        }
        if self.should_take_intentional_loss(state) {
            if let Some(action) = Self::find_available(actions, |available| {
                available.reason == "league champion fight can be intentionally lost"
                    && matches!(
                        available.action,
                        WallAction::LeagueFight {
                            intent: LeagueFightIntent::IntentionallyLose
                        }
                    )
            }) {
                return PolicyDecision::Execute(action);
            }
        }
        if self.must_max_after_intentional_loss {
            if !state.is_magikarp_maxed() {
                return PolicyDecision::EndSession;
            }
            if let Some(action) = Self::find_action(actions, |action| {
                matches!(
                    action,
                    WallAction::LeagueFight {
                        intent: LeagueFightIntent::TryWin
                    }
                )
            }) {
                return PolicyDecision::Execute(action);
            }
        }
        if let Some(action) = Self::find_available(actions, |available| {
            available.reason == "league battle is strategically winnable"
                && matches!(
                    available.action,
                    WallAction::LeagueFight {
                        intent: LeagueFightIntent::TryWin
                    }
                )
        }) {
            return PolicyDecision::Execute(action);
        }
        if !self.ate_rest_after_block {
            if let Some(action) = Self::eat_all_berries_action(actions) {
                self.ate_rest_after_block = true;
                return PolicyDecision::Execute(action);
            }
        }
        PolicyDecision::EndSession
    }

    fn observe_action(&mut self, before: &GameState, action: &WallAction, after: &GameState) {
        match action {
            WallAction::Train => {
                self.berries_eaten_before_fight = 0;
                self.ate_rest_after_block = false;
            }
            WallAction::UpgradeBerry { .. } => {
                self.berry_upgrade_coins_spent = self
                    .berry_upgrade_coins_spent
                    .saturating_add(before.coins.saturating_sub(after.coins));
            }
            WallAction::UpgradeTraining { .. } => {
                self.training_upgrade_coins_spent = self
                    .training_upgrade_coins_spent
                    .saturating_add(before.coins.saturating_sub(after.coins));
            }
            WallAction::EatBerries { count, .. } => {
                self.berries_eaten_before_fight =
                    self.berries_eaten_before_fight.saturating_add(*count);
            }
            WallAction::LeagueFight {
                intent: LeagueFightIntent::IntentionallyLose,
            } => {
                self.ensure_league_slots(before.league);
                self.league_loss_done[before.league as usize] = true;
                self.must_max_after_intentional_loss = true;
                self.berries_eaten_before_fight = 0;
                self.ate_rest_after_block = false;
            }
            WallAction::LeagueFight {
                intent: LeagueFightIntent::TryWin,
            } => {
                if after.league > before.league {
                    self.must_max_after_intentional_loss = false;
                }
                self.berries_eaten_before_fight = 0;
                self.ate_rest_after_block = false;
            }
            _ => {}
        }
    }
}

fn action_coin_cost(actions: &[AvailableAction], action: &WallAction) -> Option<u64> {
    actions
        .iter()
        .find(|available| available.action == *action)
        .and_then(|available| available.cost_coins)
}

fn ratio_error(training_spent: u128, berry_spent: u128, target_training_share: u128) -> u128 {
    let total = training_spent + berry_spent;
    if total == 0 {
        return target_training_share;
    }
    let left = training_spent.saturating_mul(10_000);
    let right = total.saturating_mul(target_training_share);
    left.abs_diff(right)
}

fn target_owned(state: &GameState, target: &PurchaseTarget) -> bool {
    match target.kind {
        PurchaseKind::Support => state
            .supports
            .iter()
            .any(|support| support.id == target.id && support.owned),
        PurchaseKind::Decor => state
            .decors
            .iter()
            .any(|decor| decor.id == target.id && decor.owned),
    }
}
