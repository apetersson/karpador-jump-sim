use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

use crate::data::{DecorEffect, GameData, SupportSkill};
use crate::model::{
    BerryState, DecorState, DiamondLedgerEntry, DiamondSource, GameState, Kp, PendingDiamondReward,
    Provenance, PurchaseKind, PurchasePlan, PurchaseTarget, SupportState, WallClock,
};
use crate::rules::Rules;

#[derive(Clone, Debug)]
pub struct WallSimConfig {
    pub max_actions: u32,
    pub max_wall_days: u32,
    pub max_sessions_per_day: u8,
    pub target_league: u32,
}

impl Default for WallSimConfig {
    fn default() -> Self {
        Self {
            max_actions: 100_000,
            max_wall_days: 240,
            max_sessions_per_day: 10,
            target_league: 10,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub enum WallRunOutcome {
    TargetReached,
    ActionLimit,
    DayLimit,
}

#[derive(Clone, Debug, Serialize)]
pub struct WallSimResult {
    pub seed: u64,
    pub plan: String,
    pub dataset: &'static str,
    pub outcome: WallRunOutcome,
    pub wall_days: f64,
    pub sessions: u32,
    pub purchases: Vec<PurchasedItem>,
    pub diamond_income_by_source: Vec<DiamondIncomeSummary>,
    pub action_log: Vec<ActionLogEntry>,
    pub final_state: GameState,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PurchasedItem {
    pub minute: u64,
    pub kind: PurchaseKind,
    pub id: String,
    pub price_diamonds: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct DiamondIncomeSummary {
    pub source: DiamondSource,
    pub amount: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct ActionLogEntry {
    pub minute: u64,
    pub day: u32,
    pub time: String,
    pub event: String,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SessionAction {
    CollectGold,
    ClaimAchievement,
    UpgradeSupport(usize),
    UseSupport(usize),
    BuyNextPlanItem,
    UpgradeBerry(usize),
    Train,
    EatBerries { index: usize, count: u32 },
    LeagueFight { intentional_loss: bool },
}

#[derive(Clone, Debug, Default)]
struct SessionContext {
    berries_eaten_before_fight: u32,
    ate_rest_after_block: bool,
}

pub struct WallTimeSimulator<R> {
    rules: R,
    data: GameData,
    config: WallSimConfig,
}

impl<R: Rules> WallTimeSimulator<R> {
    pub fn new(rules: R, data: GameData, config: WallSimConfig) -> Self {
        Self {
            rules,
            data,
            config,
        }
    }

    pub fn run(&self, seed: u64, plan: PurchasePlan) -> WallSimResult {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut state = self.initial_wall_state(&mut rng);
        let mut purchases = Vec::new();
        let mut action_log = Vec::new();
        let mut plan_cursor = 0_usize;
        let mut session_count = 0_u32;
        log_event(
            &mut action_log,
            &state,
            "start",
            format!(
                "rank {}, gen {}, diamonds {}, max_level {}",
                state.player_rank, state.generation, state.diamonds, state.magikarp.max_level
            ),
        );

        while state.action_count < self.config.max_actions
            && state.clock.day < self.config.max_wall_days
            && state.league < self.config.target_league
        {
            if !self.start_next_session(&mut state) {
                break;
            }
            session_count += 1;
            log_event(
                &mut action_log,
                &state,
                "session_start",
                format!(
                    "session {} of day {}, league {}-{}, rank {}, diamonds {}",
                    state.clock.sessions_today,
                    state.clock.day + 1,
                    state.league + 1,
                    state.competition + 1,
                    state.player_rank,
                    state.diamonds
                ),
            );
            let mut ctx = SessionContext::default();
            let mut actions_in_session = 0_u32;

            loop {
                self.refresh_timers(&mut state);
                let Some(action) = self.decide_session_action(&state, &plan, plan_cursor, &ctx)
                else {
                    break;
                };
                self.apply_session_action(
                    &mut state,
                    action,
                    &mut rng,
                    &mut ctx,
                    &plan,
                    &mut plan_cursor,
                    &mut purchases,
                    &mut action_log,
                );
                state.action_count += 1;
                actions_in_session += 1;
                state.magikarp.level = self
                    .rules
                    .level_for_kp(state.magikarp.kp, state.magikarp.max_level);

                if state.league >= self.config.target_league || actions_in_session > 1_000 {
                    break;
                }
            }
            log_event(
                &mut action_log,
                &state,
                "session_end",
                format!(
                    "{} actions, level {}/{}, kp {}, diamonds {}",
                    actions_in_session,
                    state.magikarp.level,
                    state.magikarp.max_level,
                    state.magikarp.kp,
                    state.diamonds
                ),
            );
        }

        self.claim_all_pending_diamond_rewards(&mut state);
        log_event(
            &mut action_log,
            &state,
            "finish",
            format!(
                "outcome candidate: league {}, gen {}, rank {}, diamonds {}, purchases {}",
                state.league,
                state.generation,
                state.player_rank,
                state.diamonds,
                purchases.len()
            ),
        );

        let outcome = if state.league >= self.config.target_league {
            WallRunOutcome::TargetReached
        } else if state.clock.day >= self.config.max_wall_days {
            WallRunOutcome::DayLimit
        } else {
            WallRunOutcome::ActionLimit
        };
        let wall_days = state.now() as f64 / WallClock::MINUTES_PER_DAY as f64;
        let mut warnings = self.data.audit().warnings;
        if purchases.is_empty() {
            warnings.push("no diamond purchases were affordable/reached for this plan".to_string());
        }
        let diamond_income_by_source = summarize_diamond_income(&state.diamond_ledger);

        WallSimResult {
            seed,
            plan: plan.name,
            dataset: self.data.name,
            outcome,
            wall_days,
            sessions: session_count,
            purchases,
            diamond_income_by_source,
            action_log,
            final_state: state,
            warnings,
        }
    }

    fn initial_wall_state(&self, rng: &mut impl Rng) -> GameState {
        let mut state = self.rules.initial_state();
        state.clock = WallClock::default();
        state.elapsed_minutes = 0;
        state.diamonds = 0;
        state.diamond_ledger.clear();
        self.credit_diamonds(
            &mut state,
            self.data.economy.initial_diamonds.value,
            DiamondSource::Tutorial,
            "tutorial/intro/start wallet aggregate".to_string(),
            self.data.economy.initial_diamonds.provenance,
        );
        state.pending_achievement_claims = 0;
        state.pending_diamond_rewards.clear();
        state.diamond_achievement_keys.clear();
        state.league_wins_total = 0;
        state.support_skill_uses = 0;
        state.random_events_seen = 0;
        state.discovered_patterns = 0;
        state.login_days_claimed = 0;
        state.league_loss_done = vec![false; self.data.leagues.len()];
        state.home_treasure_ready_at = state.now();
        state.next_food_spawn_at =
            state.now() + self.data.economy.food_respawn_minutes.value as u64;
        state.next_stamina_at =
            state.now() + self.data.economy.stamina_respawn_minutes.value as u64;
        state.magikarp = self.rules.new_magikarp(&state, rng);
        self.discover_pattern(&mut state);
        self.check_fished_achievements(&mut state);
        state.berries = self
            .data
            .berries
            .iter()
            .map(|berry| BerryState {
                id: berry.id,
                name: berry.name,
                pair_group: berry.pair_group,
                level: 1,
                available: 0,
                max_available: state.max_food,
            })
            .collect();
        self.seed_initial_food(&mut state);
        state.supports = self
            .data
            .supports
            .iter()
            .map(|support| SupportState {
                id: support.id,
                name: support.name,
                owned: false,
                level: 1,
                ready_at: state.now(),
            })
            .collect();
        state.decors = self
            .data
            .decors
            .iter()
            .map(|decor| DecorState {
                id: decor.id,
                name: decor.name,
                owned: false,
            })
            .collect();
        state
    }

    fn start_next_session(&self, state: &mut GameState) -> bool {
        if self.config.max_sessions_per_day == 0 {
            return false;
        }
        let mut now = state.now();
        let day_start = state.clock.day as u64 * WallClock::MINUTES_PER_DAY as u64
            + WallClock::SESSION_START as u64;
        let day_end = state.clock.day as u64 * WallClock::MINUTES_PER_DAY as u64
            + WallClock::SESSION_END as u64;

        if now < day_start {
            now = day_start;
        }
        if now > day_end || state.clock.sessions_today >= self.config.max_sessions_per_day {
            let next_day_start = (state.clock.day as u64 + 1) * WallClock::MINUTES_PER_DAY as u64
                + WallClock::SESSION_START as u64;
            state.clock.sessions_today = 0;
            state.set_now(next_day_start);
            now = state.now();
        }

        let next_slot = self.next_session_slot_time(state);
        let next_useful = self.next_useful_time(state).max(now).max(next_slot);
        let current_day_end = state.clock.day as u64 * WallClock::MINUTES_PER_DAY as u64
            + WallClock::SESSION_END as u64;
        if next_useful > current_day_end {
            let next_day_start = (state.clock.day as u64 + 1) * WallClock::MINUTES_PER_DAY as u64
                + WallClock::SESSION_START as u64;
            state.clock.sessions_today = 0;
            state.set_now(next_day_start);
        } else {
            state.set_now(next_useful);
        }
        self.refresh_timers(state);

        if state.clock.day >= self.config.max_wall_days {
            return false;
        }
        state.clock.sessions_today += 1;
        if state.clock.sessions_today == 1 {
            self.record_daily_login(state);
        }
        true
    }

    fn next_session_slot_time(&self, state: &GameState) -> u64 {
        let day_start = state.clock.day as u64 * WallClock::MINUTES_PER_DAY as u64
            + WallClock::SESSION_START as u64;
        let slots = self.config.max_sessions_per_day.max(1) as u64;
        if slots == 1 {
            return day_start;
        }
        let slot_index = state
            .clock
            .sessions_today
            .min(self.config.max_sessions_per_day) as u64;
        let span = (WallClock::SESSION_END - WallClock::SESSION_START) as u64;
        day_start + span * slot_index / (slots - 1)
    }

    fn next_useful_time(&self, state: &GameState) -> u64 {
        let now = state.now();
        if self.has_immediate_action(state) {
            return now;
        }

        let mut next = u64::MAX;
        if self.total_food_available(state) < state.max_food {
            next = next.min(state.next_food_spawn_at);
        }
        if state.stamina < state.max_stamina {
            next = next.min(state.next_stamina_at);
        }
        next = next.min(state.home_treasure_ready_at);
        for (i, support) in state.supports.iter().enumerate() {
            if support.owned && self.support_is_unlocked(state, i) {
                next = next.min(support.ready_at);
            }
        }
        if next == u64::MAX { now + 60 } else { next }
    }

    fn has_immediate_action(&self, state: &GameState) -> bool {
        state.home_treasure_ready_at <= state.now()
            || state.pending_achievement_claims > 0
            || state.stamina > 0
            || state
                .berries
                .iter()
                .enumerate()
                .any(|(index, berry)| berry.available > 0 && self.berry_is_unlocked(state, index))
            || self.expected_jump_clears_current_opponent(state)
            || (state.must_max_after_intentional_loss && state.is_magikarp_maxed())
            || state.supports.iter().enumerate().any(|(i, support)| {
                support.owned
                    && support.ready_at <= state.now()
                    && self.support_is_unlocked(state, i)
            })
    }

    fn refresh_timers(&self, state: &mut GameState) {
        let now = state.now();
        let food_respawn = self.data.economy.food_respawn_minutes.value as u64;
        while state.next_food_spawn_at <= now && self.total_food_available(state) < state.max_food {
            for _ in 0..self.food_spawns_per_minute_tick() {
                if self.total_food_available(state) >= state.max_food {
                    break;
                }
                if let Some(index) = self.next_food_spawn_index(state) {
                    state.berries[index].available += 1;
                }
            }
            state.next_food_spawn_at += food_respawn.max(1);
        }

        let stamina_respawn = self.data.economy.stamina_respawn_minutes.value as u64;
        while state.next_stamina_at <= now && state.stamina < state.max_stamina {
            state.stamina += 1;
            state.next_stamina_at += stamina_respawn.max(1);
        }
    }

    fn food_spawns_per_minute_tick(&self) -> u32 {
        let seconds = self.data.economy.food_respawn_seconds.value.max(1);
        (60 / seconds).max(1)
    }

    fn decide_session_action(
        &self,
        state: &GameState,
        plan: &PurchasePlan,
        plan_cursor: usize,
        ctx: &SessionContext,
    ) -> Option<SessionAction> {
        if state.home_treasure_ready_at <= state.now() {
            return Some(SessionAction::CollectGold);
        }
        if state.pending_achievement_claims > 0 {
            return Some(SessionAction::ClaimAchievement);
        }
        if self.active_kp_gain_buff_permyriad(state) > 10_000 {
            if state.stamina > 0 {
                return Some(SessionAction::Train);
            }
            if let Some((index, count)) = self.next_berries_to_eat(state, u32::MAX) {
                return Some(SessionAction::EatBerries { index, count });
            }
        }
        if let Some(index) = self.next_support_upgrade(state) {
            return Some(SessionAction::UpgradeSupport(index));
        }
        if let Some(index) = state
            .supports
            .iter()
            .enumerate()
            .find(|(i, support)| {
                support.owned
                    && support.ready_at <= state.now()
                    && self.support_is_unlocked(state, *i)
            })
            .map(|(i, _)| i)
        {
            return Some(SessionAction::UseSupport(index));
        }
        if self.can_buy_next_plan_item(state, plan, plan_cursor) {
            return Some(SessionAction::BuyNextPlanItem);
        }
        if let Some(index) = self.next_equal_berry_upgrade(state) {
            return Some(SessionAction::UpgradeBerry(index));
        }
        if state.stamina > 0 {
            return Some(SessionAction::Train);
        }
        if ctx.berries_eaten_before_fight < 3 {
            if let Some((index, count)) =
                self.next_berries_to_eat(state, 3 - ctx.berries_eaten_before_fight)
            {
                return Some(SessionAction::EatBerries { index, count });
            }
        }

        if self.should_take_intentional_loss(state) {
            return Some(SessionAction::LeagueFight {
                intentional_loss: true,
            });
        }
        if self.expected_jump_clears_current_opponent(state)
            || (state.must_max_after_intentional_loss && state.is_magikarp_maxed())
        {
            return Some(SessionAction::LeagueFight {
                intentional_loss: false,
            });
        }
        if !ctx.ate_rest_after_block {
            if let Some((index, count)) = self.next_berries_to_eat(state, u32::MAX) {
                return Some(SessionAction::EatBerries { index, count });
            }
        }
        None
    }

    fn apply_session_action(
        &self,
        state: &mut GameState,
        action: SessionAction,
        rng: &mut impl Rng,
        ctx: &mut SessionContext,
        plan: &PurchasePlan,
        plan_cursor: &mut usize,
        purchases: &mut Vec<PurchasedItem>,
        action_log: &mut Vec<ActionLogEntry>,
    ) {
        match action {
            SessionAction::CollectGold => {
                let before_coins = state.coins;
                let before_diamonds = state.diamonds;
                let coin_bonus = self.coin_bonus_permyriad(state);
                let coins = self.data.economy.home_treasure_base_coins.value
                    * (1 + state.player_rank as u64)
                    * coin_bonus as u64
                    / 10_000;
                state.coins = state.coins.saturating_add(coins);
                self.maybe_grant_sunken_treasure_diamonds(state, rng, "home sunken treasure");
                state.home_treasure_ready_at =
                    state.now() + self.data.economy.home_treasure_cooldown_minutes.value as u64;
                log_event(
                    action_log,
                    state,
                    "collect_gold",
                    format!(
                        "+{} coins, +{} diamonds",
                        state.coins.saturating_sub(before_coins),
                        state.diamonds.saturating_sub(before_diamonds)
                    ),
                );
                self.advance_minutes(state, 1);
            }
            SessionAction::ClaimAchievement => {
                let before = state.diamonds;
                let count = state.pending_diamond_rewards.len();
                self.claim_all_pending_diamond_rewards(state);
                log_event(
                    action_log,
                    state,
                    "claim_achievements",
                    format!(
                        "{} rewards, +{} diamonds",
                        count,
                        state.diamonds.saturating_sub(before)
                    ),
                );
                self.advance_minutes(state, 1);
            }
            SessionAction::UpgradeSupport(index) => {
                if let Some(cost) = self.next_support_upgrade_cost(state, index) {
                    if state.candy >= cost {
                        state.candy -= cost;
                        state.supports[index].level += 1;
                        log_event(
                            action_log,
                            state,
                            "upgrade_support",
                            format!(
                                "{} to level {} for {} candy",
                                state.supports[index].name, state.supports[index].level, cost
                            ),
                        );
                    }
                }
                self.advance_minutes(state, 1);
            }
            SessionAction::UseSupport(index) => {
                let before_kp = state.magikarp.kp;
                let before_coins = state.coins;
                let before_diamonds = state.diamonds;
                let name = state.supports[index].name;
                self.use_support(state, index, rng);
                log_event(
                    action_log,
                    state,
                    "use_support",
                    format!(
                        "{}: +{} kp, +{} coins, +{} diamonds",
                        name,
                        state.magikarp.kp.saturating_sub(before_kp),
                        state.coins.saturating_sub(before_coins),
                        state.diamonds.saturating_sub(before_diamonds)
                    ),
                );
                self.advance_minutes(state, 1);
            }
            SessionAction::BuyNextPlanItem => {
                if let Some(target) = plan.targets.get(*plan_cursor) {
                    let price = self.data.purchase_price(target);
                    if state.diamonds >= price {
                        state.diamonds -= price;
                        self.mark_purchase_owned(state, target);
                        purchases.push(PurchasedItem {
                            minute: state.now(),
                            kind: target.kind.clone(),
                            id: target.id.clone(),
                            price_diamonds: price,
                        });
                        log_event(
                            action_log,
                            state,
                            "buy",
                            format!("{:?} {} for {} diamonds", target.kind, target.id, price),
                        );
                    }
                    *plan_cursor += 1;
                }
                self.advance_minutes(state, 1);
            }
            SessionAction::UpgradeBerry(index) => {
                let cost = self.berry_upgrade_cost(state, index);
                if state.coins >= cost {
                    state.coins -= cost;
                    state.berries[index].level += 1;
                    log_event(
                        action_log,
                        state,
                        "upgrade_berry",
                        format!(
                            "{} to level {} for {} coins",
                            state.berries[index].name, state.berries[index].level, cost
                        ),
                    );
                }
                self.advance_minutes(state, 1);
            }
            SessionAction::Train => {
                let before_kp = state.magikarp.kp;
                state.stamina = state.stamina.saturating_sub(1);
                if state.stamina + 1 == state.max_stamina {
                    state.next_stamina_at =
                        state.now() + self.data.economy.stamina_respawn_minutes.value as u64;
                }
                let result = self.rules.training_result(rng);
                let training_bonus = self.training_bonus_permyriad(state);
                let (training_name, base_gain) = self.training_base_gain(state, rng);
                let gained = self.training_result_gain(base_gain, result)
                    * training_bonus as Kp
                    * self.active_kp_gain_buff_permyriad(state) as Kp
                    / 10_000
                    / 10_000;
                state.magikarp.kp = state.magikarp.kp.saturating_add(gained);
                state.magikarp.trainings_done += 1;
                self.maybe_trigger_random_event(state, rng, "training");
                self.check_training_achievements(state);
                log_event(
                    action_log,
                    state,
                    "train",
                    format!(
                        "{} {:?}, +{} kp, stamina {}/{}",
                        training_name,
                        result,
                        state.magikarp.kp.saturating_sub(before_kp),
                        state.stamina,
                        state.max_stamina
                    ),
                );
                ctx.berries_eaten_before_fight = 0;
                ctx.ate_rest_after_block = false;
                self.advance_minutes(state, 1);
            }
            SessionAction::EatBerries { index, count } => {
                let eaten = state.berries[index].available.min(count);
                if eaten > 0 {
                    let before_kp = state.magikarp.kp;
                    state.berries[index].available -= eaten;
                    let kp = self.berry_kp(state, index) * eaten as Kp;
                    state.magikarp.kp = state.magikarp.kp.saturating_add(kp);
                    state.magikarp.foods_eaten += eaten;
                    self.check_feed_achievements(state);
                    ctx.berries_eaten_before_fight += eaten;
                    if count == u32::MAX || ctx.berries_eaten_before_fight >= 3 {
                        ctx.ate_rest_after_block = true;
                    }
                    log_event(
                        action_log,
                        state,
                        "eat_berries",
                        format!(
                            "{} x{}, +{} kp",
                            state.berries[index].name,
                            eaten,
                            state.magikarp.kp.saturating_sub(before_kp)
                        ),
                    );
                }
                self.advance_minutes(state, 1);
            }
            SessionAction::LeagueFight { intentional_loss } => {
                let before_league = state.league;
                let before_competition = state.competition;
                let before_rank = state.player_rank;
                let before_diamonds = state.diamonds;
                self.league_fight(state, intentional_loss, rng);
                log_event(
                    action_log,
                    state,
                    if intentional_loss {
                        "league_loss"
                    } else {
                        "league_fight"
                    },
                    format!(
                        "{}-{} -> {}-{}, rank {}->{}, diamonds +{}",
                        before_league + 1,
                        before_competition + 1,
                        state.league + 1,
                        state.competition + 1,
                        before_rank,
                        state.player_rank,
                        state.diamonds.saturating_sub(before_diamonds)
                    ),
                );
                ctx.berries_eaten_before_fight = 0;
                ctx.ate_rest_after_block = false;
                self.advance_minutes(state, 1);
            }
        }
    }

    fn advance_minutes(&self, state: &mut GameState, minutes: u64) {
        state.set_now(state.now() + minutes);
        self.refresh_timers(state);
    }

    fn use_support(&self, state: &mut GameState, index: usize, rng: &mut impl Rng) {
        let support_data = &self.data.supports[index];
        state.support_skill_uses += 1;
        self.check_support_skill_achievements(state);
        let support_param = self.support_level_param(state, index);
        match support_data.skill {
            SupportSkill::KpFlat { base } => {
                let gain = support_param.unwrap_or(base)
                    * (1 + state.player_rank as Kp)
                    * self.kp_bonus_permyriad(state) as Kp
                    * self.active_kp_gain_buff_permyriad(state) as Kp
                    / 10_000
                    / 10_000
                    / 4;
                state.magikarp.kp = state.magikarp.kp.saturating_add(gain);
            }
            SupportSkill::Coins { base } => {
                let gain = support_param.unwrap_or(base as Kp) as u64
                    * (1 + state.player_rank as u64)
                    * self.coin_bonus_permyriad(state) as u64
                    / 10_000;
                state.coins = state.coins.saturating_add(gain);
            }
            SupportSkill::Stamina { amount } => {
                let amount = support_param.unwrap_or(amount as Kp) as u32;
                state.stamina = (state.stamina + amount).min(state.max_stamina);
            }
            SupportSkill::Diamonds { amount } => {
                self.credit_diamonds(
                    state,
                    amount,
                    DiamondSource::SupportSkill,
                    format!("{} skill", support_data.name),
                    Provenance::Wiki,
                );
            }
            SupportSkill::Food { amount } => {
                let mut remaining = support_param.unwrap_or(amount as Kp) as u32;
                while remaining > 0 && self.total_food_available(state) < state.max_food {
                    let Some(spawn_index) = self.next_food_spawn_index(state) else {
                        break;
                    };
                    state.berries[spawn_index].available += 1;
                    remaining -= 1;
                }
            }
            SupportSkill::Item { base_coin_value } => {
                if rng.random_range(0..10_000)
                    < self
                        .data
                        .economy
                        .sunken_treasure_diamond_chance_permyriad
                        .value
                {
                    self.credit_diamonds(
                        state,
                        self.data.economy.sunken_treasure_diamonds.value,
                        DiamondSource::SupportSkill,
                        format!("{} item drop diamonds", support_data.name),
                        self.data.economy.sunken_treasure_diamonds.provenance,
                    );
                } else {
                    let gain = base_coin_value
                        * (1 + state.player_rank as u64)
                        * self.coin_bonus_permyriad(state) as u64
                        / 10_000;
                    state.coins = state.coins.saturating_add(gain);
                }
            }
            SupportSkill::RecoverSkills => {
                let now = state.now();
                for (other_index, support) in state.supports.iter_mut().enumerate() {
                    if other_index != index && support.owned {
                        support.ready_at = now;
                    }
                }
            }
            SupportSkill::LeaguePoint => {
                state.pending_achievement_claims += 1;
            }
            SupportSkill::TrainingGreat => {
                let (_, base_gain) = self.training_base_gain(state, rng);
                let gain = self
                    .training_result_gain(base_gain, crate::rules::TrainingResult::Great)
                    * self.active_kp_gain_buff_permyriad(state) as Kp
                    / 10_000;
                state.magikarp.kp = state.magikarp.kp.saturating_add(gain);
            }
            SupportSkill::KpBoost {
                multiplier_permyriad,
            } => {
                state.kp_gain_buff_until = state.now() + 1;
                state.kp_gain_buff_permyriad = multiplier_permyriad;
            }
        }
        state.supports[index].ready_at = state.now() + support_data.cooldown_minutes.value as u64;
    }

    fn league_fight(&self, state: &mut GameState, intentional_loss: bool, rng: &mut impl Rng) {
        let Some(competition) = self.current_competition(state).cloned() else {
            return;
        };
        if intentional_loss {
            state.coins = state.coins.saturating_add(
                competition.loss_reward_coins.value * self.coin_bonus_permyriad(state) as u64
                    / 10_000,
            );
            if let Some(done) = state.league_loss_done.get_mut(state.league as usize) {
                *done = true;
            }
            state.must_max_after_intentional_loss = true;
            self.maybe_trigger_random_event(state, rng, "intentional league loss");
            return;
        }

        let cheer_permyriad = match rng.random_range(0..100) {
            0..=74 => 10_000_u64,
            75..=94 => 11_500_u64,
            _ => 13_000_u64,
        };
        let own_jump = state.magikarp.kp * cheer_permyriad as Kp / 10_000;
        if own_jump >= competition.opponent_jump_cm.value as Kp
            || (state.must_max_after_intentional_loss && state.is_magikarp_maxed())
        {
            let won_league = state.league;
            let won_competition = state.competition;
            state.magikarp.wins += 1;
            state.league_wins_total += 1;
            state.coins = state.coins.saturating_add(
                competition.win_reward_coins.value * self.coin_bonus_permyriad(state) as u64
                    / 10_000,
            );
            self.credit_diamonds(
                state,
                competition.reward_diamonds.value,
                DiamondSource::LeagueBattleReward,
                format!("competition {}", competition.id),
                competition.reward_diamonds.provenance,
            );
            state.candy = state.candy.saturating_add(competition.reward_candy.value);
            let exp = competition.breeder_exp_win.value
                * self.trainer_exp_bonus_permyriad(state) as u128
                / 10_000;
            self.increase_trainer_exp(state, exp, format!("competition {}", competition.id));
            self.grant_league_battle_diamonds_if_due(state);
            self.grant_battle_rewards(state, won_league, won_competition);
            state.competition += 1;
            if state.competition
                >= self.data.leagues[state.league as usize].competitions.len() as u32
            {
                state.league += 1;
                state.competition = 0;
                state.max_stamina = state.max_stamina.max(3 + state.player_rank / 2);
                state.stamina = state.max_stamina;
                state.must_max_after_intentional_loss = false;
                self.grant_league_rewards(state);
            }
            self.maybe_trigger_random_event(state, rng, "league win");
        } else if state.is_magikarp_maxed() {
            let xp = self.rules.retirement_rank_xp(state);
            self.increase_trainer_exp(state, xp as u128, "retirement xp".to_string());
            state.retirements += 1;
            state.generation += 1;
            state.magikarp = self.rules.new_magikarp(state, rng);
            self.discover_pattern(state);
            self.check_fished_achievements(state);
            self.check_retirement_achievements(state);
            for berry in &mut state.berries {
                berry.available = berry.max_available;
            }
            state.stamina = state.max_stamina;
        }
    }

    fn current_competition(&self, state: &GameState) -> Option<&crate::data::CompetitionData> {
        self.data
            .leagues
            .get(state.league as usize)?
            .competitions
            .get(state.competition as usize)
    }

    fn should_take_intentional_loss(&self, state: &GameState) -> bool {
        let Some(league) = self.data.leagues.get(state.league as usize) else {
            return false;
        };
        let loss_done = state
            .league_loss_done
            .get(state.league as usize)
            .copied()
            .unwrap_or(false);
        !loss_done
            && state.competition + 1 == league.competitions.len() as u32
            && !state.must_max_after_intentional_loss
    }

    fn expected_jump_clears_current_opponent(&self, state: &GameState) -> bool {
        self.current_competition(state)
            .map(|competition| state.magikarp.kp >= competition.opponent_jump_cm.value as Kp)
            .unwrap_or(false)
    }

    fn support_is_unlocked(&self, state: &GameState, index: usize) -> bool {
        self.data
            .supports
            .get(index)
            .map(|support| state.league >= support.unlock_league)
            .unwrap_or(false)
    }

    fn credit_diamonds(
        &self,
        state: &mut GameState,
        amount: u32,
        source: DiamondSource,
        detail: String,
        provenance: Provenance,
    ) {
        if amount == 0 {
            return;
        }
        state.diamonds = state.diamonds.saturating_add(amount);
        state.diamond_ledger.push(DiamondLedgerEntry {
            minute: state.now(),
            amount,
            source,
            detail,
            provenance,
        });
    }

    fn claim_all_pending_diamond_rewards(&self, state: &mut GameState) {
        let rewards = std::mem::take(&mut state.pending_diamond_rewards);
        for reward in rewards {
            self.credit_diamonds(
                state,
                reward.amount,
                reward.source,
                reward.detail,
                reward.provenance,
            );
        }
        state.pending_achievement_claims = 0;
    }

    fn queue_achievement_diamonds_once(
        &self,
        state: &mut GameState,
        key: String,
        amount: u32,
        detail: String,
    ) {
        if state.diamond_achievement_keys.contains(&key) {
            return;
        }
        state.diamond_achievement_keys.push(key);
        state.pending_diamond_rewards.push(PendingDiamondReward {
            amount,
            source: DiamondSource::Achievement,
            detail,
            provenance: self.data.economy.achievement_diamonds_small.provenance,
        });
        state.pending_achievement_claims = state.pending_diamond_rewards.len() as u32;
    }

    fn record_daily_login(&self, state: &mut GameState) {
        let login_count = state.clock.day + 1;
        if state.login_days_claimed >= login_count {
            return;
        }
        state.login_days_claimed = login_count;
        for (milestone, amount) in [(3, 25), (7, 25), (14, 25), (30, 25), (70, 50)] {
            if login_count >= milestone {
                self.queue_achievement_diamonds_once(
                    state,
                    format!("login:{milestone}"),
                    amount,
                    format!("number of logins {milestone}"),
                );
            }
        }
    }

    fn increase_trainer_exp(&self, state: &mut GameState, exp: u128, reason: String) {
        if exp == 0 {
            return;
        }
        let before_rank = state.player_rank;
        state.trainer_exp = state.trainer_exp.saturating_add(exp);
        state.player_rank = self.rules.player_rank_for_exp(state.trainer_exp);
        if state.player_rank > before_rank {
            for rank in before_rank + 1..=state.player_rank {
                self.credit_diamonds(
                    state,
                    self.data.economy.trainer_rank_up_diamonds.value,
                    DiamondSource::TrainerRank,
                    format!("rank {rank} from {reason}"),
                    self.data.economy.trainer_rank_up_diamonds.provenance,
                );
            }
        }
    }

    fn grant_league_battle_diamonds_if_due(&self, state: &mut GameState) {
        if state.league_wins_total == 0 || state.league_wins_total % 5 != 0 {
            return;
        }
        self.credit_diamonds(
            state,
            self.data.economy.league_battle_milestone_diamonds.value,
            DiamondSource::LeagueBattleReward,
            format!("league battle reward {}", state.league_wins_total),
            self.data
                .economy
                .league_battle_milestone_diamonds
                .provenance,
        );
    }

    fn discover_pattern(&self, state: &mut GameState) {
        state.discovered_patterns += 1;
        self.credit_diamonds(
            state,
            self.data.economy.pattern_discovery_diamonds.value,
            DiamondSource::PatternDiscovery,
            format!("new pattern #{}", state.discovered_patterns),
            self.data.economy.pattern_discovery_diamonds.provenance,
        );
        for milestone in [3, 10, 20, 30, 40] {
            if state.discovered_patterns >= milestone {
                self.queue_achievement_diamonds_once(
                    state,
                    format!("patterns:{milestone}"),
                    self.data.economy.achievement_diamonds_minor.value,
                    format!("patterns collected {milestone}"),
                );
            }
        }
    }

    fn maybe_trigger_random_event(&self, state: &mut GameState, rng: &mut impl Rng, trigger: &str) {
        let training_event_bonus = self.training_event_bonus_permyriad(state);
        let chance = self
            .data
            .economy
            .random_event_diamond_chance_permyriad
            .value
            .saturating_mul(training_event_bonus)
            / 10_000;
        if rng.random_range(0..10_000) >= chance {
            return;
        }
        state.random_events_seen += 1;
        self.credit_diamonds(
            state,
            self.data.economy.random_event_diamonds.value,
            DiamondSource::RandomEncounter,
            format!("diamond random encounter after {trigger}"),
            self.data.economy.random_event_diamonds.provenance,
        );
        for milestone in [3, 10, 20, 34] {
            if state.random_events_seen >= milestone {
                self.queue_achievement_diamonds_once(
                    state,
                    format!("events:{milestone}"),
                    self.data.economy.achievement_diamonds_minor.value,
                    format!("events triggered {milestone}"),
                );
            }
        }
    }

    fn maybe_grant_sunken_treasure_diamonds(
        &self,
        state: &mut GameState,
        rng: &mut impl Rng,
        detail: &str,
    ) {
        if rng.random_range(0..10_000)
            >= self
                .data
                .economy
                .sunken_treasure_diamond_chance_permyriad
                .value
        {
            return;
        }
        self.credit_diamonds(
            state,
            self.data.economy.sunken_treasure_diamonds.value,
            DiamondSource::SunkenTreasure,
            detail.to_string(),
            self.data.economy.sunken_treasure_diamonds.provenance,
        );
    }

    fn check_feed_achievements(&self, state: &mut GameState) {
        let _ = state;
        // Times Fed achievements award coins, not diamonds.
    }

    fn check_training_achievements(&self, state: &mut GameState) {
        let _ = state;
        // Times Trained achievements award coins, not diamonds.
    }

    fn check_support_skill_achievements(&self, state: &mut GameState) {
        for milestone in [3, 20, 100, 300, 800] {
            if state.support_skill_uses >= milestone {
                self.queue_achievement_diamonds_once(
                    state,
                    format!("support-skills:{milestone}"),
                    self.data.economy.achievement_diamonds_minor.value,
                    format!("number of skill uses {milestone}"),
                );
            }
        }
    }

    fn check_fished_achievements(&self, state: &mut GameState) {
        for milestone in [3, 10, 40, 90, 190] {
            if state.generation >= milestone {
                self.queue_achievement_diamonds_once(
                    state,
                    format!("fished:{milestone}"),
                    self.data.economy.achievement_diamonds_minor.value,
                    format!("magikarp fished {milestone}"),
                );
            }
        }
    }

    fn check_retirement_achievements(&self, state: &mut GameState) {
        for milestone in [1, 4, 12, 25, 70] {
            if state.retirements >= milestone {
                self.queue_achievement_diamonds_once(
                    state,
                    format!("retirements:{milestone}"),
                    self.data.economy.achievement_diamonds_minor.value,
                    format!("magikarp forced to retire {milestone}"),
                );
            }
        }
    }

    fn next_support_upgrade(&self, state: &GameState) -> Option<usize> {
        state
            .supports
            .iter()
            .enumerate()
            .filter(|(index, support)| {
                support.owned
                    && self.support_is_unlocked(state, *index)
                    && self
                        .next_support_upgrade_cost(state, *index)
                        .is_some_and(|cost| state.candy >= cost)
            })
            .min_by_key(|(index, _)| {
                self.next_support_upgrade_cost(state, *index)
                    .unwrap_or(u32::MAX)
            })
            .map(|(index, _)| index)
    }

    fn next_support_upgrade_cost(&self, state: &GameState, index: usize) -> Option<u32> {
        let current_level = state.supports.get(index)?.level;
        self.data
            .supports
            .get(index)?
            .upgrade_candy_costs
            .get(current_level.saturating_sub(1) as usize)
            .map(|cost| cost.value)
    }

    fn can_buy_next_plan_item(
        &self,
        state: &GameState,
        plan: &PurchasePlan,
        plan_cursor: usize,
    ) -> bool {
        let Some(target) = plan.targets.get(plan_cursor) else {
            return false;
        };
        if self.is_owned(state, target) {
            return true;
        }
        state.diamonds >= self.data.purchase_price(target)
            && self.is_unlocked_for_purchase(state, target)
    }

    fn is_unlocked_for_purchase(&self, state: &GameState, target: &PurchaseTarget) -> bool {
        match target.kind {
            PurchaseKind::Support => self
                .data
                .supports
                .iter()
                .find(|support| support.id == target.id)
                .map(|support| state.league >= support.unlock_league)
                .unwrap_or(false),
            PurchaseKind::Decor => true,
        }
    }

    fn is_owned(&self, state: &GameState, target: &PurchaseTarget) -> bool {
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

    fn mark_purchase_owned(&self, state: &mut GameState, target: &PurchaseTarget) {
        let now = state.now();
        match target.kind {
            PurchaseKind::Support => {
                if let Some(support) = state
                    .supports
                    .iter_mut()
                    .find(|support| support.id == target.id)
                {
                    support.owned = true;
                    support.ready_at = now;
                }
            }
            PurchaseKind::Decor => {
                if let Some(decor) = state.decors.iter_mut().find(|decor| decor.id == target.id) {
                    decor.owned = true;
                    self.apply_decor_on_acquire(state, target.id.as_str());
                }
            }
        }
    }

    fn grant_league_rewards(&self, state: &mut GameState) {
        let cleared_league = state.league;
        for (index, data) in self.data.supports.iter().enumerate() {
            if data.acquisition.league_reward_after() == Some(cleared_league) {
                state.supports[index].owned = true;
                state.supports[index].ready_at = state.now();
            }
        }
        let reward_decor_ids = self
            .data
            .decors
            .iter()
            .enumerate()
            .filter(|(_, data)| data.acquisition.league_reward_after() == Some(cleared_league))
            .map(|(index, data)| (index, data.id))
            .collect::<Vec<_>>();
        for (index, id) in reward_decor_ids {
            state.decors[index].owned = true;
            self.apply_decor_on_acquire(state, id);
        }
    }

    fn grant_battle_rewards(&self, state: &mut GameState, league: u32, competition: u32) {
        for (index, data) in self.data.supports.iter().enumerate() {
            if data.acquisition.battle_reward_at() == Some((league, competition)) {
                state.supports[index].owned = true;
                state.supports[index].ready_at = state.now();
            }
        }
        let reward_decor_ids = self
            .data
            .decors
            .iter()
            .enumerate()
            .filter(|(_, data)| data.acquisition.battle_reward_at() == Some((league, competition)))
            .map(|(index, data)| (index, data.id))
            .collect::<Vec<_>>();
        for (index, id) in reward_decor_ids {
            state.decors[index].owned = true;
            self.apply_decor_on_acquire(state, id);
        }
    }

    fn apply_decor_on_acquire(&self, state: &mut GameState, decor_id: &str) {
        let Some(data) = self.data.decors.iter().find(|decor| decor.id == decor_id) else {
            return;
        };
        if let DecorEffect::FoodCapacity(extra) = data.effect {
            for berry in &mut state.berries {
                berry.max_available = berry.max_available.saturating_add(extra);
            }
            state.max_food = state.max_food.saturating_add(extra);
        }
    }

    fn next_equal_berry_upgrade(&self, state: &GameState) -> Option<usize> {
        state
            .berries
            .iter()
            .enumerate()
            .filter(|(index, berry)| {
                berry.pair_group == "primary_equal"
                    && self.berry_is_unlocked(state, *index)
                    && berry.level < self.data.berries[*index].max_level
                    && state.coins >= self.berry_upgrade_cost(state, *index)
            })
            .min_by_key(|(index, berry)| (berry.level, self.berry_upgrade_cost(state, *index)))
            .map(|(index, _)| index)
    }

    fn berry_upgrade_cost(&self, state: &GameState, index: usize) -> u64 {
        self.data
            .food_upgrade_costs
            .get(state.berries[index].level.saturating_sub(1) as usize)
            .and_then(|row| row.get(index))
            .map(|cost| cost.value)
            .unwrap_or_else(|| {
                let data = &self.data.berries[index];
                data.upgrade_cost_base
                    .value
                    .saturating_mul(3_u64.pow(state.berries[index].level.saturating_sub(1).min(10)))
            })
    }

    fn next_berries_to_eat(&self, state: &GameState, wanted: u32) -> Option<(usize, u32)> {
        state
            .berries
            .iter()
            .enumerate()
            .filter(|(index, berry)| berry.available > 0 && self.berry_is_unlocked(state, *index))
            .max_by_key(|(index, _)| self.berry_kp(state, *index))
            .map(|(index, berry)| (index, berry.available.min(wanted)))
    }

    fn berry_kp(&self, state: &GameState, index: usize) -> Kp {
        let berry = &state.berries[index];
        if !self.berry_is_unlocked(state, index) {
            return 0;
        }
        let base = self
            .data
            .berry_jp(berry.id, berry.level)
            .unwrap_or(self.data.berries[index].base_kp.value);
        self.apply_magikarp_bonus(state, base)
            * self.kp_bonus_permyriad(state) as Kp
            * self.active_kp_gain_buff_permyriad(state) as Kp
            / 10_000
            / 10_000
    }

    fn training_base_gain(&self, state: &GameState, rng: &mut impl Rng) -> (&'static str, Kp) {
        let unlocked = self.data.unlocked_training_indices(state.player_rank);
        if unlocked.is_empty() {
            return (
                "Approx Training",
                self.rules
                    .training_kp(state, crate::rules::TrainingResult::Normal),
            );
        }
        let index = unlocked[rng.random_range(0..unlocked.len())];
        let training = &self.data.trainings[index];
        let base = self
            .data
            .training_jp(training.id, state.training_level)
            .unwrap_or(0);
        (training.name, self.apply_magikarp_bonus(state, base))
    }

    fn training_result_gain(&self, base: Kp, result: crate::rules::TrainingResult) -> Kp {
        let mult = match result {
            crate::rules::TrainingResult::Normal => 100,
            crate::rules::TrainingResult::Good => 150,
            crate::rules::TrainingResult::Great => 350,
        };
        base * mult / 100
    }

    fn apply_magikarp_bonus(&self, state: &GameState, base: Kp) -> Kp {
        base.saturating_mul(10_000 + state.magikarp.individual_bonus_permyriad as Kp) / 10_000
    }

    fn kp_bonus_permyriad(&self, state: &GameState) -> u32 {
        state
            .decors
            .iter()
            .enumerate()
            .filter(|(_, decor)| decor.owned)
            .fold(10_000, |acc, (index, _)| {
                match self.data.decors[index].effect {
                    DecorEffect::KpPermyriad(mult) => acc * mult / 10_000,
                    _ => acc,
                }
            })
    }

    fn coin_bonus_permyriad(&self, state: &GameState) -> u32 {
        state
            .decors
            .iter()
            .enumerate()
            .filter(|(_, decor)| decor.owned)
            .fold(10_000, |acc, (index, _)| {
                match self.data.decors[index].effect {
                    DecorEffect::CoinPermyriad(mult) => acc * mult / 10_000,
                    _ => acc,
                }
            })
    }

    fn training_bonus_permyriad(&self, state: &GameState) -> u32 {
        state
            .decors
            .iter()
            .enumerate()
            .filter(|(_, decor)| decor.owned)
            .fold(10_000, |acc, (index, _)| {
                match self.data.decors[index].effect {
                    DecorEffect::TrainingPermyriad(mult) => acc * mult / 10_000,
                    _ => acc,
                }
            })
    }

    fn training_event_bonus_permyriad(&self, state: &GameState) -> u32 {
        state
            .decors
            .iter()
            .enumerate()
            .filter(|(_, decor)| decor.owned)
            .fold(10_000, |acc, (index, _)| {
                match self.data.decors[index].effect {
                    DecorEffect::TrainingEventPermyriad(mult) => acc * mult / 10_000,
                    _ => acc,
                }
            })
    }

    fn active_kp_gain_buff_permyriad(&self, state: &GameState) -> u32 {
        if state.kp_gain_buff_until >= state.now() {
            state.kp_gain_buff_permyriad.max(10_000)
        } else {
            10_000
        }
    }

    fn trainer_exp_bonus_permyriad(&self, state: &GameState) -> u32 {
        state
            .decors
            .iter()
            .enumerate()
            .filter(|(_, decor)| decor.owned)
            .fold(10_000, |acc, (index, _)| {
                match self.data.decors[index].effect {
                    DecorEffect::TrainerExpPermyriad(mult) => acc * mult / 10_000,
                    _ => acc,
                }
            })
    }

    fn berry_is_unlocked(&self, state: &GameState, index: usize) -> bool {
        self.data
            .berries
            .get(index)
            .map(|berry| state.player_rank >= berry.unlock_rank)
            .unwrap_or(false)
    }

    fn total_food_available(&self, state: &GameState) -> u32 {
        state
            .berries
            .iter()
            .enumerate()
            .filter(|(index, _)| self.berry_is_unlocked(state, *index))
            .map(|(_, berry)| berry.available)
            .sum()
    }

    fn next_food_spawn_index(&self, state: &GameState) -> Option<usize> {
        state
            .berries
            .iter()
            .enumerate()
            .filter(|(index, _)| self.berry_is_unlocked(state, *index))
            .min_by_key(|(_, berry)| berry.available)
            .map(|(index, _)| index)
    }

    fn seed_initial_food(&self, state: &mut GameState) {
        let mut seeded = 0;
        while seeded < state.max_food.min(3) {
            let Some(index) = self.next_food_spawn_index(state) else {
                break;
            };
            state.berries[index].available += 1;
            seeded += 1;
        }
    }

    fn support_level_param(&self, state: &GameState, index: usize) -> Option<Kp> {
        let level = state.supports.get(index)?.level.max(1) as usize;
        self.data
            .supports
            .get(index)?
            .level_params
            .get(level.saturating_sub(1))
            .map(|param| param.value as Kp)
    }
}

fn summarize_diamond_income(ledger: &[DiamondLedgerEntry]) -> Vec<DiamondIncomeSummary> {
    let mut summary: Vec<DiamondIncomeSummary> = Vec::new();
    for entry in ledger {
        if let Some(existing) = summary.iter_mut().find(|item| item.source == entry.source) {
            existing.amount = existing.amount.saturating_add(entry.amount);
        } else {
            summary.push(DiamondIncomeSummary {
                source: entry.source,
                amount: entry.amount,
            });
        }
    }
    summary.sort_by_key(|item| std::cmp::Reverse(item.amount));
    summary
}

fn log_event(
    action_log: &mut Vec<ActionLogEntry>,
    state: &GameState,
    event: impl Into<String>,
    detail: impl Into<String>,
) {
    let minute = state.now();
    action_log.push(ActionLogEntry {
        minute,
        day: state.clock.day + 1,
        time: format!(
            "{:02}:{:02}",
            state.clock.minute_of_day / 60,
            state.clock.minute_of_day % 60
        ),
        event: event.into(),
        detail: detail.into(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameData;
    use crate::rules::{ApkRules, ApproxRules};

    fn sim() -> WallTimeSimulator<ApproxRules> {
        WallTimeSimulator::new(ApproxRules, GameData::approx_v1(), WallSimConfig::default())
    }

    #[test]
    fn walltime_run_is_deterministic_for_seed_and_plan() {
        let data = GameData::approx_v1();
        let plan = data.preset_plan("balanced");
        let a = sim().run(7, plan.clone());
        let b = sim().run(7, plan);
        assert_eq!(a.final_state.league, b.final_state.league);
        assert_eq!(a.final_state.magikarp.kp, b.final_state.magikarp.kp);
        assert_eq!(a.purchases.len(), b.purchases.len());
    }

    #[test]
    fn sessions_respect_daily_cap() {
        let data = GameData::approx_v1();
        let config = WallSimConfig {
            max_wall_days: 3,
            max_actions: 10_000,
            max_sessions_per_day: 10,
            target_league: 4,
        };
        let result = WallTimeSimulator::new(ApproxRules, data.clone(), config)
            .run(11, data.preset_plan("none"));
        assert!(result.final_state.clock.sessions_today <= 10);
        assert!(result.final_state.clock.minute_of_day >= WallClock::SESSION_START);
    }

    #[test]
    fn sessions_are_evenly_spaced_across_active_window() {
        let data = GameData::approx_v1();
        let config = WallSimConfig {
            max_wall_days: 1,
            max_actions: 10_000,
            max_sessions_per_day: 10,
            target_league: 99,
        };
        let result = WallTimeSimulator::new(ApproxRules, data.clone(), config)
            .run(11, data.preset_plan("none"));
        let starts = result
            .action_log
            .iter()
            .filter(|entry| entry.day == 1 && entry.event == "session_start")
            .map(|entry| (entry.minute % WallClock::MINUTES_PER_DAY as u64) as u16)
            .collect::<Vec<_>>();
        assert_eq!(
            starts,
            vec![480, 560, 640, 720, 800, 880, 960, 1040, 1120, 1200]
        );
    }

    #[test]
    fn apk_food_timer_spawns_multiple_berries_during_animation_minute() {
        let data = GameData::apk_master();
        let simulator = WallTimeSimulator::new(
            ApkRules::new(&data),
            data,
            WallSimConfig {
                max_wall_days: 1,
                max_actions: 100,
                max_sessions_per_day: 10,
                target_league: 1,
            },
        );
        assert_eq!(simulator.food_spawns_per_minute_tick(), 3);
    }

    #[test]
    fn one_intentional_loss_per_reached_league() {
        let data = GameData::approx_v1();
        let result = sim().run(3, data.preset_plan("none"));
        for league in 0..result.final_state.league.min(4) as usize {
            assert!(result.final_state.league_loss_done[league]);
        }
    }

    #[test]
    fn current_magikarp_max_level_is_fixed_when_fished() {
        let data = GameData::approx_v1();
        let result = sim().run(42, data.preset_plan("none"));
        assert!(result.final_state.player_rank > 1);
        assert_eq!(result.final_state.magikarp.max_level, 11);
    }

    #[test]
    fn data_audit_marks_non_exact_fields() {
        let audit = GameData::approx_v1().audit();
        assert!(audit.assumption_fields > 0);
        assert!(!audit.warnings.is_empty());
    }

    #[test]
    fn plan_contains_only_support_and_decor_targets() {
        let data = GameData::approx_v1();
        let plan = data.preset_plan("balanced");
        assert!(
            plan.targets
                .iter()
                .all(|target| matches!(target.kind, PurchaseKind::Support | PurchaseKind::Decor))
        );
    }

    #[test]
    fn free_league_rewards_are_not_purchase_candidates() {
        let data = GameData::approx_v1();
        let ids = data
            .purchase_candidates()
            .into_iter()
            .map(|target| target.id)
            .collect::<Vec<_>>();
        assert!(!ids.contains(&"pikachu".to_string()));
        assert!(!ids.contains(&"piplup".to_string()));
        assert!(!ids.contains(&"meowth".to_string()));
        assert!(!ids.contains(&"sudowoodo".to_string()));
        assert!(ids.contains(&"litten".to_string()));
        assert!(ids.contains(&"charizard".to_string()));
    }

    #[test]
    fn diamond_income_is_ledgered_by_source() {
        let data = GameData::approx_v1();
        let result = sim().run(42, data.preset_plan("balanced"));
        assert!(result.final_state.pending_diamond_rewards.is_empty());
        assert_eq!(result.final_state.pending_achievement_claims, 0);
        assert!(
            result
                .final_state
                .diamond_ledger
                .iter()
                .any(|entry| entry.source == DiamondSource::Tutorial)
        );
        assert!(
            result
                .final_state
                .diamond_ledger
                .iter()
                .any(|entry| entry.source == DiamondSource::TrainerRank)
        );
        assert!(
            result
                .final_state
                .diamond_ledger
                .iter()
                .any(|entry| entry.source == DiamondSource::LeagueBattleReward)
        );
        assert!(
            result
                .final_state
                .diamond_ledger
                .iter()
                .any(|entry| entry.source == DiamondSource::Achievement)
        );
        assert!(
            result
                .final_state
                .diamond_ledger
                .iter()
                .any(|entry| entry.source == DiamondSource::PatternDiscovery)
        );
    }

    #[test]
    fn feeding_and_training_achievements_do_not_award_diamonds() {
        let data = GameData::approx_v1();
        let result = sim().run(42, data.preset_plan("balanced"));
        assert!(
            !result
                .final_state
                .diamond_ledger
                .iter()
                .any(|entry| entry.detail.starts_with("times fed"))
        );
        assert!(
            !result
                .final_state
                .diamond_ledger
                .iter()
                .any(|entry| entry.detail.starts_with("times trained"))
        );
    }
}
