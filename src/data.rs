use serde::{Deserialize, Serialize};

use crate::model::{Provenance, Provenanced, PurchaseKind, PurchasePlan, PurchaseTarget};

#[derive(Clone, Debug, Serialize)]
pub struct GameData {
    pub name: &'static str,
    pub sources: Vec<&'static str>,
    pub berries: Vec<BerryData>,
    pub trainings: Vec<TrainingData>,
    pub supports: Vec<SupportItemData>,
    pub decors: Vec<DecorItemData>,
    pub leagues: Vec<LeagueData>,
    pub economy: EconomyData,
    pub random_events: Vec<RandomEventData>,
    pub random_event_parameters: RandomEventParameters,
    pub treasure_rewards: Vec<TreasureRewardData>,
    pub food_upgrade_costs: Vec<Vec<Provenanced<u64>>>,
    pub training_upgrade_costs: Vec<Vec<Provenanced<u64>>>,
    pub breeder_ranks: Vec<BreederRankData>,
    pub magikarp_ranks: Vec<MagikarpRankData>,
    pub jump_curve: Vec<JumpCurvePoint>,
}

#[derive(Clone, Debug, Serialize)]
pub struct BerryData {
    pub id: &'static str,
    pub name: &'static str,
    pub pair_group: &'static str,
    pub unlock_rank: u32,
    pub base_kp: Provenanced<u128>,
    pub jp_by_rank: Vec<CurvePoint>,
    pub upgrade_cost_base: Provenanced<u64>,
    pub max_level: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct CurvePoint {
    pub rank: u32,
    pub value: Provenanced<u128>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TrainingData {
    pub id: &'static str,
    pub name: &'static str,
    pub unlock_rank: u32,
    pub jp_by_rank: Vec<CurvePoint>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SupportItemData {
    pub id: &'static str,
    pub name: &'static str,
    pub acquisition: Acquisition,
    pub cooldown_minutes: Provenanced<u32>,
    pub level_params: Vec<Provenanced<u32>>,
    pub upgrade_candy_costs: Vec<Provenanced<u32>>,
    pub unlock_league: u32,
    pub skill: SupportSkill,
}

#[derive(Clone, Debug, Serialize)]
pub enum SupportSkill {
    KpFlat { base: u128 },
    Coins { base: u64 },
    Stamina { amount: u32 },
    Diamonds { amount: u32 },
    Food { amount: u32 },
    Item { base_coin_value: u64 },
    RecoverSkills,
    LeaguePoint,
    TrainingGreat,
    KpBoost { multiplier_permyriad: u32 },
}

#[derive(Clone, Debug, Serialize)]
pub struct DecorItemData {
    pub id: &'static str,
    pub name: &'static str,
    pub acquisition: Acquisition,
    pub effect: DecorEffect,
}

#[derive(Clone, Debug, Serialize)]
pub enum DecorEffect {
    KpPermyriad(u32),
    FoodKpPermyriad(u32),
    EventKpPermyriad(u32),
    CoinPermyriad(u32),
    LeagueCoinPermyriad(u32),
    TrainingPermyriad(u32),
    SkillKpPermyriad(u32),
    EventCoinPermyriad(u32),
    TrainerExpPermyriad(u32),
    TrainingEventPermyriad(u32),
    SkillRecoveryPermyriad(u32),
    LevelUpCoinPermyriad(u32),
    TreasureCoinPermyriad(u32),
    LeagueEventPermyriad(u32),
    FoodCapacity(u32),
    Unknown,
}

#[derive(Clone, Debug, Serialize)]
pub enum Acquisition {
    DiamondShop {
        price_diamonds: Provenanced<u32>,
    },
    LeagueReward {
        after_league: u32,
        provenance: Provenance,
        source: &'static str,
    },
    BattleReward {
        league: u32,
        competition: u32,
        provenance: Provenance,
        source: &'static str,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct LeagueData {
    pub id: u32,
    pub name: &'static str,
    pub competitions: Vec<CompetitionData>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CompetitionData {
    pub id: u32,
    pub name: &'static str,
    pub opponent_jump_cm: Provenanced<u64>,
    pub win_reward_coins: Provenanced<u64>,
    pub loss_reward_coins: Provenanced<u64>,
    pub reward_diamonds: Provenanced<u32>,
    pub reward_candy: Provenanced<u32>,
    pub reward_support_id: u32,
    pub reward_decor_id: u32,
    pub breeder_exp_win: Provenanced<u128>,
}

#[derive(Clone, Debug, Serialize)]
pub struct BreederRankData {
    pub rank: u32,
    pub need_exp: Provenanced<u128>,
    pub magikarp_max_rank: Provenanced<u32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MagikarpRankData {
    pub rank: u32,
    pub need_kp: Provenanced<u128>,
    pub retirement_breeder_exp: Provenanced<u128>,
    pub level_up_coins: Provenanced<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct JumpCurvePoint {
    pub need_kp: Provenanced<u128>,
    pub height: Provenanced<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct EconomyData {
    pub initial_diamonds: Provenanced<u32>,
    pub trainer_rank_up_diamonds: Provenanced<u32>,
    pub home_treasure_cooldown_minutes: Provenanced<u32>,
    pub home_treasure_base_coins: Provenanced<u64>,
    pub achievement_diamonds_minor: Provenanced<u32>,
    pub achievement_diamonds_small: Provenanced<u32>,
    pub achievement_diamonds_medium: Provenanced<u32>,
    pub league_battle_milestone_diamonds: Provenanced<u32>,
    pub league_clear_support_candy: Provenanced<u32>,
    pub pattern_discovery_diamonds: Provenanced<u32>,
    pub random_event_diamonds: Provenanced<u32>,
    pub random_event_diamond_chance_permyriad: Provenanced<u32>,
    pub sunken_treasure_diamonds: Provenanced<u32>,
    pub sunken_treasure_diamond_chance_permyriad: Provenanced<u32>,
    pub diamond_miner_diamonds: Provenanced<u32>,
    pub diamond_miner_cooldown_minutes: Provenanced<u32>,
    pub diamond_miner_f2p_enabled: Provenanced<bool>,
    pub food_respawn_minutes: Provenanced<u32>,
    pub food_respawn_seconds: Provenanced<u32>,
    pub home_food_max: Provenanced<u32>,
    pub manaphy_food_num: Provenanced<u32>,
    pub stamina_respawn_minutes: Provenanced<u32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TreasureRewardData {
    pub genre_id: u32,
    pub freq: u32,
    pub num: u32,
    pub memo: &'static str,
    pub provenance: Provenance,
    pub source: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct RandomEventParameters {
    pub training_chance_permyriad: Provenanced<u32>,
    pub training_max_per_day: Provenanced<u32>,
    pub league_win_chance_permyriad: Provenanced<u32>,
    pub league_win_max_per_day: Provenanced<u32>,
    pub league_loss_chance_permyriad: Provenanced<u32>,
    pub league_loss_max_per_day: Provenanced<u32>,
    pub home_chance_permyriad: Provenanced<u32>,
    pub home_cooldown_minutes: Provenanced<u32>,
    pub home_max_cooldown_minutes: Provenanced<u32>,
}

impl RandomEventParameters {
    fn approx(source: &'static str) -> Self {
        Self {
            training_chance_permyriad: Provenanced::new(250, Provenance::Assumption, source),
            training_max_per_day: Provenanced::new(3, Provenance::Assumption, source),
            league_win_chance_permyriad: Provenanced::new(0, Provenance::Assumption, source),
            league_win_max_per_day: Provenanced::new(0, Provenance::Assumption, source),
            league_loss_chance_permyriad: Provenanced::new(0, Provenance::Assumption, source),
            league_loss_max_per_day: Provenanced::new(0, Provenance::Assumption, source),
            home_chance_permyriad: Provenanced::new(0, Provenance::Assumption, source),
            home_cooldown_minutes: Provenanced::new(240, Provenance::Assumption, source),
            home_max_cooldown_minutes: Provenanced::new(1_440, Provenance::Assumption, source),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum RandomEventOccurrence {
    Home,
    Training,
    LeagueWin,
    LeagueLoss,
}

#[derive(Clone, Debug, Serialize)]
pub struct RandomEventData {
    pub id: u32,
    pub name: &'static str,
    pub occurrence: RandomEventOccurrence,
    pub need_league_id: u32,
    pub need_support_pokemon_id: u32,
    pub need_generation: u32,
    pub bonus_type: u32,
    pub bonus_num: u32,
    pub success_bonus_type: u32,
    pub success_bonus_num: u32,
    pub success_chance_permyriad: u32,
    pub penalty_type: u32,
    pub penalty_num: u32,
    pub freq: u32,
    pub provenance: Provenance,
    pub source: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct DataAuditReport {
    pub dataset: &'static str,
    pub sources: Vec<&'static str>,
    pub total_fields: usize,
    pub exact_fields: usize,
    pub assumption_fields: usize,
    pub wiki_fields: usize,
    pub warnings: Vec<String>,
}

impl GameData {
    pub fn approx_v1() -> Self {
        let code_src = "APK symbols: libMyGame.so ResourceUtils/MagicarpData/CompetitionManager";
        let assumption_src = "approx-v1 placeholder until encrypted master data is decoded";
        let wiki_src =
            "Magikarp Jump Wiki/Bulbapedia secondary item tables; validate against APK master data";

        let berries = vec![
            BerryData {
                id: "oran",
                name: "Sinelbeere / Oran Berry",
                pair_group: "primary_equal",
                unlock_rank: 1,
                base_kp: Provenanced::new(2, Provenance::Wiki, wiki_src),
                jp_by_rank: curve_points(
                    &[
                        (1, 2),
                        (25, 81),
                        (50, 2_550),
                        (75, 76_300),
                        (100, 3_189_075),
                    ],
                    Provenance::Wiki,
                    wiki_src,
                ),
                upgrade_cost_base: Provenanced::new(90, Provenance::Assumption, assumption_src),
                max_level: 100,
            },
            BerryData {
                id: "sitrus",
                name: "Tsitrusbeere / Sitrus Berry",
                pair_group: "primary_equal",
                unlock_rank: 1,
                base_kp: Provenanced::new(9, Provenance::Wiki, wiki_src),
                jp_by_rank: curve_points(
                    &[
                        (1, 9),
                        (25, 163),
                        (50, 3_367),
                        (75, 90_340),
                        (100, 3_724_529),
                    ],
                    Provenance::Wiki,
                    wiki_src,
                ),
                upgrade_cost_base: Provenanced::new(95, Provenance::Assumption, assumption_src),
                max_level: 100,
            },
        ];

        let trainings = vec![
            TrainingData {
                id: "sandbag_slam",
                name: "Sandbag Slam",
                unlock_rank: 1,
                jp_by_rank: curve_points(&[(1, 35)], Provenance::Wiki, wiki_src),
            },
            TrainingData {
                id: "jump_counter",
                name: "Jump Counter",
                unlock_rank: 1,
                jp_by_rank: curve_points(&[(1, 122)], Provenance::Wiki, wiki_src),
            },
        ];

        let supports = vec![
            SupportItemData {
                id: "pikachu",
                name: "Light Ball / Pikachu",
                acquisition: Acquisition::LeagueReward {
                    after_league: 1,
                    provenance: Provenance::Wiki,
                    source: wiki_src,
                },
                cooldown_minutes: Provenanced::new(90, Provenance::Wiki, wiki_src),
                level_params: Vec::new(),
                upgrade_candy_costs: support_candy_costs(&[1, 2, 5]),
                unlock_league: 1,
                skill: SupportSkill::KpFlat { base: 900 },
            },
            SupportItemData {
                id: "piplup",
                name: "Mystic Water / Piplup",
                acquisition: Acquisition::LeagueReward {
                    after_league: 2,
                    provenance: Provenance::Wiki,
                    source: wiki_src,
                },
                cooldown_minutes: Provenanced::new(90, Provenance::Wiki, wiki_src),
                level_params: Vec::new(),
                upgrade_candy_costs: support_candy_costs(&[10]),
                unlock_league: 2,
                skill: SupportSkill::Stamina { amount: 1 },
            },
            SupportItemData {
                id: "meowth",
                name: "Amulet Coin / Meowth",
                acquisition: Acquisition::LeagueReward {
                    after_league: 3,
                    provenance: Provenance::Wiki,
                    source: wiki_src,
                },
                cooldown_minutes: Provenanced::new(150, Provenance::Wiki, wiki_src),
                level_params: Vec::new(),
                upgrade_candy_costs: support_candy_costs(&[1, 2, 5, 7]),
                unlock_league: 3,
                skill: SupportSkill::Coins { base: 450 },
            },
            SupportItemData {
                id: "bulbasaur",
                name: "Miracle Seed / Bulbasaur",
                acquisition: Acquisition::LeagueReward {
                    after_league: 5,
                    provenance: Provenance::Wiki,
                    source: wiki_src,
                },
                cooldown_minutes: Provenanced::new(240, Provenance::Wiki, wiki_src),
                level_params: Vec::new(),
                upgrade_candy_costs: Vec::new(),
                unlock_league: 5,
                skill: SupportSkill::LeaguePoint,
            },
            SupportItemData {
                id: "mudkip",
                name: "Soft Sand / Mudkip",
                acquisition: Acquisition::LeagueReward {
                    after_league: 8,
                    provenance: Provenance::Wiki,
                    source: wiki_src,
                },
                cooldown_minutes: Provenanced::new(600, Provenance::Wiki, wiki_src),
                level_params: Vec::new(),
                upgrade_candy_costs: Vec::new(),
                unlock_league: 8,
                skill: SupportSkill::Item {
                    base_coin_value: 650,
                },
            },
            SupportItemData {
                id: "litten",
                name: "Flame Plate / Litten",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(250, Provenance::Wiki, wiki_src),
                },
                cooldown_minutes: Provenanced::new(210, Provenance::Wiki, wiki_src),
                level_params: Vec::new(),
                upgrade_candy_costs: Vec::new(),
                unlock_league: 1,
                skill: SupportSkill::KpFlat { base: 1_400 },
            },
            SupportItemData {
                id: "slowpoke",
                name: "Damp Rock / Slowpoke",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(350, Provenance::Wiki, wiki_src),
                },
                cooldown_minutes: Provenanced::new(720, Provenance::Wiki, wiki_src),
                level_params: Vec::new(),
                upgrade_candy_costs: Vec::new(),
                unlock_league: 1,
                skill: SupportSkill::RecoverSkills,
            },
            SupportItemData {
                id: "mimikyu",
                name: "Spell Tag / Mimikyu",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(400, Provenance::Wiki, wiki_src),
                },
                cooldown_minutes: Provenanced::new(94, Provenance::Wiki, wiki_src),
                level_params: Vec::new(),
                upgrade_candy_costs: support_candy_costs(&[1, 2, 5]),
                unlock_league: 1,
                skill: SupportSkill::KpFlat { base: 1_100 },
            },
            SupportItemData {
                id: "rowlet",
                name: "Meadow Plate / Rowlet",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(450, Provenance::Wiki, wiki_src),
                },
                cooldown_minutes: Provenanced::new(240, Provenance::Wiki, wiki_src),
                level_params: Vec::new(),
                upgrade_candy_costs: Vec::new(),
                unlock_league: 1,
                skill: SupportSkill::Coins { base: 1_200 },
            },
            SupportItemData {
                id: "snorlax",
                name: "Leftovers / Snorlax",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(500, Provenance::Wiki, wiki_src),
                },
                cooldown_minutes: Provenanced::new(50, Provenance::Wiki, wiki_src),
                level_params: Vec::new(),
                upgrade_candy_costs: Vec::new(),
                unlock_league: 1,
                skill: SupportSkill::Food { amount: 10 },
            },
            SupportItemData {
                id: "greninja",
                name: "Shell Bell / Greninja",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(500, Provenance::Wiki, wiki_src),
                },
                cooldown_minutes: Provenanced::new(300, Provenance::Wiki, wiki_src),
                level_params: Vec::new(),
                upgrade_candy_costs: support_candy_costs(&[8, 10, 15]),
                unlock_league: 1,
                skill: SupportSkill::KpFlat { base: 3_000 },
            },
            SupportItemData {
                id: "popplio",
                name: "Splash Plate / Popplio",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(500, Provenance::Wiki, wiki_src),
                },
                cooldown_minutes: Provenanced::new(420, Provenance::Wiki, wiki_src),
                level_params: Vec::new(),
                upgrade_candy_costs: Vec::new(),
                unlock_league: 1,
                skill: SupportSkill::Item {
                    base_coin_value: 900,
                },
            },
            SupportItemData {
                id: "eevee",
                name: "Soothe Bell / Eevee",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(500, Provenance::Wiki, wiki_src),
                },
                cooldown_minutes: Provenanced::new(180, Provenance::Wiki, wiki_src),
                level_params: Vec::new(),
                upgrade_candy_costs: Vec::new(),
                unlock_league: 1,
                skill: SupportSkill::TrainingGreat,
            },
            SupportItemData {
                id: "charizard",
                name: "Charcoal / Charizard",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(600, Provenance::Wiki, wiki_src),
                },
                cooldown_minutes: Provenanced::new(720, Provenance::Wiki, wiki_src),
                level_params: Vec::new(),
                upgrade_candy_costs: Vec::new(),
                unlock_league: 1,
                skill: SupportSkill::Item {
                    base_coin_value: 1_400,
                },
            },
            SupportItemData {
                id: "gengar",
                name: "Black Sludge / Gengar",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(600, Provenance::Wiki, wiki_src),
                },
                cooldown_minutes: Provenanced::new(840, Provenance::Wiki, wiki_src),
                level_params: Vec::new(),
                upgrade_candy_costs: Vec::new(),
                unlock_league: 1,
                skill: SupportSkill::KpBoost {
                    multiplier_permyriad: 15_000,
                },
            },
            SupportItemData {
                id: "gardevoir",
                name: "Choice Scarf / Gardevoir",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(600, Provenance::Wiki, wiki_src),
                },
                cooldown_minutes: Provenanced::new(360, Provenance::Wiki, wiki_src),
                level_params: Vec::new(),
                upgrade_candy_costs: support_candy_costs(&[8, 10, 15]),
                unlock_league: 1,
                skill: SupportSkill::Coins { base: 2_200 },
            },
        ];

        let decors = vec![
            DecorItemData {
                id: "shaymin_planter",
                name: "Shaymin Planter",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(250, Provenance::Wiki, wiki_src),
                },
                effect: DecorEffect::KpPermyriad(11_400),
            },
            DecorItemData {
                id: "parasect_puffballs",
                name: "Parasect Puffballs",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(100, Provenance::Wiki, wiki_src),
                },
                effect: DecorEffect::KpPermyriad(11_600),
            },
            DecorItemData {
                id: "exeggutor_palm",
                name: "Exeggutor Palm",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(150, Provenance::Wiki, wiki_src),
                },
                effect: DecorEffect::CoinPermyriad(11_200),
            },
            DecorItemData {
                id: "sunflora_bloom",
                name: "Sunflora Bloom",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(200, Provenance::Wiki, wiki_src),
                },
                effect: DecorEffect::EventCoinPermyriad(12_800),
            },
            DecorItemData {
                id: "lampent_lamp",
                name: "Lampent Lamp",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(250, Provenance::Wiki, wiki_src),
                },
                effect: DecorEffect::LevelUpCoinPermyriad(12_900),
            },
            DecorItemData {
                id: "substitute_plush",
                name: "Substitute Plush",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(300, Provenance::Wiki, wiki_src),
                },
                effect: DecorEffect::FoodCapacity(2),
            },
            DecorItemData {
                id: "clefairy_doll",
                name: "Clefairy Doll",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(350, Provenance::Wiki, wiki_src),
                },
                effect: DecorEffect::SkillKpPermyriad(12_000),
            },
            DecorItemData {
                id: "aegislash_statue",
                name: "Aegislash Statue",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(350, Provenance::Wiki, wiki_src),
                },
                effect: DecorEffect::TrainingPermyriad(12_000),
            },
            DecorItemData {
                id: "dugtrio_rock",
                name: "Dugtrio Rock",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(350, Provenance::Wiki, wiki_src),
                },
                effect: DecorEffect::LevelUpCoinPermyriad(13_200),
            },
            DecorItemData {
                id: "cacnea_planter",
                name: "Cacnea Planter",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(400, Provenance::Wiki, wiki_src),
                },
                effect: DecorEffect::SkillRecoveryPermyriad(11_000),
            },
            DecorItemData {
                id: "ss_anne_model",
                name: "S.S. Anne Model",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(450, Provenance::Wiki, wiki_src),
                },
                effect: DecorEffect::TreasureCoinPermyriad(16_800),
            },
            DecorItemData {
                id: "whimsicott_cushion",
                name: "Whimsicott Cushion",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(450, Provenance::Wiki, wiki_src),
                },
                effect: DecorEffect::TrainerExpPermyriad(10_900),
            },
            DecorItemData {
                id: "lilligant_doll",
                name: "Lilligant Doll",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(450, Provenance::Wiki, wiki_src),
                },
                effect: DecorEffect::TrainingEventPermyriad(10_600),
            },
            DecorItemData {
                id: "bronze_eevee",
                name: "Bronze Eevee",
                acquisition: Acquisition::DiamondShop {
                    price_diamonds: Provenanced::new(550, Provenance::Wiki, wiki_src),
                },
                effect: DecorEffect::LeagueEventPermyriad(10_400),
            },
            DecorItemData {
                id: "octillery_pot",
                name: "Octillery Pot",
                acquisition: Acquisition::LeagueReward {
                    after_league: 2,
                    provenance: Provenance::Wiki,
                    source: wiki_src,
                },
                effect: DecorEffect::KpPermyriad(11_000),
            },
            DecorItemData {
                id: "sudowoodo",
                name: "Sudowoodo Bonsai",
                acquisition: Acquisition::LeagueReward {
                    after_league: 4,
                    provenance: Provenance::Wiki,
                    source: wiki_src,
                },
                effect: DecorEffect::EventCoinPermyriad(12_900),
            },
            DecorItemData {
                id: "important_sign",
                name: "Important Sign",
                acquisition: Acquisition::BattleReward {
                    league: 2,
                    competition: 1,
                    provenance: Provenance::Wiki,
                    source: wiki_src,
                },
                effect: DecorEffect::FoodCapacity(1),
            },
            DecorItemData {
                id: "starmie_bubbler",
                name: "Starmie Bubbler",
                acquisition: Acquisition::LeagueReward {
                    after_league: 7,
                    provenance: Provenance::Wiki,
                    source: wiki_src,
                },
                effect: DecorEffect::Unknown,
            },
        ];

        let leagues = wiki_leagues(wiki_src, assumption_src);

        Self {
            name: "approx-v1-walltime",
            sources: vec![
                code_src,
                "native_symbols_demangled.txt",
                "business_symbols_focus.txt",
                wiki_src,
            ],
            berries,
            trainings,
            supports,
            decors,
            leagues,
            economy: EconomyData {
                initial_diamonds: Provenanced::new(
                    100,
                    Provenance::Assumption,
                    "tutorial/intro/start wallet aggregate until exact rewards are decoded",
                ),
                trainer_rank_up_diamonds: Provenanced::new(
                    10,
                    Provenance::Disassembly,
                    "OtherParameter::getPlayerRankUpBonusDiaNum symbol identified; amount cross-checked by wiki",
                ),
                home_treasure_cooldown_minutes: Provenanced::new(
                    180,
                    Provenance::Disassembly,
                    "OtherParameter::getHomeTreasureDurationSec symbol identified",
                ),
                home_treasure_base_coins: Provenanced::new(
                    80,
                    Provenance::Assumption,
                    assumption_src,
                ),
                achievement_diamonds_minor: Provenanced::new(5, Provenance::Wiki, wiki_src),
                achievement_diamonds_small: Provenanced::new(25, Provenance::Wiki, wiki_src),
                achievement_diamonds_medium: Provenanced::new(50, Provenance::Wiki, wiki_src),
                league_battle_milestone_diamonds: Provenanced::new(25, Provenance::Wiki, wiki_src),
                league_clear_support_candy: Provenanced::new(1, Provenance::Wiki, wiki_src),
                pattern_discovery_diamonds: Provenanced::new(10, Provenance::Wiki, wiki_src),
                random_event_diamonds: Provenanced::new(5, Provenance::Wiki, wiki_src),
                random_event_diamond_chance_permyriad: Provenanced::new(
                    350,
                    Provenance::Assumption,
                    "approximate combined chance for All That Glitters/Lost Luvdisc/Name Rater diamond events",
                ),
                sunken_treasure_diamonds: Provenanced::new(5, Provenance::Wiki, wiki_src),
                sunken_treasure_diamond_chance_permyriad: Provenanced::new(
                    800,
                    Provenance::Assumption,
                    "approximate chance for diamonds from home treasure or Popplio treasure",
                ),
                diamond_miner_diamonds: Provenanced::new(100, Provenance::Wiki, wiki_src),
                diamond_miner_cooldown_minutes: Provenanced::new(
                    22 * 60,
                    Provenance::Wiki,
                    wiki_src,
                ),
                diamond_miner_f2p_enabled: Provenanced::new(
                    false,
                    Provenance::Wiki,
                    "Diamond Miner requires Exchange Tickets/IAP path; disabled for F2P simulations",
                ),
                food_respawn_minutes: Provenanced::new(
                    8,
                    Provenance::Disassembly,
                    "OtherParameter::getHomeFoodNeedSec symbol identified",
                ),
                food_respawn_seconds: Provenanced::new(
                    8 * 60,
                    Provenance::Disassembly,
                    "OtherParameter::getHomeFoodNeedSec symbol identified",
                ),
                home_food_max: Provenanced::new(3, Provenance::Assumption, assumption_src),
                manaphy_food_num: Provenanced::new(25, Provenance::Assumption, assumption_src),
                stamina_respawn_minutes: Provenanced::new(
                    30,
                    Provenance::Disassembly,
                    "Stamina respawn behavior inferred from simulator v0 and symbols",
                ),
            },
            random_events: Vec::new(),
            random_event_parameters: RandomEventParameters::approx(assumption_src),
            treasure_rewards: Vec::new(),
            food_upgrade_costs: Vec::new(),
            training_upgrade_costs: Vec::new(),
            breeder_ranks: Vec::new(),
            magikarp_ranks: Vec::new(),
            jump_curve: Vec::new(),
        }
    }

    pub fn apk_master() -> Self {
        let asset_src = "decoded APK master_data_enc JSON recovered via Ghidra ResourceUtils";
        let disasm_src = "Ghidra: libMyGame.so ResourceUtils/MagicarpData/CompetitionManager";

        let food_base: Vec<FoodBaseRow> = json_rows(include_str!(
            "../decoded_master_data/food_base_data.json"
        ));
        let food_power: Vec<LevelTableRow> =
            json_rows(include_str!("../decoded_master_data/food_power.json"));
        let food_price: Vec<LevelTableRow> =
            json_rows(include_str!("../decoded_master_data/food_price.json"));
        let training_base: Vec<TrainingBaseRow> = json_rows(include_str!(
            "../decoded_master_data/training_base_data.json"
        ));
        let training_min_power: Vec<LevelTableRow> = json_rows(include_str!(
            "../decoded_master_data/training_min_power.json"
        ));
        let training_price: Vec<LevelTableRow> = json_rows(include_str!(
            "../decoded_master_data/training_price.json"
        ));
        let support_rows: Vec<SupportPokemonRow> = json_rows(include_str!(
            "../decoded_master_data/support_pokemon.json"
        ));
        let decor_rows: Vec<DecorationRow> =
            json_rows(include_str!("../decoded_master_data/decoration.json"));
        let league_rows: Vec<LeagueRow> =
            json_rows(include_str!("../decoded_master_data/league_list.json"));
        let competition_rows: Vec<CompetitionRow> = json_rows(include_str!(
            "../decoded_master_data/competition_list.json"
        ));
        let breeder_rows: Vec<BreederRankRow> =
            json_rows(include_str!("../decoded_master_data/breeder_rank.json"));
        let magikarp_rows: Vec<MagikarpRankRow> =
            json_rows(include_str!("../decoded_master_data/magikarp_rank.json"));
        let jump_rows: Vec<JumpCurveRow> = json_rows(include_str!(
            "../decoded_master_data/kp_to_jump_height.json"
        ));
        let other_rows: Vec<OtherParametersRow> = json_rows(include_str!(
            "../decoded_master_data/other_parameters.json"
        ));
        let treasure_rows: Vec<TreasureRow> =
            json_rows(include_str!("../decoded_master_data/treasure_data.json"));
        let random_event_rows: Vec<RandomEventRow> = json_rows(include_str!(
            "../decoded_master_data/random_event_list.json"
        ));
        let random_event_parameter_rows: Vec<RandomEventParametersRow> = json_rows(include_str!(
            "../decoded_master_data/random_event_parameters.json"
        ));
        let other = other_rows
            .first()
            .expect("other_parameters.json contains one row");
        let random_event_parameters_row = random_event_parameter_rows
            .first()
            .expect("random_event_parameters.json contains one row");

        let food_upgrade_costs =
            table_matrix(&food_price, "food_", 17, Provenance::Asset, asset_src);
        let training_upgrade_costs = table_matrix(
            &training_price,
            "training_",
            17,
            Provenance::Asset,
            asset_src,
        );
        let food_power_table = table_matrix(&food_power, "food_", 17, Provenance::Asset, asset_src);
        let training_power_table = table_matrix(
            &training_min_power,
            "training_",
            17,
            Provenance::Asset,
            asset_src,
        );

        let berries = food_base
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let level_values = food_power_table
                    .iter()
                    .enumerate()
                    .map(|(level, values)| CurvePoint {
                        rank: level as u32 + 1,
                        value: Provenanced::new(
                            values[index].value as u128,
                            Provenance::Asset,
                            asset_src,
                        ),
                    })
                    .collect::<Vec<_>>();
                BerryData {
                    id: leak_str(format!("food_{}", row.id)),
                    name: leak_str(format!("{} / food_{}", row.name_memo, row.id)),
                    pair_group: if row.id == "1" || row.id == "2" {
                        "primary_equal"
                    } else {
                        "other"
                    },
                    unlock_rank: parse_u32(&row.unlock_breeder_rank),
                    base_kp: level_values
                        .first()
                        .map(|point| point.value.clone())
                        .unwrap_or_else(|| Provenanced::new(0, Provenance::Asset, asset_src)),
                    jp_by_rank: level_values,
                    upgrade_cost_base: Provenanced::new(
                        food_upgrade_costs
                            .first()
                            .and_then(|row| row.get(index))
                            .map(|cost| cost.value)
                            .unwrap_or(0),
                        Provenance::Asset,
                        asset_src,
                    ),
                    max_level: 100,
                }
            })
            .collect::<Vec<_>>();

        let trainings = training_base
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let level_values = training_power_table
                    .iter()
                    .enumerate()
                    .map(|(level, values)| CurvePoint {
                        rank: level as u32 + 1,
                        value: Provenanced::new(
                            values[index].value as u128,
                            Provenance::Asset,
                            asset_src,
                        ),
                    })
                    .collect::<Vec<_>>();
                TrainingData {
                    id: leak_str(format!("training_{}", row.id)),
                    name: leak_str(format!("{} / training_{}", row.name_memo, row.id)),
                    unlock_rank: parse_u32(&row.unlock_breeder_rank),
                    jp_by_rank: level_values,
                }
            })
            .collect::<Vec<_>>();

        let competition_positions = competition_rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                (
                    parse_u32(&row.id),
                    competition_position(&competition_rows, index),
                )
            })
            .collect::<std::collections::HashMap<_, _>>();

        let supports = support_rows
            .iter()
            .map(|row| {
                let numeric_id = parse_u32(&row.id);
                let price = parse_u32(&row.unlock_price);
                let reward_competition_id = parse_u32(&row.reward_competition_id);
                let acquisition = if price > 0 {
                    Acquisition::DiamondShop {
                        price_diamonds: Provenanced::new(price, Provenance::Asset, asset_src),
                    }
                } else if let Some((league, competition)) =
                    competition_positions.get(&reward_competition_id).copied()
                {
                    Acquisition::BattleReward {
                        league,
                        competition,
                        provenance: Provenance::Asset,
                        source: asset_src,
                    }
                } else {
                    Acquisition::LeagueReward {
                        after_league: 0,
                        provenance: Provenance::Asset,
                        source: asset_src,
                    }
                };
                let level_params = csv_u32(&row.levelup_params)
                    .into_iter()
                    .map(|value| Provenanced::new(value, Provenance::Asset, asset_src))
                    .collect::<Vec<_>>();
                SupportItemData {
                    id: support_slug(numeric_id),
                    name: leak_str(format!("{} / {}", row.name_memo, row.item_name_memo)),
                    unlock_league: 0,
                    cooldown_minutes: Provenanced::new(
                        parse_u32(&row.refresh_time).div_ceil(60),
                        Provenance::Asset,
                        asset_src,
                    ),
                    level_params,
                    upgrade_candy_costs: csv_u32(&row.levelup_prices)
                        .into_iter()
                        .map(|value| Provenanced::new(value, Provenance::Asset, asset_src))
                        .collect(),
                    acquisition,
                    skill: support_skill(numeric_id, &row),
                }
            })
            .collect::<Vec<_>>();

        let decors = decor_rows
            .iter()
            .map(|row| {
                let numeric_id = parse_u32(&row.id);
                let price = parse_u32(&row.price);
                let reward_competition_id = parse_u32(&row.reward_competition_id);
                let acquisition = if price > 0 {
                    Acquisition::DiamondShop {
                        price_diamonds: Provenanced::new(price, Provenance::Asset, asset_src),
                    }
                } else if let Some((league, competition)) =
                    competition_positions.get(&reward_competition_id).copied()
                {
                    Acquisition::BattleReward {
                        league,
                        competition,
                        provenance: Provenance::Asset,
                        source: asset_src,
                    }
                } else {
                    Acquisition::LeagueReward {
                        after_league: 0,
                        provenance: Provenance::Asset,
                        source: asset_src,
                    }
                };
                DecorItemData {
                    id: decor_slug(numeric_id),
                    name: leak_str(format!("{} / decor_{}", row.name_memo, row.id)),
                    acquisition,
                    effect: decor_effect(parse_u32(&row.bonus_type), parse_u32(&row.bonus_num)),
                }
            })
            .collect::<Vec<_>>();

        let leagues = league_rows
            .iter()
            .map(|league| {
                let league_id = parse_u32(&league.id);
                let competitions = competition_rows
                    .iter()
                    .filter(|competition| parse_u32(&competition.league_id) == league_id)
                    .enumerate()
                    .map(|(index, competition)| CompetitionData {
                        id: parse_u32(&competition.id),
                        name: if index + 1
                            == competition_rows
                                .iter()
                                .filter(|row| parse_u32(&row.league_id) == league_id)
                                .count()
                        {
                            "Champion"
                        } else {
                            "League Fight"
                        },
                        opponent_jump_cm: Provenanced::new(
                            parse_u64(&competition.enemy_estimated_kp),
                            Provenance::Asset,
                            asset_src,
                        ),
                        win_reward_coins: Provenanced::new(
                            parse_u64(&competition.victory_bonus_coin),
                            Provenance::Asset,
                            asset_src,
                        ),
                        loss_reward_coins: Provenanced::new(
                            parse_u64(&competition.loser_bonus),
                            Provenance::Asset,
                            asset_src,
                        ),
                        reward_diamonds: Provenanced::new(
                            parse_u32(&competition.reward_dia),
                            Provenance::Asset,
                            asset_src,
                        ),
                        reward_candy: Provenanced::new(
                            parse_u32(&competition.reward_candy),
                            Provenance::Asset,
                            asset_src,
                        ),
                        reward_support_id: parse_u32(&competition.reward_support_pokemon),
                        reward_decor_id: parse_u32(&competition.reward_deco_id),
                        breeder_exp_win: Provenanced::new(
                            parse_u128(&competition.breeder_exp_win),
                            Provenance::Asset,
                            asset_src,
                        ),
                    })
                    .collect();
                LeagueData {
                    id: league_id - 1,
                    name: leak_str(format!("{} / league_{}", league.name_memo, league.id)),
                    competitions,
                }
            })
            .collect::<Vec<_>>();

        let breeder_ranks = breeder_rows
            .iter()
            .map(|row| BreederRankData {
                rank: parse_u32(&row.rank),
                need_exp: Provenanced::new(parse_u128(&row.need_exp), Provenance::Asset, asset_src),
                magikarp_max_rank: Provenanced::new(
                    parse_u32(&row.magikarp_max_rank),
                    Provenance::Asset,
                    asset_src,
                ),
            })
            .collect::<Vec<_>>();
        let magikarp_ranks = magikarp_rows
            .iter()
            .map(|row| MagikarpRankData {
                rank: parse_u32(&row.rank),
                need_kp: Provenanced::new(parse_u128(&row.need_kp), Provenance::Asset, asset_src),
                retirement_breeder_exp: Provenanced::new(
                    parse_u128(&row.retirement_breeder_exp),
                    Provenance::Asset,
                    asset_src,
                ),
                level_up_coins: Provenanced::new(
                    parse_u64(&row.second_bonus_coin),
                    Provenance::Asset,
                    asset_src,
                ),
            })
            .collect::<Vec<_>>();
        let jump_curve = jump_rows
            .iter()
            .map(|row| JumpCurvePoint {
                need_kp: Provenanced::new(parse_u128(&row.need_kp), Provenance::Asset, asset_src),
                height: Provenanced::new(parse_u64(&row.height), Provenance::Asset, asset_src),
            })
            .collect::<Vec<_>>();

        let treasure_weight_total = treasure_rows
            .iter()
            .map(|row| parse_u32(&row.freq))
            .sum::<u32>()
            .max(1);
        let treasure_diamond = treasure_rows
            .iter()
            .find(|row| row.genre_id == "3")
            .map(|row| (parse_u32(&row.num), parse_u32(&row.freq)))
            .unwrap_or((0, 0));
        let treasure_rewards = treasure_rows
            .iter()
            .map(|row| TreasureRewardData {
                genre_id: parse_u32(&row.genre_id),
                freq: parse_u32(&row.freq),
                num: parse_u32(&row.num),
                memo: leak_str(row.memo.clone()),
                provenance: Provenance::Asset,
                source: asset_src,
            })
            .collect::<Vec<_>>();
        let random_events = random_event_rows
            .iter()
            .filter(|row| row.is_active == "TRUE")
            .filter(|row| parse_u32(&row.freq) > 0)
            .filter(|row| parse_u32(&row.need_command) == 0)
            .filter_map(|row| {
                let occurrence = match parse_u32(&row.occurrance_type) {
                    1 => RandomEventOccurrence::Home,
                    2 => RandomEventOccurrence::Training,
                    3 => RandomEventOccurrence::LeagueWin,
                    4 => RandomEventOccurrence::LeagueLoss,
                    _ => return None,
                };
                Some(RandomEventData {
                    id: parse_u32(&row.id),
                    name: leak_str(row.name_memo.clone()),
                    occurrence,
                    need_league_id: parse_u32(&row.need_league_id),
                    need_support_pokemon_id: parse_u32(&row.need_support_pokemon_id),
                    need_generation: parse_u32(&row.need_generation),
                    bonus_type: parse_u32(&row.bonus_type),
                    bonus_num: parse_u32(&row.bonus_num),
                    success_bonus_type: parse_u32(&row.success_bonus_type),
                    success_bonus_num: parse_u32(&row.success_bonus_num),
                    success_chance_permyriad: parse_u32(&row.success_per).saturating_mul(100),
                    penalty_type: parse_u32(&row.penalty_type),
                    penalty_num: parse_u32(&row.penalty_num),
                    freq: parse_u32(&row.freq),
                    provenance: Provenance::Asset,
                    source: asset_src,
                })
            })
            .collect::<Vec<_>>();
        let random_event_parameters = RandomEventParameters {
            training_chance_permyriad: Provenanced::new(
                parse_u32(&random_event_parameters_row.training_occurrence_per) * 100,
                Provenance::Asset,
                asset_src,
            ),
            training_max_per_day: Provenanced::new(
                parse_u32(&random_event_parameters_row.training_occurrence_max),
                Provenance::Asset,
                asset_src,
            ),
            league_win_chance_permyriad: Provenanced::new(
                parse_u32(&random_event_parameters_row.league_win_occurrence_per) * 100,
                Provenance::Asset,
                asset_src,
            ),
            league_win_max_per_day: Provenanced::new(
                parse_u32(&random_event_parameters_row.league_win_occurence_max),
                Provenance::Asset,
                asset_src,
            ),
            league_loss_chance_permyriad: Provenanced::new(
                parse_u32(&random_event_parameters_row.league_lose_occurrence_per) * 100,
                Provenance::Asset,
                asset_src,
            ),
            league_loss_max_per_day: Provenanced::new(
                parse_u32(&random_event_parameters_row.league_lose_occurence_max),
                Provenance::Asset,
                asset_src,
            ),
            home_chance_permyriad: Provenanced::new(
                parse_u32(&random_event_parameters_row.home_occurrence_per) * 100,
                Provenance::Asset,
                asset_src,
            ),
            home_cooldown_minutes: Provenanced::new(
                parse_u32(&random_event_parameters_row.home_occurrence_sec).div_ceil(60),
                Provenance::Asset,
                asset_src,
            ),
            home_max_cooldown_minutes: Provenanced::new(
                parse_u32(&random_event_parameters_row.home_occurrence_max_sec).div_ceil(60),
                Provenance::Asset,
                asset_src,
            ),
        };

        Self {
            name: "apk-masterdata-v1",
            sources: vec![asset_src, disasm_src, "GHIDRA_FINDINGS.md"],
            berries,
            trainings,
            supports,
            decors,
            leagues,
            economy: EconomyData {
                initial_diamonds: Provenanced::new(
                    parse_u32(&other.tutorial_clear_dia),
                    Provenance::Asset,
                    asset_src,
                ),
                trainer_rank_up_diamonds: Provenanced::new(
                    parse_u32(&other.breeder_rank_up_dia_num),
                    Provenance::Asset,
                    asset_src,
                ),
                home_treasure_cooldown_minutes: Provenanced::new(
                    parse_u32(&other.treasure_duration_sec).div_ceil(60),
                    Provenance::Asset,
                    asset_src,
                ),
                home_treasure_base_coins: Provenanced::new(
                    parse_u64(&other.first_coin_num),
                    Provenance::Asset,
                    asset_src,
                ),
                achievement_diamonds_minor: Provenanced::new(5, Provenance::Asset, asset_src),
                achievement_diamonds_small: Provenanced::new(25, Provenance::Asset, asset_src),
                achievement_diamonds_medium: Provenanced::new(50, Provenance::Asset, asset_src),
                league_battle_milestone_diamonds: Provenanced::new(
                    0,
                    Provenance::Asset,
                    "competition_list.json uses exact per-battle reward_dia instead of synthetic milestones",
                ),
                league_clear_support_candy: Provenanced::new(
                    0,
                    Provenance::Asset,
                    "competition_list.json uses exact per-battle reward_candy instead of synthetic league candy",
                ),
                pattern_discovery_diamonds: Provenanced::new(
                    parse_u32(&other.new_pattern_bonus_dia),
                    Provenance::Asset,
                    asset_src,
                ),
                random_event_diamonds: Provenanced::new(5, Provenance::Asset, asset_src),
                random_event_diamond_chance_permyriad: Provenanced::new(
                    parse_u32(&random_event_parameters_row.training_occurrence_per) * 100,
                    Provenance::Asset,
                    "legacy aggregate; simulator uses random_event_parameters.json plus random_event_list.json",
                ),
                sunken_treasure_diamonds: Provenanced::new(
                    treasure_diamond.0,
                    Provenance::Asset,
                    asset_src,
                ),
                sunken_treasure_diamond_chance_permyriad: Provenanced::new(
                    treasure_diamond.1 * 10_000 / treasure_weight_total,
                    Provenance::Asset,
                    asset_src,
                ),
                diamond_miner_diamonds: Provenanced::new(0, Provenance::Asset, asset_src),
                diamond_miner_cooldown_minutes: Provenanced::new(0, Provenance::Asset, asset_src),
                diamond_miner_f2p_enabled: Provenanced::new(false, Provenance::Asset, asset_src),
                food_respawn_minutes: Provenanced::new(
                    parse_u32(&other.home_food_sec).div_ceil(60).max(1),
                    Provenance::Asset,
                    "other_parameters.json home_food_sec rounded up to simulator minute ticks",
                ),
                food_respawn_seconds: Provenanced::new(
                    parse_u32(&other.home_food_sec).max(1),
                    Provenance::Asset,
                    "other_parameters.json home_food_sec exact value",
                ),
                home_food_max: Provenanced::new(
                    parse_u32(&other.home_food_max_num),
                    Provenance::Asset,
                    "other_parameters.json home_food_max_num exact value",
                ),
                manaphy_food_num: Provenanced::new(
                    parse_u32(&other.manaphy_fever_food_num),
                    Provenance::Asset,
                    "other_parameters.json manaphy_fever_food_num exact value",
                ),
                stamina_respawn_minutes: Provenanced::new(
                    30,
                    Provenance::Assumption,
                    "training point recovery interval not yet isolated in decoded master data",
                ),
            },
            random_events,
            random_event_parameters,
            treasure_rewards,
            food_upgrade_costs,
            training_upgrade_costs,
            breeder_ranks,
            magikarp_ranks,
            jump_curve,
        }
    }

    pub fn purchase_candidates(&self) -> Vec<PurchaseTarget> {
        self.supports
            .iter()
            .filter(|item| item.acquisition.shop_price().is_some())
            .map(|item| PurchaseTarget {
                kind: PurchaseKind::Support,
                id: item.id.to_string(),
            })
            .chain(
                self.decors
                    .iter()
                    .filter(|item| item.acquisition.shop_price().is_some())
                    .map(|item| PurchaseTarget {
                        kind: PurchaseKind::Decor,
                        id: item.id.to_string(),
                    }),
            )
            .collect()
    }

    pub fn preset_plan(&self, name_or_json: &str) -> PurchasePlan {
        if name_or_json.trim_start().starts_with('[') {
            if let Ok(ids) = serde_json::from_str::<Vec<String>>(name_or_json) {
                return PurchasePlan {
                    name: "json".to_string(),
                    targets: ids
                        .into_iter()
                        .filter_map(|id| self.target_by_id(&id))
                        .collect(),
                };
            }
        }

        let mut candidates = self.purchase_candidates();
        match name_or_json {
            "none" => candidates.clear(),
            "decor-first" => candidates.sort_by_key(|target| match target.kind {
                PurchaseKind::Decor => 0,
                PurchaseKind::Support => 1,
            }),
            "support-first" => candidates.sort_by_key(|target| match target.kind {
                PurchaseKind::Support => 0,
                PurchaseKind::Decor => 1,
            }),
            "balanced" | _ => candidates.sort_by_key(|target| self.purchase_price(target)),
        }

        PurchasePlan {
            name: name_or_json.to_string(),
            targets: candidates,
        }
    }

    pub fn target_by_id(&self, id: &str) -> Option<PurchaseTarget> {
        self.supports
            .iter()
            .any(|item| item.id == id && item.acquisition.shop_price().is_some())
            .then(|| PurchaseTarget {
                kind: PurchaseKind::Support,
                id: id.to_string(),
            })
            .or_else(|| {
                self.decors
                    .iter()
                    .any(|item| item.id == id && item.acquisition.shop_price().is_some())
                    .then(|| PurchaseTarget {
                        kind: PurchaseKind::Decor,
                        id: id.to_string(),
                    })
            })
    }

    pub fn purchase_price(&self, target: &PurchaseTarget) -> u32 {
        match target.kind {
            PurchaseKind::Support => self
                .supports
                .iter()
                .find(|item| item.id == target.id)
                .and_then(|item| item.acquisition.shop_price())
                .unwrap_or(u32::MAX),
            PurchaseKind::Decor => self
                .decors
                .iter()
                .find(|item| item.id == target.id)
                .and_then(|item| item.acquisition.shop_price())
                .unwrap_or(u32::MAX),
        }
    }

    pub fn berry_jp(&self, id: &str, rank: u32) -> Option<u128> {
        self.berries
            .iter()
            .find(|berry| berry.id == id)
            .and_then(|berry| interpolate_curve(&berry.jp_by_rank, rank))
    }

    pub fn training_jp(&self, id: &str, rank: u32) -> Option<u128> {
        self.trainings
            .iter()
            .find(|training| training.id == id)
            .and_then(|training| interpolate_curve(&training.jp_by_rank, rank))
    }

    pub fn unlocked_training_indices(&self, player_rank: u32) -> Vec<usize> {
        self.trainings
            .iter()
            .enumerate()
            .filter(|(_, training)| player_rank >= training.unlock_rank)
            .map(|(index, _)| index)
            .collect()
    }

    pub fn audit(&self) -> DataAuditReport {
        let mut total = 0;
        let mut exact = 0;
        let mut assumptions = 0;
        let mut wiki = 0;

        let mut count = |provenance: Provenance| {
            total += 1;
            match provenance {
                Provenance::Code | Provenance::Disassembly | Provenance::Asset => exact += 1,
                Provenance::Wiki => wiki += 1,
                Provenance::Assumption => assumptions += 1,
            }
        };

        for berry in &self.berries {
            count(berry.base_kp.provenance);
            for point in &berry.jp_by_rank {
                count(point.value.provenance);
            }
            count(berry.upgrade_cost_base.provenance);
        }
        for training in &self.trainings {
            for point in &training.jp_by_rank {
                count(point.value.provenance);
            }
        }
        for support in &self.supports {
            count(support.acquisition.provenance());
            count(support.cooldown_minutes.provenance);
            for cost in &support.upgrade_candy_costs {
                count(cost.provenance);
            }
        }
        for decor in &self.decors {
            count(decor.acquisition.provenance());
        }
        for league in &self.leagues {
            for competition in &league.competitions {
                count(competition.opponent_jump_cm.provenance);
                count(competition.win_reward_coins.provenance);
                count(competition.loss_reward_coins.provenance);
            }
        }
        for rank in &self.magikarp_ranks {
            count(rank.need_kp.provenance);
            count(rank.retirement_breeder_exp.provenance);
            count(rank.level_up_coins.provenance);
        }
        count(self.economy.initial_diamonds.provenance);
        count(self.economy.trainer_rank_up_diamonds.provenance);
        count(self.economy.home_treasure_cooldown_minutes.provenance);
        count(self.economy.home_treasure_base_coins.provenance);
        count(self.economy.achievement_diamonds_minor.provenance);
        count(self.economy.achievement_diamonds_small.provenance);
        count(self.economy.achievement_diamonds_medium.provenance);
        count(self.economy.league_battle_milestone_diamonds.provenance);
        count(self.economy.league_clear_support_candy.provenance);
        count(self.economy.pattern_discovery_diamonds.provenance);
        count(self.economy.random_event_diamonds.provenance);
        count(
            self.economy
                .random_event_diamond_chance_permyriad
                .provenance,
        );
        count(self.economy.sunken_treasure_diamonds.provenance);
        count(
            self.economy
                .sunken_treasure_diamond_chance_permyriad
                .provenance,
        );
        count(self.economy.diamond_miner_diamonds.provenance);
        count(self.economy.diamond_miner_cooldown_minutes.provenance);
        count(self.economy.diamond_miner_f2p_enabled.provenance);
        count(self.economy.food_respawn_minutes.provenance);
        count(self.economy.food_respawn_seconds.provenance);
        count(self.economy.home_food_max.provenance);
        count(self.economy.manaphy_food_num.provenance);
        count(self.economy.stamina_respawn_minutes.provenance);
        count(self.random_event_parameters.training_chance_permyriad.provenance);
        count(self.random_event_parameters.training_max_per_day.provenance);
        count(
            self.random_event_parameters
                .league_win_chance_permyriad
                .provenance,
        );
        count(self.random_event_parameters.league_win_max_per_day.provenance);
        count(
            self.random_event_parameters
                .league_loss_chance_permyriad
                .provenance,
        );
        count(self.random_event_parameters.league_loss_max_per_day.provenance);
        count(self.random_event_parameters.home_chance_permyriad.provenance);
        count(self.random_event_parameters.home_cooldown_minutes.provenance);
        count(
            self.random_event_parameters
                .home_max_cooldown_minutes
                .provenance,
        );
        for event in &self.random_events {
            count(event.provenance);
        }
        for treasure in &self.treasure_rewards {
            count(treasure.provenance);
        }

        let mut warnings = Vec::new();
        if assumptions > 0 {
            warnings.push(format!(
                "{assumptions} fields are approximate assumptions and need APK/master-data recovery"
            ));
        }
        if wiki > 0 {
            warnings.push(format!(
                "{wiki} fields use wiki provenance and should be validated against local code/assets"
            ));
        }
        if !self.random_events.is_empty() {
            warnings.push(
                "random event chances and weights come from APK assets; effect formulas are inferred from APK result fields plus wiki wording and still need native-function validation".to_string(),
            );
        }

        DataAuditReport {
            dataset: self.name,
            sources: self.sources.clone(),
            total_fields: total,
            exact_fields: exact,
            assumption_fields: assumptions,
            wiki_fields: wiki,
            warnings,
        }
    }
}

