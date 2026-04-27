# Karpador simulator

Rust Monte-Carlo simulator for testing Karpador Jump tactics.

The simulator now has two layers:

- `apk-masterdata-v1`: session-based engine with APK master data, wall-clock time, diamond purchase plans, support/decor cooldowns, per-league planned losses, and data provenance.
- `approx-v0`: legacy action simulator kept for quick strategy comparisons while APK-exact formulas and master data are recovered.

`apk-masterdata-v1` is deliberately explicit about exactness. `data audit` reports which fields are recovered from decoded APK assets and which are still assumptions.

## Run

```sh
cargo run -- run --plan balanced --seed 42 --max-days 240
```

Start from a saved/provided game state and policy direction:

```sh
cargo run -- run --start-config examples/start_config.json --seed 42 --max-days 240
```

JSON output:

```sh
cargo run -- run --plan '["exeggutor_palm","pikachu","sudowoodo"]' --seed 42 --json
```

Optimize support/decor purchase order:

```sh
cargo run -- optimize --runs 100 --beam-width 10 --target master-league
```

Audit recovered data exactness:

```sh
cargo run -- data audit
```

Legacy strategies:

```sh
cargo run -- legacy --strategy shop-roi --runs 1000 --seed 42 --max-actions 500 --target-league 2
```

- `greedy-kp`: feed all available food, train when stamina is available, compete only at max level.
- `early-compete`: compete when estimated jump height clears the next opponent with a safety margin.
- `shop-roi`: buy the currently better food/training upgrade by approximate KP-per-coin, then play like an early-compete strategy.

## Wall-time policy

The v1 policy models the requested F2P player:

- Up to 10 useful sessions per day between 08:00 and 20:00.
- Sessions are evenly spaced across the active window (for 10/day: 08:00, 09:20, ..., 20:00).
- A session ends only when no policy action remains.
- Gold/HomeTreasure and achievement claims are collected immediately.
- Ready support skills are used before spending/upgrades.
- Automatically awarded Friendship Items and decorations are simulated as rewards, but are not purchase candidates.
- Pokédrops/Support Candies are simulated as `candy` and are auto-spent on owned helper upgrades using a fixed cheapest-next-upgrade policy.
- Diamonds are spent strictly in the active `PurchasePlan`, only on Diamond Shop Support/Friendship Items and Diamond Shop decorations.
- Sinelbeere/Oran and Tsitrusbeere/Sitrus are upgraded evenly.
- Training happens before berry eating when stamina is available.
- At least 3 berries are eaten before league attempts.
- Each league gets exactly one intentional loss, at the last fight/Champion. After that loss, the current Magikarp is brought to max level before forced league progress resumes.

## Start config

`run --start-config <file.json>` accepts a JSON file with two optional blocks:

- `start_state`: patches the initial wall-time state, including `player_rank`, `gold`/`coins`, `diamonds`, `league`, `competition`, inventory items, owned supports/decors, berry levels, training levels, and current Magikarp level/KP.
- `policy`: adjusts the active player policy, including `purchase_plan`, item usage flags, support upgrades, allowed berry/training upgrades, `training_upgrade_share`, and `karpador_loss_risk_max_level_percent`.

IDs are validated against the loaded master data. Invalid support, decor, berry, training, or JSON purchase-plan IDs abort the run with a clear error. League and competition values use the simulator's zero-based indices, matching `final_state.league` and `final_state.competition` in JSON output.

## Structure

- `src/data.rs`: approximate master data, provenance, and audit reporting.
- `decoded_master_data/`: decoded APK master tables embedded at compile time.
- `src/model.rs`: state and action types.
- `src/walltime.rs`: session loop, player policy, support/decor effects, league loss behavior.
- `src/optimizer.rs`: Monte-Carlo purchase-order evaluation.
- `src/rules.rs`: rule/formula trait and the current approximate rules.
- `src/strategy.rs`: tactic policies.
- `src/sim.rs`: legacy action simulation loop and aggregate report.
- `src/main.rs`: CLI.

## Next exactness targets

Replace approximate fields with recovered formulas/data from the APK:

- `MagicarpData::calculateJumpHeight`
- `MagicarpData::getNeedPower`
- `TrainingManager::calcResultData`
- `CompetitionManager::calculateResultJumpHeight`
- `BonusManager::calcKpNum`
- `SupportPokemon::getPrice`, `SupportPokemon::getRefreshTime`, skill effects
- `Deco::getPrice` and decoration effects
- `HomeTreasure::calcCoinNum`
- exact Pokédrop/Support Candy reward schedule and upgrade costs
- food/training/league/support/decor/achievement master tables from encrypted assets
