use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum};
use karpador_sim::{
    ApkRules, ApproxRules, EarlyCompeteStrategy, GameData, GreedyKpStrategy, OptimizerConfig,
    ShopRoiStrategy, SimConfig, Simulator, WallSimConfig, WallTimeSimulator, audit_curves,
    optimize_purchase_plans,
};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum StrategyKind {
    GreedyKp,
    EarlyCompete,
    ShopRoi,
}

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Monte-Carlo simulator for Karpador Jump tactics"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[command(flatten)]
    legacy: LegacyArgs,
}

#[derive(Debug, Subcommand)]
enum Command {
    Run(RunArgs),
    Optimize(OptimizeArgs),
    Data {
        #[command(subcommand)]
        command: DataCommand,
    },
    Legacy(LegacyArgs),
}

#[derive(Debug, Subcommand)]
enum DataCommand {
    Audit {
        #[arg(long)]
        json: bool,
    },
    CurveAudit {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, ClapArgs, Clone)]
struct LegacyArgs {
    #[arg(long, value_enum, default_value_t = StrategyKind::GreedyKp)]
    strategy: StrategyKind,

    #[arg(long, default_value_t = 1_000)]
    runs: u32,

    #[arg(long, default_value_t = 42)]
    seed: u64,

    #[arg(long, default_value_t = 1_000)]
    max_actions: u32,

    #[arg(long, default_value_t = 3)]
    target_league: u32,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, ClapArgs)]
struct RunArgs {
    #[arg(long, default_value = "balanced")]
    plan: String,

    #[arg(long, default_value_t = 42)]
    seed: u64,

    #[arg(long, default_value_t = 240)]
    max_days: u32,

    #[arg(long, default_value_t = 100_000)]
    max_actions: u32,

    #[arg(long, default_value_t = 10)]
    sessions_per_day: u8,

    #[arg(long, default_value_t = 0)]
    training_upgrade_share: u32,

    #[arg(long, default_value = "master-league")]
    target: String,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, ClapArgs)]
struct OptimizeArgs {
    #[arg(long, default_value_t = 100)]
    runs: u32,

    #[arg(long, default_value_t = 42)]
    seed: u64,

    #[arg(long, default_value_t = 10)]
    beam_width: usize,

    #[arg(long, default_value = "master-league")]
    target: String,

    #[arg(long, default_value_t = 240)]
    max_days: u32,

    #[arg(long, default_value_t = 10)]
    sessions_per_day: u8,

    #[arg(long, default_value_t = 0)]
    training_upgrade_share: u32,

    #[arg(long)]
    json: bool,
}

fn main() -> anyhow_free::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Run(args)) => run_walltime(args),
        Some(Command::Optimize(args)) => optimize(args),
        Some(Command::Data {
            command: DataCommand::Audit { json },
        }) => data_audit(json),
        Some(Command::Data {
            command: DataCommand::CurveAudit { json },
        }) => curve_audit(json),
        Some(Command::Legacy(args)) => run_legacy(args),
        None => run_legacy(cli.legacy),
    }
}

fn run_walltime(args: RunArgs) -> anyhow_free::Result<()> {
    let data = GameData::apk_master();
    let rules = ApkRules::new(&data);
    let plan = data.preset_plan(&args.plan);
    let sim = WallTimeSimulator::new(
        rules,
        data,
        WallSimConfig {
            max_actions: args.max_actions,
            max_wall_days: args.max_days,
            max_sessions_per_day: args.sessions_per_day,
            target_league: target_league(&args.target),
            ..WallSimConfig::default()
        },
    );
    let result = if args.training_upgrade_share > 0 {
        let mut policy = karpador_sim::ActivePlayerPolicy::with_purchase_plan_and_training_share(
            plan,
            args.training_upgrade_share.min(10_000),
        );
        sim.run_with_policy(args.seed, &mut policy)
    } else {
        sim.run(args.seed, plan)
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("plan:          {}", result.plan);
        println!("dataset:       {}", result.dataset);
        println!("outcome:       {:?}", result.outcome);
        println!("wall days:     {:.2}", result.wall_days);
        println!("sessions:      {}", result.sessions);
        println!("league:        {}", result.final_state.league);
        println!("rank:          {}", result.final_state.player_rank);
        println!("diamonds left: {}", result.final_state.diamonds);
        println!("purchases:     {}", result.purchases.len());
        for purchase in &result.purchases {
            println!(
                "  day {:>6.2}: {:?} {} ({} diamonds)",
                purchase.minute as f64 / 1_440.0,
                purchase.kind,
                purchase.id,
                purchase.price_diamonds
            );
        }
        for warning in &result.warnings {
            println!("warning:       {warning}");
        }
    }
    Ok(())
}