impl Acquisition {
    pub fn shop_price(&self) -> Option<u32> {
        match self {
            Acquisition::DiamondShop { price_diamonds } => Some(price_diamonds.value),
            Acquisition::LeagueReward { .. } | Acquisition::BattleReward { .. } => None,
        }
    }

    pub fn provenance(&self) -> Provenance {
        match self {
            Acquisition::DiamondShop { price_diamonds } => price_diamonds.provenance,
            Acquisition::LeagueReward { provenance, .. }
            | Acquisition::BattleReward { provenance, .. } => *provenance,
        }
    }

    pub fn league_reward_after(&self) -> Option<u32> {
        match self {
            Acquisition::LeagueReward { after_league, .. } => Some(*after_league),
            _ => None,
        }
    }

    pub fn battle_reward_at(&self) -> Option<(u32, u32)> {
        match self {
            Acquisition::BattleReward {
                league,
                competition,
                ..
            } => Some((*league, *competition)),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct FoodBaseRow {
    id: String,
    name_memo: String,
    unlock_breeder_rank: String,
}

#[derive(Debug, Deserialize)]
struct TrainingBaseRow {
    id: String,
    name_memo: String,
    unlock_breeder_rank: String,
}

#[derive(Debug, Deserialize)]
struct LevelTableRow {
    #[allow(dead_code)]
    level: String,
    #[serde(flatten)]
    values: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct SupportPokemonRow {
    id: String,
    name_memo: String,
    item_name_memo: String,
    unlock_price: String,
    reward_competition_id: String,
    levelup_params: String,
    levelup_prices: String,
    refresh_time: String,
}

#[derive(Debug, Deserialize)]
struct DecorationRow {
    id: String,
    name_memo: String,
    price: String,
    bonus_type: String,
    bonus_num: String,
    reward_competition_id: String,
}

#[derive(Debug, Deserialize)]
struct LeagueRow {
    id: String,
    name_memo: String,
}

#[derive(Debug, Deserialize)]
struct CompetitionRow {
    id: String,
    league_id: String,
    reward_dia: String,
    reward_support_pokemon: String,
    reward_candy: String,
    reward_deco_id: String,
    victory_bonus_coin: String,
    enemy_estimated_kp: String,
    loser_bonus: String,
    breeder_exp_win: String,
}

#[derive(Debug, Deserialize)]
struct BreederRankRow {
    rank: String,
    need_exp: String,
    magikarp_max_rank: String,
}

#[derive(Debug, Deserialize)]
struct MagikarpRankRow {
    rank: String,
    need_kp: String,
    retirement_breeder_exp: String,
    second_bonus_coin: String,
}

#[derive(Debug, Deserialize)]
struct JumpCurveRow {
    need_kp: String,
    height: String,
}

#[derive(Debug, Deserialize)]
struct OtherParametersRow {
    treasure_duration_sec: String,
    breeder_rank_up_dia_num: String,
    new_pattern_bonus_dia: String,
    first_coin_num: String,
    home_food_sec: String,
    home_food_max_num: String,
    manaphy_fever_food_num: String,
    tutorial_clear_dia: String,
}

#[derive(Debug, Deserialize)]
struct TreasureRow {
    freq: String,
    num: String,
    genre_id: String,
    memo: String,
}

#[derive(Debug, Deserialize)]
struct RandomEventRow {
    id: String,
    #[serde(rename = "name:memo")]
    name_memo: String,
    occurrance_type: String,
    need_league_id: String,
    need_support_pokemon_id: String,
    need_command: String,
    need_generation: String,
    bonus_type: String,
    bonus_num: String,
    success_bonus_type: String,
    success_bonus_num: String,
    success_per: String,
    penalty_type: String,
    penalty_num: String,
    freq: String,
    is_active: String,
}

#[derive(Debug, Deserialize)]
struct RandomEventParametersRow {
    training_occurrence_per: String,
    training_occurrence_max: String,
    league_win_occurrence_per: String,
    league_win_occurence_max: String,
    league_lose_occurrence_per: String,
    league_lose_occurence_max: String,
    home_occurrence_per: String,
    home_occurrence_sec: String,
    home_occurrence_max_sec: String,
}

fn json_rows<T: for<'de> Deserialize<'de>>(raw: &'static str) -> Vec<T> {
    serde_json::from_str(raw).expect("decoded APK master JSON should parse")
}

fn parse_u32(value: &str) -> u32 {
    value.parse().expect("APK numeric field should be u32")
}

fn parse_u64(value: &str) -> u64 {
    value.parse().expect("APK numeric field should be u64")
}

fn parse_u128(value: &str) -> u128 {
    value.parse().expect("APK numeric field should be u128")
}

fn csv_u32(value: &str) -> Vec<u32> {
    value
        .split(',')
        .filter_map(|part| {
            let trimmed = part.trim();
            (!trimmed.is_empty() && trimmed != "0").then(|| parse_u32(trimmed))
        })
        .collect()
}

fn leak_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

fn table_matrix(
    rows: &[LevelTableRow],
    prefix: &str,
    columns: usize,
    provenance: Provenance,
    source: &'static str,
) -> Vec<Vec<Provenanced<u64>>> {
    rows.iter()
        .map(|row| {
            (1..=columns)
                .map(|id| {
                    let key = format!("{prefix}{id}");
                    Provenanced::new(
                        row.values
                            .get(&key)
                            .map(|value| parse_u64(value))
                            .unwrap_or(0),
                        provenance,
                        source,
                    )
                })
                .collect()
        })
        .collect()
}

fn competition_position(rows: &[CompetitionRow], index: usize) -> (u32, u32) {
    let league_id = parse_u32(&rows[index].league_id);
    let competition = rows[..=index]
        .iter()
        .filter(|row| parse_u32(&row.league_id) == league_id)
        .count() as u32
        - 1;
    (league_id - 1, competition)
}

fn support_slug(id: u32) -> &'static str {
    match id {
        1 => "pikachu",
        2 => "piplup",
        3 => "snorlax",
        4 => "charizard",
        5 => "greninja",
        6 => "meowth",
        7 => "bulbasaur",
        8 => "slowpoke",
        9 => "mudkip",
        10 => "popplio",
        11 => "rowlet",
        12 => "litten",
        13 => "gengar",
        14 => "eevee",
        15 => "mimikyu",
        16 => "gardevoir",
        _ => "support_unknown",
    }
}

fn decor_slug(id: u32) -> &'static str {
    match id {
        1 => "octillery_pot",
        2 => "sudowoodo_bonsai",
        3 => "starmie_shower",
        5 => "exeggutor_palm",
        6 => "important_sign",
        8 => "lampent_lamp",
        9 => "parasect_puffballs",
        10 => "sunflora_bloom",
        11 => "dugtrio_rock",
        12 => "shaymin_planter",
        13 => "clefairy_doll",
        15 => "substitute_plush",
        16 => "whimsicott_cushion",
        17 => "lilligant_doll",
        18 => "bronze_eevee",
        20 => "ss_anne_model",
        22 => "aegislash_statue",
        24 => "cacnea_planter",
        25 => "red_cap",
        26 => "ditto_cushion",
        27 => "gold_magikarp_statue",
        _ => "decor_unknown",
    }
}

fn support_skill(id: u32, row: &SupportPokemonRow) -> SupportSkill {
    let first = csv_u32(&row.levelup_params).first().copied().unwrap_or(0);
    match id {
        1 | 5 | 12 | 15 => SupportSkill::KpFlat {
            base: first as u128,
        },
        2 => SupportSkill::Stamina {
            amount: first.max(1),
        },
        3 => SupportSkill::Food {
            amount: first.max(1),
        },
        4 | 9 => SupportSkill::Item {
            base_coin_value: first.max(1) as u64,
        },
        6 | 11 | 16 => SupportSkill::Coins {
            base: first.max(1) as u64,
        },
        7 => SupportSkill::LeaguePoint,
        8 => SupportSkill::RecoverSkills,
        10 => SupportSkill::Item { base_coin_value: 0 },
        13 => SupportSkill::KpBoost {
            multiplier_permyriad: 15_000,
        },
        14 => SupportSkill::TrainingGreat,
        _ => SupportSkill::Item { base_coin_value: 0 },
    }
}

fn decor_effect(bonus_type: u32, bonus_num: u32) -> DecorEffect {
    let mult = 10_000 + bonus_num * 100;
    match bonus_type {
        2 => DecorEffect::CoinPermyriad(mult),
        3 => DecorEffect::EventKpPermyriad(mult),
        4 => DecorEffect::EventCoinPermyriad(mult),
        5 => DecorEffect::LeagueCoinPermyriad(mult),
        6 => DecorEffect::FoodKpPermyriad(mult),
        7 => DecorEffect::TrainingPermyriad(mult),
        10 => DecorEffect::SkillKpPermyriad(mult),
        11 => DecorEffect::SkillRecoveryPermyriad(mult),
        12 => DecorEffect::TreasureCoinPermyriad(mult),
        13 => DecorEffect::TrainerExpPermyriad(mult),
        14 => DecorEffect::FoodCapacity(bonus_num),
        15 => DecorEffect::TrainingEventPermyriad(mult),
        16 => DecorEffect::LeagueEventPermyriad(mult),
        17 => DecorEffect::LevelUpCoinPermyriad(mult),
        _ => DecorEffect::Unknown,
    }
}

fn support_candy_costs(costs: &[u32]) -> Vec<Provenanced<u32>> {
    costs
        .iter()
        .copied()
        .map(|cost| {
            Provenanced::new(
                cost,
                Provenance::Wiki,
                "Magikarp Jump Wiki friendship item support candy costs",
            )
        })
        .collect()
}

fn curve_points(
    values: &[(u32, u128)],
    provenance: Provenance,
    source: &'static str,
) -> Vec<CurvePoint> {
    values
        .iter()
        .copied()
        .map(|(rank, value)| CurvePoint {
            rank,
            value: Provenanced::new(value, provenance, source),
        })
        .collect()
}

fn interpolate_curve(points: &[CurvePoint], rank: u32) -> Option<u128> {
    let first = points.first()?;
    if rank <= first.rank {
        return Some(first.value.value);
    }

    for pair in points.windows(2) {
        let a = &pair[0];
        let b = &pair[1];
        if rank == a.rank {
            return Some(a.value.value);
        }
        if rank <= b.rank {
            let span = b.rank.saturating_sub(a.rank) as u128;
            if span == 0 {
                return Some(b.value.value);
            }
            let offset = rank.saturating_sub(a.rank) as u128;
            let av = a.value.value;
            let bv = b.value.value;
            return Some(if bv >= av {
                av + (bv - av) * offset / span
            } else {
                av - (av - bv) * offset / span
            });
        }
    }

    points.last().map(|point| point.value.value)
}

fn wiki_leagues(wiki_src: &'static str, assumption_src: &'static str) -> Vec<LeagueData> {
    vec![
        league_from_values(
            0,
            "Friend League",
            &[10, 387, 663, 939, 1_408],
            Provenance::Wiki,
            wiki_src,
        ),
        league_from_points(
            1,
            "Quick League",
            10,
            &[(1, 1_836), (10, 32_357)],
            Provenance::Assumption,
            assumption_src,
        ),
        league_from_points(
            2,
            "Heavy League",
            15,
            &[(1, 45_215), (15, 338_518)],
            Provenance::Assumption,
            assumption_src,
        ),
        league_from_points(
            3,
            "Great League",
            15,
            &[(1, 423_147), (15, 1_404_121)],
            Provenance::Assumption,
            assumption_src,
        ),
        league_from_points(
            4,
            "Fast League",
            15,
            &[(1, 1_513_526), (15, 5_961_483)],
            Provenance::Assumption,
            assumption_src,
        ),
        league_from_points(
            5,
            "Luxury League",
            20,
            &[(1, 6_715_534), (20, 35_136_554)],
            Provenance::Assumption,
            assumption_src,
        ),
        league_from_points(
            6,
            "Heal League",
            20,
            &[(1, 39_080_537), (20, 213_229_220)],
            Provenance::Assumption,
            assumption_src,
        ),
        league_from_points(
            7,
            "Ultra League",
            20,
            &[(1, 256_453_723), (20, 1_064_369_552)],
            Provenance::Assumption,
            assumption_src,
        ),
        league_from_points(
            8,
            "Elite Four League",
            20,
            &[(1, 1_219_314_925), (20, 1_700_000_000)],
            Provenance::Assumption,
            assumption_src,
        ),
        league_from_points(
            9,
            "Master League",
            15,
            &[
                (1, 2_006_467_625),
                (5, 12_038_805_760),
                (10, 16_740_451_833),
                (15, 24_355_293_657),
            ],
            Provenance::Wiki,
            wiki_src,
        ),
    ]
}

fn league_from_values(
    id: u32,
    name: &'static str,
    values: &[u128],
    provenance: Provenance,
    source: &'static str,
) -> LeagueData {
    let competitions = values
        .iter()
        .copied()
        .enumerate()
        .map(|(index, opponent)| {
            competition_from_opponent(index as u32, values.len(), opponent, provenance, source)
        })
        .collect();
    LeagueData {
        id,
        name,
        competitions,
    }
}

fn league_from_points(
    id: u32,
    name: &'static str,
    count: u32,
    anchors: &[(u32, u128)],
    provenance: Provenance,
    source: &'static str,
) -> LeagueData {
    let points = curve_points(anchors, provenance, source);
    let competitions = (0..count)
        .map(|index| {
            let opponent = interpolate_curve(&points, index + 1).unwrap_or(0);
            competition_from_opponent(index, count as usize, opponent, provenance, source)
        })
        .collect();
    LeagueData {
        id,
        name,
        competitions,
    }
}

fn competition_from_opponent(
    index: u32,
    count: usize,
    opponent: u128,
    provenance: Provenance,
    source: &'static str,
) -> CompetitionData {
    let reward = ((opponent / 250).max(10).min(u64::MAX as u128)) as u64;
    CompetitionData {
        id: index,
        name: if index as usize + 1 == count {
            "Champion"
        } else {
            "League Fight"
        },
        opponent_jump_cm: Provenanced::new(
            opponent.min(u64::MAX as u128) as u64,
            provenance,
            source,
        ),
        win_reward_coins: Provenanced::new(reward, Provenance::Assumption, source),
        loss_reward_coins: Provenanced::new(reward * 2 / 5, Provenance::Assumption, source),
        reward_diamonds: Provenanced::new(0, Provenance::Assumption, source),
        reward_candy: Provenanced::new(0, Provenance::Assumption, source),
        reward_support_id: 0,
        reward_decor_id: 0,
        breeder_exp_win: Provenanced::new(1, Provenance::Assumption, source),
    }
}
