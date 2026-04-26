use serde::Serialize;

use crate::data::GameData;
use crate::model::Provenance;
use crate::rules::Rules;

#[derive(Clone, Debug, Serialize)]
pub struct CurveAuditReport {
    pub dataset: &'static str,
    pub findings: Vec<CurveFinding>,
    pub summary: CurveAuditSummary,
    pub sources: Vec<&'static str>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CurveAuditSummary {
    pub total: usize,
    pub critical: usize,
    pub warn: usize,
    pub ok: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct CurveFinding {
    pub category: &'static str,
    pub item: String,
    pub sim_value: f64,
    pub wiki_value: f64,
    pub ratio: f64,
    pub severity: CurveSeverity,
    pub provenance: Provenance,
    pub note: String,
    pub source: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum CurveSeverity {
    Ok,
    Warn,
    Critical,
}

pub fn audit_curves<R: Rules>(rules: &R, data: &GameData) -> CurveAuditReport {
    let mut findings = Vec::new();
    audit_rank_level_curve(rules, &mut findings);
    audit_food_curve(data, &mut findings);
    audit_training_curve(data, &mut findings);
    audit_league_curve(data, &mut findings);
    audit_structure(data, &mut findings);

    let summary = CurveAuditSummary {
        total: findings.len(),
        critical: findings
            .iter()
            .filter(|finding| finding.severity == CurveSeverity::Critical)
            .count(),
        warn: findings
            .iter()
            .filter(|finding| finding.severity == CurveSeverity::Warn)
            .count(),
        ok: findings
            .iter()
            .filter(|finding| finding.severity == CurveSeverity::Ok)
            .count(),
    };

    CurveAuditReport {
        dataset: data.name,
        findings,
        summary,
        sources: vec![
            "https://bulbapedia.bulbagarden.net/wiki/Jump%21_Magikarp",
            "https://magikarpjump.fandom.com/wiki/Food",
            "https://magikarpjump.fandom.com/wiki/Sitrus_Berry",
            "https://magikarpjump.fandom.com/wiki/Friend_League",
            "https://magikarpjump.fandom.com/wiki/Master_League",
            "https://magikarpjump.fandom.com/wiki/Trainer_Rank",
        ],
    }
}

fn audit_rank_level_curve<R: Rules>(rules: &R, findings: &mut Vec<CurveFinding>) {
    for (rank, wiki) in [(1, 11), (15, 25), (90, 100), (100, 100)] {
        push_ratio(
            findings,
            "trainer-rank",
            format!("max level at trainer rank {rank}"),
            rules.max_level_for_rank(rank) as f64,
            wiki as f64,
            Provenance::Wiki,
            "A newly fished Magikarp should have max level trainer rank + 10, capped at 100.",
            "https://magikarpjump.fandom.com/wiki/Trainer_Rank",
        );
    }
}

fn audit_food_curve(data: &GameData, findings: &mut Vec<CurveFinding>) {
    for (id, apk_id, level, wiki) in [
        ("oran", "food_1", 1, 2.0),
        ("oran", "food_1", 25, 81.0),
        ("oran", "food_1", 50, 2_550.0),
        ("oran", "food_1", 75, 76_300.0),
        ("oran", "food_1", 100, 3_189_075.0),
        ("sitrus", "food_2", 1, 9.0),
        ("sitrus", "food_2", 25, 163.0),
        ("sitrus", "food_2", 50, 3_367.0),
        ("sitrus", "food_2", 75, 90_340.0),
        ("sitrus", "food_2", 100, 3_724_529.0),
    ] {
        let sim = data
            .berry_jp(id, level)
            .or_else(|| data.berry_jp(apk_id, level))
            .map(|value| value as f64)
            .unwrap_or(0.0);
        push_ratio(
            findings,
            "food-jp",
            format!("{id} rank {level}"),
            sim,
            wiki,
            Provenance::Wiki,
            "Food JP should come from the wiki/APK rank table, not the old linear placeholder.",
            "https://bulbapedia.bulbagarden.net/wiki/Jump%21_Magikarp",
        );
    }
}

fn audit_training_curve(data: &GameData, findings: &mut Vec<CurveFinding>) {
    for (id, apk_id, name, wiki) in [
        ("sandbag_slam", "training_1", "Sandbag Slam rank 1", 35.0),
        ("jump_counter", "training_2", "Jump Counter rank 1", 122.0),
    ] {
        let sim = data
            .training_jp(id, 1)
            .or_else(|| data.training_jp(apk_id, 1))
            .map(|value| value as f64)
            .unwrap_or(0.0);
        push_ratio(
            findings,
            "training-jp",
            name.to_string(),
            sim,
            wiki,
            Provenance::Wiki,
            "Training JP should be table-driven per training regimen.",
            "https://bulbapedia.bulbagarden.net/wiki/Jump%21_Magikarp",
        );
    }
}

fn audit_league_curve(data: &GameData, findings: &mut Vec<CurveFinding>) {
    for (competition, wiki) in [(0, 10.0), (1, 387.0), (2, 663.0), (3, 939.0), (4, 1_408.0)] {
        let sim = data
            .leagues
            .first()
            .and_then(|league| league.competitions.get(competition))
            .map(|competition| competition.opponent_jump_cm.value as f64)
            .unwrap_or(0.0);
        push_ratio(
            findings,
            "league-opponent",
            format!("Friend League battle {}", competition + 1),
            sim,
            wiki,
            Provenance::Wiki,
            "League opponents should use wiki/APK opponent JP tables.",
            "https://magikarpjump.fandom.com/wiki/Friend_League",
        );
    }

    for (competition, wiki) in [
        (0, 2_006_467_625.0),
        (4, 12_038_805_760.0),
        (9, 16_740_451_833.0),
        (14, 24_355_293_657.0),
    ] {
        let sim = data
            .leagues
            .get(9)
            .and_then(|league| league.competitions.get(competition))
            .map(|competition| competition.opponent_jump_cm.value as f64)
            .unwrap_or(0.0);
        push_ratio(
            findings,
            "league-opponent",
            format!("Master League battle {}", competition + 1),
            sim,
            wiki,
            Provenance::Wiki,
            "Master League should be the 10th league with 15 battles and wiki opponent JP anchors.",
            "https://magikarpjump.fandom.com/wiki/Master_League",
        );
    }
}

fn audit_structure(data: &GameData, findings: &mut Vec<CurveFinding>) {
    push_ratio(
        findings,
        "league-structure",
        "league count".to_string(),
        data.leagues.len() as f64,
        10.0,
        Provenance::Wiki,
        "Story completion target should include 10 leagues ending in Master League.",
        "https://magikarpjump.fandom.com/wiki/Master_League",
    );
    let master_battles = data
        .leagues
        .get(9)
        .map(|league| league.competitions.len())
        .unwrap_or(0);
    push_ratio(
        findings,
        "league-structure",
        "Master League battle count".to_string(),
        master_battles as f64,
        15.0,
        Provenance::Wiki,
        "Master League has 15 battles and reward checkpoints at battles 5, 10, and 15.",
        "https://magikarpjump.fandom.com/wiki/Master_League",
    );
}

fn push_ratio(
    findings: &mut Vec<CurveFinding>,
    category: &'static str,
    item: String,
    sim_value: f64,
    wiki_value: f64,
    provenance: Provenance,
    note: &'static str,
    source: &'static str,
) {
    let ratio = if wiki_value == 0.0 {
        0.0
    } else {
        sim_value / wiki_value
    };
    let severity = classify_ratio(ratio);
    findings.push(CurveFinding {
        category,
        item,
        sim_value,
        wiki_value,
        ratio,
        severity,
        provenance,
        note: note.to_string(),
        source,
    });
}

fn classify_ratio(ratio: f64) -> CurveSeverity {
    if (0.8..=1.25).contains(&ratio) {
        CurveSeverity::Ok
    } else if (0.5..=2.0).contains(&ratio) {
        CurveSeverity::Warn
    } else {
        CurveSeverity::Critical
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::ApproxRules;

    #[test]
    fn curve_audit_accepts_wiki_anchor_curves() {
        let report = audit_curves(&ApproxRules, &GameData::approx_v1());
        assert_eq!(report.summary.critical, 0);
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.item == "Master League battle count"
                    && finding.severity == CurveSeverity::Ok)
        );
    }
}