fn optimize(args: OptimizeArgs) -> anyhow_free::Result<()> {
    let data = GameData::apk_master();
    let rules = ApkRules::new(&data);
    let report = optimize_purchase_plans(
        rules,
        data,
        WallSimConfig {
            max_wall_days: args.max_days,
            max_sessions_per_day: args.sessions_per_day,
            target_league: target_league(&args.target),
            ..WallSimConfig::default()
        },
        OptimizerConfig {
            runs: args.runs,
            seed: args.seed,
            beam_width: args.beam_width,
            training_upgrade_share: args.training_upgrade_share.min(10_000),
        },
    );

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("dataset:         {}", report.dataset);
        println!("runs per plan:   {}", report.runs_per_plan);
        println!("evaluated plans: {}", report.evaluated_plans);
        for score in &report.ranked_plans {
            println!(
                "#{:<2} median {:>6.2}d mean {:>6.2}d success {:>5.1}% rank {:>5.1}/{:>5.1} progress {:>6.1} spent {:>6.1} first {:?} plan {}",
                score.rank,
                score.median_days,
                score.mean_days,
                score.success_rate * 100.0,
                score.median_player_rank,
                score.mean_player_rank,
                score.mean_league_progress,
                score.mean_diamonds_spent,
                score
                    .mean_first_purchase_day
                    .map(|day| format!("{day:.2}d")),
                score.plan
            );
        }
        for warning in &report.warnings {
            println!("warning:         {warning}");
        }
    }
    Ok(())
}

fn data_audit(json: bool) -> anyhow_free::Result<()> {
    let audit = GameData::apk_master().audit();
    if json {
        println!("{}", serde_json::to_string_pretty(&audit)?);
    } else {
        println!("dataset:     {}", audit.dataset);
        println!("fields:      {}", audit.total_fields);
        println!("exact:       {}", audit.exact_fields);
        println!("wiki:        {}", audit.wiki_fields);
        println!("assumption:  {}", audit.assumption_fields);
        for source in &audit.sources {
            println!("source:      {source}");
        }
        for warning in &audit.warnings {
            println!("warning:     {warning}");
        }
    }
    Ok(())
}

fn curve_audit(json: bool) -> anyhow_free::Result<()> {
    let data = GameData::apk_master();
    let rules = ApkRules::new(&data);
    let report = audit_curves(&rules, &data);
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("dataset:  {}", report.dataset);
        println!(
            "summary:  {} total, {} critical, {} warn, {} ok",
            report.summary.total, report.summary.critical, report.summary.warn, report.summary.ok
        );
        for finding in &report.findings {
            println!(
                "{:?} {:<17} {:<32} sim={:<14.3} wiki={:<14.3} ratio={:<10.6}",
                finding.severity,
                finding.category,
                finding.item,
                finding.sim_value,
                finding.wiki_value,
                finding.ratio
            );
            if finding.severity != karpador_sim::CurveSeverity::Ok {
                println!("  note:   {}", finding.note);
                println!("  source: {}", finding.source);
            }
        }
    }
    Ok(())
}

fn run_legacy(args: LegacyArgs) -> anyhow_free::Result<()> {
    let sim = Simulator::new(
        ApproxRules,
        SimConfig {
            max_actions: args.max_actions,
            target_league: args.target_league,
        },
    );

    let report = match args.strategy {
        StrategyKind::GreedyKp => sim.experiment(args.runs, args.seed, || GreedyKpStrategy),
        StrategyKind::EarlyCompete => {
            sim.experiment(args.runs, args.seed, EarlyCompeteStrategy::default)
        }
        StrategyKind::ShopRoi => sim.experiment(args.runs, args.seed, || ShopRoiStrategy),
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("strategy:       {} (legacy approx-v0)", report.strategy);
        println!("rules:          {}", report.rules);
        println!("runs:           {}", report.runs);
        println!("target league:  {}", report.target_league);
        println!("success rate:   {:.1}%", report.success_rate * 100.0);
        println!("avg actions:    {:.1}", report.avg_actions);
        println!("avg minutes:    {:.1}", report.avg_minutes);
        println!("avg generation: {:.1}", report.avg_generation);
        println!("avg rank:       {:.1}", report.avg_rank);
        println!("avg coins:      {:.1}", report.avg_coins);
        println!("avg kp:         {:.1}", report.avg_kp);
        println!("note:           use `run`, `optimize`, or `data audit` for wall-time v1");
    }

    Ok(())
}

fn target_league(target: &str) -> u32 {
    match target {
        "master-league" | "master" => 10,
        _ => 10,
    }
}

mod anyhow_free {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
}
