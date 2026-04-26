use serde::Serialize;

pub type Kp = u128;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum Provenance {
    Code,
    Disassembly,
    Asset,
    Wiki,
    Assumption,
}

#[derive(Clone, Debug, Serialize)]
pub struct Provenanced<T> {
    pub value: T,
    pub provenance: Provenance,
    pub source: &'static str,
}

impl<T> Provenanced<T> {
    pub const fn new(value: T, provenance: Provenance, source: &'static str) -> Self {
        Self {
            value,
            provenance,
            source,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct GameState {
    pub elapsed_minutes: u64,
    pub action_count: u32,
    pub clock: WallClock,
    pub trainer_exp: u128,
    pub player_rank: u32,
    pub coins: u64,
    pub diamonds: u32,
    pub diamond_ledger: Vec<DiamondLedgerEntry>,
    pub candy: u32,
    pub training_sodas: u32,
    pub skill_herbs: u32,
    pub league_aids: u32,
    pub stamina: u32,
    pub max_stamina: u32,
    pub food_level: u32,
    pub training_level: u32,
    pub food_available: u32,
    pub max_food: u32,
    pub league: u32,
    pub competition: u32,
    pub generation: u32,
    pub retirements: u32,
    pub berries: Vec<BerryState>,
    pub trainings: Vec<TrainingState>,
    pub supports: Vec<SupportState>,
    pub decors: Vec<DecorState>,
    pub pending_achievement_claims: u32,
    pub pending_diamond_rewards: Vec<PendingDiamondReward>,
    pub pending_coin_rewards: Vec<PendingCoinReward>,
    pub pending_candy_rewards: Vec<PendingCandyReward>,
    pub diamond_achievement_keys: Vec<String>,
    pub league_wins_total: u32,
    pub support_skill_uses: u32,
    pub items_used: u32,
    pub random_events_seen: u32,
    pub random_event_ids_seen: Vec<u32>,
    pub random_event_retirements: u32,
    pub random_event_day: u32,
    pub training_random_events_today: u32,
    pub league_win_random_events_today: u32,
    pub league_loss_random_events_today: u32,
    pub discovered_patterns: u32,
    pub login_days_claimed: u32,
    pub home_treasure_ready_at: u64,
    pub home_random_event_ready_at: u64,
    pub next_food_spawn_at: u64,
    pub next_stamina_at: u64,
    pub kp_gain_buff_until: u64,
    pub kp_gain_buff_permyriad: u32,
    pub magikarp: MagikarpState,
}

#[derive(Clone, Debug, Serialize)]
pub struct MagikarpState {
    pub level: u32,
    pub max_level: u32,
    pub kp: Kp,
    pub individual_bonus_permyriad: u32,
    pub pattern_rarity: u32,
    pub foods_eaten: u32,
    pub trainings_done: u32,
    pub wins: u32,
    pub level_coin_bonus_claimed_to: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct DiamondLedgerEntry {
    pub minute: u64,
    pub amount: u32,
    pub source: DiamondSource,
    pub detail: String,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Serialize)]
pub struct PendingDiamondReward {
    pub amount: u32,
    pub source: DiamondSource,
    pub detail: String,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Serialize)]
pub struct PendingCoinReward {
    pub amount: u64,
    pub detail: String,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Serialize)]
pub struct PendingCandyReward {
    pub amount: u32,
    pub detail: String,
    pub provenance: Provenance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum DiamondSource {
    Tutorial,
    TrainerRank,
    LeagueBattleReward,
    Achievement,
    RandomEncounter,
    SunkenTreasure,
    PatternDiscovery,
    SupportSkill,
    DiamondMiner,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum Action {
    FeedAll,
    Train,
    Compete,
    StartSession,
    CollectGold,
    ClaimAchievement,
    UseSupportSkill(&'static str),
    BuySupport(&'static str),
    BuyDecor(&'static str),
    UpgradeBerry(&'static str),
    EatBerry(&'static str, u32),
    LeagueFight { intentional_loss: bool },
    WaitUntil(u64),
    EndSession,
    BuyFoodUpgrade,
    BuyTrainingUpgrade,
    Rest { minutes: u32 },
    RetireAndFish,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct WallClock {
    pub day: u32,
    pub minute_of_day: u16,
    pub sessions_today: u8,
}

impl Default for WallClock {
    fn default() -> Self {
        Self {
            day: 0,
            minute_of_day: Self::SESSION_START,
            sessions_today: 0,
        }
    }
}

impl WallClock {
    pub const SESSION_START: u16 = 8 * 60;
    pub const SESSION_END: u16 = 20 * 60;
    pub const MINUTES_PER_DAY: u16 = 24 * 60;

    pub fn absolute_minutes(self) -> u64 {
        self.day as u64 * Self::MINUTES_PER_DAY as u64 + self.minute_of_day as u64
    }

    pub fn from_absolute_minutes(minutes: u64, sessions_today: u8) -> Self {
        Self {
            day: (minutes / Self::MINUTES_PER_DAY as u64) as u32,
            minute_of_day: (minutes % Self::MINUTES_PER_DAY as u64) as u16,
            sessions_today,
        }
    }

    pub fn in_session_window(self) -> bool {
        self.minute_of_day >= Self::SESSION_START && self.minute_of_day <= Self::SESSION_END
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct BerryState {
    pub id: &'static str,
    pub name: &'static str,
    pub pair_group: &'static str,
    pub level: u32,
    pub available: u32,
    pub max_available: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct TrainingState {
    pub id: &'static str,
    pub name: &'static str,
    pub level: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct SupportState {
    pub id: &'static str,
    pub name: &'static str,
    pub owned: bool,
    pub level: u32,
    pub ready_at: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct DecorState {
    pub id: &'static str,
    pub name: &'static str,
    pub owned: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum PurchaseKind {
    Support,
    Decor,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PurchaseTarget {
    pub kind: PurchaseKind,
    pub id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PurchasePlan {
    pub name: String,
    pub targets: Vec<PurchaseTarget>,
}

impl GameState {
    pub fn is_magikarp_maxed(&self) -> bool {
        self.magikarp.level >= self.magikarp.max_level
    }

    pub fn now(&self) -> u64 {
        self.clock.absolute_minutes()
    }

    pub fn set_now(&mut self, absolute_minutes: u64) {
        self.clock = WallClock::from_absolute_minutes(absolute_minutes, self.clock.sessions_today);
        self.elapsed_minutes = absolute_minutes.saturating_sub(WallClock::SESSION_START as u64);
    }
}
