import {
  useEffect,
  useMemo,
  useState,
  ChangeEvent,
  useRef,
} from 'react';
import { loadRuntimeApi, type RuntimeApi, type RuntimeResult } from './runtime';

type Language = 'de' | 'en' | 'ja';

interface CatalogItem {
  id: string;
  name: string;
  name_memo?: string;
  item_name?: string;
  item_name_memo?: string;
  unlock_price?: string;
  reward_competition_id?: string;
  price?: string;
}

interface LocalizedText {
  de: string;
  en: string;
  ja: string;
}

interface CompetitionRow {
  id: string;
  league_id: string;
}

interface RankRow {
  rank: string;
}

interface OptionData {
  supports: CatalogItem[];
  decors: CatalogItem[];
  berries: CatalogItem[];
  trainings: CatalogItem[];
  leagues: CatalogLeague[];
  competitionPositions: Record<string, { league: number; competition: number }>;
  leagueCompetitionCounts: Record<number, number>;
  maxPlayerRank: number;
  maxMagikarpRank: number;
  maxBerryLevel: number;
  maxTrainingLevel: number;
}

interface LeagueListRow {
  id: string;
  name_memo: string;
}

interface CatalogLeague {
  id: string;
  name_memo: string;
}

interface PurchasePlanOption {
  id: string;
  label: string;
  cost: number;
}

interface StartState {
  player_rank: number;
  gold: number;
  diamonds: number;
  league: number;
  competition: number;
  generation: number;
  retirements: number;
  magikarp_level: number;
  magikarp_kp: number;
  candy: number;
  training_sodas: number;
  skill_herbs: number;
  league_aids: number;
  owned_supports: string[];
  owned_decors: string[];
  berry_levels: Record<string, number>;
  training_levels: Record<string, number>;
  berry_enabled: Record<string, boolean>;
  training_enabled: Record<string, boolean>;
}

interface PolicyState {
  purchase_plan: string;
  allow_training_sodas: boolean;
  allow_skill_herbs: boolean;
  allow_support_upgrades: boolean;
  training_upgrade_share: number;
  allowed_berry_upgrades: string[];
  allowed_training_upgrades: string[];
  karpador_loss_risk_max_level_percent: number;
  sessions_per_day: number;
}

interface FormState {
  start_state: StartState;
  policy: PolicyState;
}

const UI_TEXT: Record<
  | 'appTitle'
  | 'languageLabel'
  | 'headerPath'
  | 'headerNote'
  | 'startStateSection'
  | 'policySection'
  | 'startStatePlayerRank'
  | 'startStateGold'
  | 'startStateDiamonds'
  | 'startStateLeague'
  | 'startStateCompetition'
  | 'startStateGeneration'
  | 'startStateRetirements'
  | 'startStateMagikarpLevel'
  | 'startStateMagikarpKp'
  | 'startStateCandy'
  | 'startStateTrainingSodas'
  | 'startStateSkillHerbs'
  | 'startStateLeagueAids'
  | 'startStateOwnedSupports'
  | 'startStateOwnedDecors'
  | 'multiSelectHint'
  | 'berryLevelsTitle'
  | 'berryLevelLabel'
  | 'trainingLevelsTitle'
  | 'trainingLevelLabel'
  | 'policyPurchasePlan'
  | 'policyCustomPlan'
  | 'policyCustomPlanHint'
  | 'currencyDiamonds'
  | 'policyAllowTrainingSodas'
  | 'policyAllowSkillHerbs'
  | 'policyAllowSupportUpgrades'
  | 'policyTrainingUpgradeShare'
  | 'policyAllowedBerryUpgrades'
  | 'policyAllowedTrainingUpgrades'
  | 'policyKarpadorLossRisk'
  | 'policySessionsPerDay'
  | 'resultJsonTitle'
  | 'copyToClipboard'
  | 'copiedToClipboard'
  | 'downloadConfig'
  | 'loading'
  | 'customPlanOption'
  | 'numberValueCorrected'
  | 'numberPleaseEnterInteger'
  | 'numberInvalidNumber'
  | 'numberMustBeInteger'
  | 'numberOutOfRange'
  | 'runtimeSection'
  | 'runtimeRun'
  | 'runtimeLoading'
  | 'runtimeReady'
  | 'runtimeUnavailable'
  | 'runtimeRunning'
  | 'runtimeStatusError'
  | 'runtimeErrorLabel'
  | 'runtimeResult'
  | 'runtimeSummaryTitle'
> = {
  appTitle: {
    de: 'Simulator-Startkonfiguration',
    en: 'Simulator Start Configuration',
    ja: 'シミュレーター開始設定',
  },
  languageLabel: {
    de: 'Sprache',
    en: 'Language',
    ja: '言語',
  },
  headerPath: {
    de: 'Konfigurations-JSON für',
    en: 'Configuration JSON for',
    ja: '設定JSONファイル',
  },
  headerNote: {
    de: 'League- und Competition-Indices sind 0-basiert.',
    en: 'League and competition indices are 0-based.',
    ja: 'リーグとコンペティションインデックスは0始まりです。',
  },
  startStateSection: {
    de: 'Startzustand',
    en: 'Start state',
    ja: '開始設定',
  },
  policySection: {
    de: 'Policy',
    en: 'Policy',
    ja: 'ポリシー',
  },
  startStatePlayerRank: {
    de: 'Spieler-Rang',
    en: 'Player rank',
    ja: 'プレイヤーランク',
  },
  startStateGold: {
    de: 'Gold (Münzen)',
    en: 'Gold (coins)',
    ja: 'ゴールド（コイン）',
  },
  startStateDiamonds: {
    de: 'Diamanten',
    en: 'Diamonds',
    ja: 'ダイヤモンド',
  },
  startStateLeague: {
    de: 'Liga',
    en: 'League',
    ja: 'リーグ',
  },
  startStateCompetition: {
    de: 'Wettkampf',
    en: 'Competition',
    ja: 'コンペ',
  },
  startStateGeneration: {
    de: 'Generation',
    en: 'Generation',
    ja: '世代',
  },
  startStateRetirements: {
    de: 'Rücksetzungen',
    en: 'Retirements',
    ja: 'リタイア',
  },
  startStateMagikarpLevel: {
    de: 'Karpador-Level',
    en: 'Magikarp level',
    ja: 'コイキングLv',
  },
  startStateMagikarpKp: {
    de: 'Karpador-KP',
    en: 'Magikarp KP',
    ja: 'コイキングKP',
  },
  startStateCandy: {
    de: 'Bonbons',
    en: 'Candy',
    ja: 'キャンディ',
  },
  startStateTrainingSodas: {
    de: 'Training-Sodas',
    en: 'Training sodas',
    ja: 'トレーニングソーダ',
  },
  startStateSkillHerbs: {
    de: 'Skill-Kräuter',
    en: 'Skill herbs',
    ja: 'スキルハーブ',
  },
  startStateLeagueAids: {
    de: 'Liga-Supports',
    en: 'League aids',
    ja: 'リーグ支援',
  },
  startStateOwnedSupports: {
    de: 'Unterstützer',
    en: 'Supports',
    ja: 'サポート',
  },
  startStateOwnedDecors: {
    de: 'Dekore',
    en: 'Decors',
    ja: 'デコ',
  },
  multiSelectHint: {
    de: 'Mehrfachauswahl mit Ctrl/⌘',
    en: 'Multi-select with Ctrl/⌘',
    ja: 'Ctrl/⌘ で複数選択',
  },
  berryLevelsTitle: {
    de: 'Berry-Levels (aktivieren + Level)',
    en: 'Food levels (enable + level)',
    ja: 'えさレベル（有効化 + レベル）',
  },
  berryLevelLabel: {
    de: 'Berry-Level',
    en: 'Food level',
    ja: 'Food level',
  },
  trainingLevelsTitle: {
    de: 'Training-Levels (aktivieren + Level)',
    en: 'Training levels (enable + level)',
    ja: 'トレーニングレベル（有効化 + レベル）',
  },
  trainingLevelLabel: {
    de: 'Training-Level',
    en: 'Training level',
    ja: 'Training level',
  },
  policyPurchasePlan: {
    de: 'Kaufplan',
    en: 'Purchase plan',
    ja: '購入計画',
  },
  policyCustomPlan: {
    de: 'Eigener purchase_plan (max. 5)',
    en: 'Custom purchase_plan (max. 5)',
    ja: 'カスタム purchase_plan（最大5）',
  },
  policyCustomPlanHint: {
    de: 'Wähle bis zu 5 unterstützende Items (Supports oder Dekos, keine Freitext-Eingabe).',
    en: 'Choose up to 5 purchase items (supports or decors, no free text input).',
    ja: '最大5件の購入対象アイテム（サポートまたはデコ）を選択してください（フリーテキスト入力不可）。',
  },
  currencyDiamonds: {
    de: 'Diamanten',
    en: 'diamonds',
    ja: 'ダイヤ',
  },
  policyAllowTrainingSodas: {
    de: 'Training-Sodas erlauben',
    en: 'Allow training sodas',
    ja: 'トレーニングソーダを許可',
  },
  policyAllowSkillHerbs: {
    de: 'Skill-Kräuter erlauben',
    en: 'Allow skill herbs',
    ja: 'スキルハーブを許可',
  },
  policyAllowSupportUpgrades: {
    de: 'Support-Upgrades erlauben',
    en: 'Allow support upgrades',
    ja: 'サポートアップグレードを許可',
  },
  policyTrainingUpgradeShare: {
    de: 'Training-Upgrade-Anteil (0..10000)',
    en: 'Training upgrade share (0..10000)',
    ja: 'トレーニングアップグレード配分 (0..10000)',
  },
  policyAllowedBerryUpgrades: {
    de: 'Erlaubte Beeren-Upgrades',
    en: 'Allowed food upgrades',
    ja: '許可されるえさ強化',
  },
  policyAllowedTrainingUpgrades: {
    de: 'Erlaubte Trainings-Upgrades',
    en: 'Allowed training upgrades',
    ja: '許可されるトレーニング強化',
  },
  policyKarpadorLossRisk: {
    de: 'Max. Verlust-Risiko (0..100)',
    en: 'Max loss risk (0..100)',
    ja: '最大損失リスク (0..100)',
  },
  policySessionsPerDay: {
    de: 'Logins pro Tag',
    en: 'Logins per day',
    ja: '1日あたりのログイン回数',
  },
  resultJsonTitle: {
    de: 'Ergebnis-JSON',
    en: 'Result JSON',
    ja: '結果JSON',
  },
  copyToClipboard: {
    de: 'In die Zwischenablage kopieren',
    en: 'Copy to clipboard',
    ja: 'クリップボードにコピー',
  },
  copiedToClipboard: {
    de: 'Kopiert!',
    en: 'Copied!',
    ja: 'コピーしました！',
  },
  downloadConfig: {
    de: 'Als start_config.json speichern',
    en: 'Save as start_config.json',
    ja: 'start_config.jsonとして保存',
  },
  loading: {
    de: 'Lade Master-Daten ...',
    en: 'Loading master data ...',
    ja: 'マスターデータを読み込み中...',
  },
  customPlanOption: {
    de: 'benutzerdefiniert',
    en: 'custom',
    ja: 'カスタム',
  },
  numberValueCorrected: {
    de: 'Wert auf {value} korrigiert',
    en: 'Corrected to {value}',
    ja: '{value} に補正されました',
  },
  numberPleaseEnterInteger: {
    de: 'Bitte eine ganze Zahl eingeben',
    en: 'Please enter a whole number',
    ja: '整数を入力してください',
  },
  numberInvalidNumber: {
    de: 'Ungültige Zahl',
    en: 'Invalid number',
    ja: '無効な数値',
  },
  numberMustBeInteger: {
    de: 'Bitte nur ganze Zahlen verwenden',
    en: 'Only whole numbers allowed',
    ja: '整数のみ入力できます',
  },
  numberOutOfRange: {
    de: 'Wert muss zwischen {min} und {max} liegen',
    en: 'Value must be between {min} and {max}',
    ja: '{min} から {max} の間の値を入力してください',
  },
  runtimeSection: {
    de: 'Browser Runtime',
    en: 'Browser runtime',
    ja: 'ブラウザ実行環境',
  },
  runtimeRun: {
    de: 'In Browser ausführen',
    en: 'Run in browser',
    ja: 'ブラウザで実行',
  },
  runtimeLoading: {
    de: 'Lädt Simulator-Runtime ...',
    en: 'Loading simulator runtime ...',
    ja: 'シミュレーターランタイムを読み込み中...',
  },
  runtimeReady: {
    de: 'Runtime bereit.',
    en: 'Runtime ready.',
    ja: 'ランタイム準備完了。',
  },
  runtimeUnavailable: {
    de: 'Runtime ist noch nicht verfügbar (Wasm-Build fehlt).',
    en: 'Runtime unavailable (Wasm build missing).',
    ja: 'ランタイムが利用できません（Wasm未ビルド）。',
  },
  runtimeRunning: {
    de: 'Läuft ...',
    en: 'Running ...',
    ja: '実行中 ...',
  },
  runtimeStatusError: {
    de: 'Runtime-Fehler',
    en: 'Runtime error',
    ja: 'ランタイムエラー',
  },
  runtimeErrorLabel: {
    de: 'Ausführungsfehler',
    en: 'Execution error',
    ja: '実行エラー',
  },
  runtimeProgress: {
    de: 'Fortschritt',
    en: 'Progress',
    ja: '進行状況',
  },
  runtimeDays: {
    de: 'Tage',
    en: 'days',
    ja: '日',
  },
  runtimeResult: {
    de: 'Ergebnis',
    en: 'Result',
    ja: '結果',
  },
  runtimeSummaryTitle: {
    de: 'Simulation-Zusammenfassung',
    en: 'Simulation summary',
    ja: 'シミュレーション要約',
  },
};

type UiTextKey = keyof typeof UI_TEXT;

const t = (key: UiTextKey, language: Language): string => UI_TEXT[key][language];
const SIMULATION_MAX_DAYS = 240;

const interpolateText = (text: string, values: Record<string, string | number>): string =>
  Object.entries(values).reduce(
    (nextText, [key, value]) => nextText.replaceAll(`{${key}}`, String(value)),
    text,
  );

const LANGUAGE_OPTIONS: Array<{ code: Language; label: string }> = [
  { code: 'de', label: 'Deutsch' },
  { code: 'en', label: 'English' },
  { code: 'ja', label: '日本語' },
];

const LEAGUE_LOCALES: Record<string, LocalizedText> = {
  '1': { de: 'Freundes-Liga', en: 'Friendly League', ja: 'フレンドリーグ' },
  '2': { de: 'Flott-Liga', en: 'Quick League', ja: 'クイックリーグ' },
  '3': { de: 'Schwer-Liga', en: 'Heavy League', ja: 'ヘビーリーグ' },
  '4': { de: 'Super-Liga', en: 'Super League', ja: 'スーパーリーグ' },
  '5': { de: 'Turbo-Liga', en: 'Turbo League', ja: 'スピードリーグ' },
  '6': { de: 'Luxus-Liga', en: 'Gorgeous League', ja: 'ゴージャスリーグ' },
  '7': { de: 'Heilungs-Liga', en: 'Healing League', ja: 'ヒールリーグ' },
  '8': { de: 'Hyper-Liga', en: 'Hyper League', ja: 'ハイパーリーグ' },
  '9': { de: 'Top-Vier-Liga', en: 'Top Four League', ja: '四天王リーグ' },
  '10': { de: 'Meister-Liga', en: 'Master League', ja: 'マスターリーグ' },
  '101': { de: 'Extra-Liga 1', en: 'Extra League 1', ja: 'エクストラリーグ1' },
  '102': { de: 'Extra-Liga 2', en: 'Extra League 2', ja: 'エクストラリーグ2' },
  '103': { de: 'Extra-Liga 3', en: 'Extra League 3', ja: 'エクストラリーグ3' },
};

const TRAINING_LOCALES: Record<string, LocalizedText> = {
  '1': { de: 'Sandsack', en: 'Sandbag Slam', ja: 'サンドバッグ' },
  '2': { de: 'Sprungzähler', en: 'Jump Counter', ja: 'はねるカウンタ' },
  '3': { de: 'Lithomith-Stoß', en: 'Dwebble Push', ja: 'イシズマイ押し' },
  '4': { de: 'Baumfällen', en: 'Timber!', ja: '大木折り' },
  '5': { de: 'Ballonpumpe', en: 'Balloon Blow', ja: '風船ふくらまし' },
  '6': { de: 'Sedimantur-Stoß', en: 'Boldore Push', ja: 'ガントル押し' },
  '7': { de: 'Miniball-Dreschen', en: 'Ball Smash', ja: 'ピンポン飛ばし' },
  '8': { de: 'Felsbrechen', en: 'Rock Cruncher', ja: '岩石割り' },
  '9': { de: 'Sprungkraftwerk', en: 'Power Generator', ja: 'はねる発電' },
  '10': { de: 'Forstellka-Stoß', en: 'Forretress Push', ja: 'フォレトス押し' },
  '11': { de: 'Pokéballdreschen', en: 'Poké Ball Smash', ja: 'モンスターボール飛ばし' },
  '12': { de: 'Eisbaumfällen', en: 'Frost Cruncher', ja: '氷柱割り' },
  '13': { de: 'Ballkicken', en: 'Soccer Ball Juggle', ja: 'はねるリフティング' },
  '14': { de: 'Geowaz-Stoß', en: 'Golem Push', ja: 'ゴローニャ押し' },
  '15': { de: 'Fußballdreschen', en: 'Soccer Ball Smash', ja: 'サッカーボール飛ばし' },
  '16': { de: 'Stahlos-Stoß', en: 'Steelix Push', ja: 'ハガネール押し' },
  '17': { de: 'Tackle-Maschine', en: 'Tackle Machine', ja: 'たいあたりマシーン' },
};

const FOOD_LOCALES: Record<string, LocalizedText> = {
  '1': { de: 'Sinelbeere', en: 'Oran Berry', ja: 'オレンのみ' },
  '2': { de: 'Tsitrusbeere', en: 'Sitrus Berry', ja: 'オボンのみ' },
  '3': { de: 'Pirsifbeere', en: 'Pecha Berry', ja: 'モモンのみ' },
  '4': { de: 'Grindobeere', en: 'Rindo Berry', ja: 'リンドのみ' },
  '5': { de: 'Kerzalbeere', en: 'Wacan Berry', ja: 'ソクノのみ' },
  '6': { de: 'Jonagobeere', en: 'Leppa Berry', ja: 'ヒメリのみ' },
  '7': { de: 'Fragiabeere', en: 'Rawst Berry', ja: 'チーゴのみ' },
  '8': { de: 'Wilbirbeere', en: 'Aspear Berry', ja: 'ナナシのみ' },
  '9': { de: 'Himmihbeere', en: 'Razz Berry', ja: 'ズリのみ' },
  '10': { de: 'Morbbeere', en: 'Bluk Berry', ja: 'ブリーのみ' },
  '11': { de: 'Lavakeks', en: 'Lava Cookie', ja: 'フエンせんべい' },
  '12': { de: 'Yantara-Sablé', en: 'Shalour Sable', ja: 'シャラサブレ' },
  '13': { de: 'Illumina-Galette', en: 'Lumiose Galette', ja: 'ミアレガレット' },
  '14': { de: 'Casteliacone', en: 'Casteliacone', ja: 'ヒウンアイス' },
  '15': { de: 'Karpador-Krapfen', en: 'Magikarp Biscuit', ja: 'コイキングやき' },
  '16': { de: 'Forst-Yokan', en: 'Forest Yokan', ja: 'もりのヨウカン' },
  '17': { de: 'Grosses Malasada', en: 'Big Malasada', ja: 'おおきいマラサダ' },
};

const SUPPORT_LOCALES: Record<
  string,
  {
    pokemon: LocalizedText;
    item: LocalizedText;
  }
> = {
  '1': {
    pokemon: { de: 'Pikachu', en: 'Pikachu', ja: 'ピカチュウ' },
    item: { de: 'Kugelblitz', en: 'Kugelblitz', ja: 'でんきだま' },
  },
  '2': {
    pokemon: { de: 'Plinfa', en: 'Piplup', ja: 'ポッチャマ' },
    item: { de: 'Zauberwasser', en: 'Zauberwasser', ja: 'しんぴのしずく' },
  },
  '3': {
    pokemon: { de: 'Relaxo', en: 'Snorlax', ja: 'カビゴン' },
    item: { de: 'Überreste', en: 'Überreste', ja: 'たべのこし' },
  },
  '4': {
    pokemon: { de: 'Glurak', en: 'Charizard', ja: 'リザードン' },
    item: { de: 'Holzkohle', en: 'Holzkohle', ja: 'もくたん' },
  },
  '5': {
    pokemon: { de: 'Quajutsu', en: 'Greninja', ja: 'ゲッコウガ' },
    item: { de: 'Seegesang', en: 'Seegesang', ja: 'かいがらのすず' },
  },
  '6': {
    pokemon: { de: 'Mauzi', en: 'Meowth', ja: 'ニャース' },
    item: { de: 'Münzamulett', en: 'Münzamulett', ja: 'おまもりこばん' },
  },
  '7': {
    pokemon: { de: 'Bisasam', en: 'Bulbasaur', ja: 'フシギダネ' },
    item: { de: 'Wundersaat', en: 'Wundersaat', ja: 'きせきのタネ' },
  },
  '8': {
    pokemon: { de: 'Flegmon', en: 'Slowpoke', ja: 'ヤドン' },
    item: { de: 'Nassbrocken', en: 'Nassbrocken', ja: 'しめったいわ' },
  },
  '9': {
    pokemon: { de: 'Hydropi', en: 'Mudkip', ja: 'ミズゴロウ' },
    item: { de: 'Pudersand', en: 'Pudersand', ja: 'やわらかいすな' },
  },
  '10': {
    pokemon: { de: 'Robball', en: 'Popplio', ja: 'アシマリ' },
    item: { de: 'Wassertafel', en: 'Wassertafel', ja: 'しずくプレート' },
  },
  '11': {
    pokemon: { de: 'Bauz', en: 'Rowlet', ja: 'モクロー' },
    item: { de: 'Wiesentafel', en: 'Wiesentafel', ja: 'みどりのプレート' },
  },
  '12': {
    pokemon: { de: 'Flamiau', en: 'Litten', ja: 'ニャビー' },
    item: { de: 'Feuertafel', en: 'Feuertafel', ja: 'ひのたまプレート' },
  },
  '13': {
    pokemon: { de: 'Gengar', en: 'Gengar', ja: 'ゲンガー' },
    item: { de: 'Giftschleim', en: 'Giftschleim', ja: 'くろいヘドロ' },
  },
  '14': {
    pokemon: { de: 'Evoli', en: 'Eevee', ja: 'イーブイ' },
    item: { de: 'Sanftglocke', en: 'Sanftglocke', ja: 'アイテム' },
  },
  '15': {
    pokemon: { de: 'Mimigma', en: 'Mimikyu', ja: 'ミミッキュ' },
    item: { de: 'Bannsticker', en: 'Bannsticker', ja: 'アイテム' },
  },
  '16': {
    pokemon: { de: 'Guardevoir', en: 'Gardevoir', ja: 'サーナイト' },
    item: { de: 'Wahlschal', en: 'Wahlschal', ja: 'アイテム' },
  },
};

const DECOR_LOCALES: Record<string, LocalizedText> = {
  '1': { de: 'Octillery-Vase', en: 'Octillery Vase', ja: 'オクタンツボ' },
  '2': { de: 'Tuska-Kaktus', en: 'Tusk Kaktus', ja: 'ウソッキー盆栽' },
  '3': { de: 'Starmie-Sprudler', en: 'Starmie Sprudler', ja: 'スターミーシャワー' },
  '5': { de: 'Mogelbaum-Bonsai', en: 'Mogelbaum Bonsai', ja: 'ナッシーツリー' },
  '6': { de: 'Angelverbotsschild', en: 'No-Fishing Sign', ja: 'だいじなかんばん' },
  '8': { de: 'Laterneco-Lampe', en: 'Laterneco Lamp', ja: 'ランプラーランプ' },
  '9': { de: 'Parasek-Pilze', en: 'Parasek Mushrooms', ja: 'パラセクトダケ' },
  '10': { de: 'Sonnflora-Blumen', en: 'Sonnflora Flowers', ja: 'キマワリフラワー' },
  '11': { de: 'Digdri-Stein', en: 'Digdri Stone', ja: 'ダグトリオロック' },
  '12': { de: 'Shaymin-Pflanze', en: 'Shaymin Plant', ja: 'シェイミそう' },
  '13': { de: 'Piepi-Puppe', en: 'Clefairy Doll', ja: 'ピッピにんぎょう' },
  '15': { de: 'Delegator-Puppe', en: 'Delegator Doll', ja: 'みがわりぬいぐるみ' },
  '16': { de: 'Elfun-Kissen', en: 'Whimsicott Cushion', ja: 'エルフーンクッション' },
  '17': { de: 'Dressella-Puppe', en: 'Dressella Doll', ja: 'ドレディアドール' },
  '18': { de: 'Evoli-Bronzefigur', en: 'Bronze Eevee Figurine', ja: 'イーブイどうぞう' },
  '20': { de: 'Modell "MS Anne"', en: 'SS Anne Model', ja: 'サントアンヌ号もけい' },
  '22': { de: 'Durengard-Figur', en: 'Aegislash Figurine', ja: 'ギルガルドオブジェ' },
  '24': { de: 'Kokowei-Palmen', en: 'Cacnea', ja: 'サボネアポット' },
  '25': { de: 'Rote Kappe', en: 'Red Cap', ja: '赤いぼうし' },
  '26': { de: 'Ditto-Kissen', en: 'Ditto Cushion', ja: 'メタモンクッション' },
  '27': { de: 'Karpador Figur', en: 'Magikarp Figurine', ja: '金のコイキング像' },
};

const SUPPORT_TARGET_IDS: Record<string, string> = {
  '1': 'pikachu',
  '2': 'piplup',
  '3': 'snorlax',
  '4': 'charizard',
  '5': 'greninja',
  '6': 'meowth',
  '7': 'bulbasaur',
  '8': 'slowpoke',
  '9': 'mudkip',
  '10': 'popplio',
  '11': 'rowlet',
  '12': 'litten',
  '13': 'gengar',
  '14': 'eevee',
  '15': 'mimikyu',
  '16': 'gardevoir',
};

const DECOR_TARGET_IDS: Record<string, string> = {
  '1': 'octillery_pot',
  '2': 'sudowoodo_bonsai',
  '3': 'starmie_shower',
  '5': 'exeggutor_palm',
  '6': 'important_sign',
  '8': 'lampent_lamp',
  '9': 'parasect_puffballs',
  '10': 'sunflora_bloom',
  '11': 'dugtrio_rock',
  '12': 'shaymin_planter',
  '13': 'clefairy_doll',
  '15': 'substitute_plush',
  '16': 'whimsicott_cushion',
  '17': 'lilligant_doll',
  '18': 'bronze_eevee',
  '20': 'ss_anne_model',
  '22': 'aegislash_statue',
  '24': 'cacnea_planter',
  '25': 'red_cap',
  '26': 'ditto_cushion',
  '27': 'gold_magikarp_statue',
};

const toSupportTargetId = (id: string): string => SUPPORT_TARGET_IDS[id] ?? id;
const toDecorTargetId = (id: string): string => DECOR_TARGET_IDS[id] ?? id;
const supportTargetIdSet = new Set(Object.values(SUPPORT_TARGET_IDS));
const decorTargetIdSet = new Set(Object.values(DECOR_TARGET_IDS));

const normalizePurchasePlanTargetId = (id: string): string => {
  if (!id) {
    return '';
  }
  if (supportTargetIdSet.has(id) || decorTargetIdSet.has(id)) {
    return id;
  }
  if (SUPPORT_TARGET_IDS[id]) {
    return SUPPORT_TARGET_IDS[id];
  }
  if (DECOR_TARGET_IDS[id]) {
    return DECOR_TARGET_IDS[id];
  }
  return '';
};

const normalizePurchasePlanIds = (ids: string[]): string[] =>
  Array.from(new Set(ids.map(normalizePurchasePlanTargetId).filter(Boolean)));

const range = (start: number, end: number): number[] =>
  Array.from({ length: Math.max(0, end - start + 1) }, (_, i) => start + i);

const toInt = (value: string | number, fallback = 0): number => {
  const n = Number(value);
  return Number.isFinite(n) ? n : fallback;
};

const clampNumber = (value: number, min: number, max: number): number =>
  Math.min(Math.max(value, min), max);

const clampToStep = (value: number, step: number): number => {
  return Math.round(value / step) * step;
};

const resolveLocalizedName = (
  id: string,
  memo: string,
  map: Record<string, LocalizedText>,
  language: Language,
): string => {
  if (language === 'ja') {
    return memo || `#${id}`;
  }
  const localized = map[id]?.[language];
  if (localized) {
    return localized;
  }
  return memo || map[id]?.ja || `#${id}`;
};

const resolvePurchasePrice = (item: CatalogItem): number =>
  toInt(item.unlock_price ?? item.price, 0);

const formatPurchasePlanOptionLabel = (
  label: string,
  cost: number,
  language: Language,
): string => `${label} (${cost} ${t('currencyDiamonds', language)})`;

const isBattleRewardUnlocked = (
  item: CatalogItem,
  selectedLeague: number,
  selectedCompetition: number,
  competitionPositions: Record<string, { league: number; competition: number }>,
): boolean => {
  const unlockPrice = toInt(item.unlock_price ?? item.price, 0);
  if (unlockPrice > 0) {
    return false;
  }
  const rewardId = item.reward_competition_id;
  if (!rewardId) {
    return false;
  }
  const rewardPosition = competitionPositions[rewardId];
  if (!rewardPosition) {
    return false;
  }
  if (rewardPosition.league > selectedLeague) {
    return false;
  }
  if (rewardPosition.league < selectedLeague) {
    return true;
  }
  return rewardPosition.competition <= selectedCompetition;
};

const getUnlockedItemsForLeagueCompetition = (
  options: OptionData,
  league: number,
  competition: number,
): { ownedSupports: string[]; ownedDecors: string[] } => {
  const unlockedSupports = options.supports
    .filter((support) => isBattleRewardUnlocked(support, league, competition, options.competitionPositions))
    .map((support) => support.id);
  const unlockedDecors = options.decors
    .filter((decor) => isBattleRewardUnlocked(decor, league, competition, options.competitionPositions))
    .map((decor) => decor.id);
  return { ownedSupports: unlockedSupports, ownedDecors: unlockedDecors };
};

const supportLabel = (support: CatalogItem, language: Language): string => {
  const locale = SUPPORT_LOCALES[support.id];
  if (locale) {
    const pokemonName = locale.pokemon[language] || locale.pokemon.en || locale.pokemon.ja;
    const itemName =
      locale.item[language] || locale.item.en || locale.item.ja || support.item_name_memo || `#${support.id}`;
    return `${itemName} (${pokemonName})`;
  }
  return `${support.item_name_memo || support.name_memo || support.name}`;
};

const decorLabel = (decor: CatalogItem, language: Language): string => {
  return (
    DECOR_LOCALES[decor.id]?.[language] ||
    DECOR_LOCALES[decor.id]?.en ||
    DECOR_LOCALES[decor.id]?.ja ||
    decor.name_memo ||
    decor.name
  );
};

interface NumberInputProps {
  label: string;
  language: Language;
  value: number;
  min: number;
  max: number;
  step?: number;
  suffix?: string;
  disabled?: boolean;
  onCommit: (value: number) => void;
}

const NumberInput = ({
  label,
  language,
  value,
  min,
  max,
  step = 1,
  suffix,
  disabled,
  onCommit,
}: NumberInputProps): JSX.Element => {
  const [input, setInput] = useState(String(value));
  const [error, setError] = useState('');

  useEffect(() => {
    setInput(String(value));
  }, [value]);

  const commitValue = (nextValue: number): void => {
    const clamped = clampNumber(nextValue, min, max);
    const clampedStep = clampToStep(clamped, step);
    const finalValue = clampNumber(clampedStep, min, max);
    setError(
      finalValue === nextValue
        ? ''
        : interpolateText(t('numberValueCorrected', language), { value: finalValue }),
    );
    setInput(String(finalValue));
    onCommit(finalValue);
  };

  const onInput = (raw: string): void => {
    if (disabled) {
      return;
    }
    setInput(raw);
    if (raw.trim() === '') {
      setError(t('numberPleaseEnterInteger', language));
      return;
    }
    const parsed = Number(raw);
    if (!Number.isFinite(parsed)) {
      setError(t('numberInvalidNumber', language));
      return;
    }
    if (!Number.isInteger(parsed)) {
      setError(t('numberMustBeInteger', language));
      return;
    }
    if (parsed < min || parsed > max) {
      setError(interpolateText(t('numberOutOfRange', language), { min, max }));
      return;
    }
    setError('');
    onCommit(parsed);
  };

  return (
    <label className="number-field">
      <span>
        {label}
        {suffix ? ` (${suffix})` : ''}
      </span>
      <input
        disabled={disabled}
        type="number"
        min={min}
        max={max}
        step={step}
        value={input}
        onChange={(e) => onInput(e.target.value)}
        onBlur={() => {
          const parsed = Number(input);
          commitValue(Number.isFinite(parsed) ? parsed : value);
        }}
      />
      {error && <small>{error}</small>}
    </label>
  );
};

const safeCopyValue = <T,>(obj: T): T => JSON.parse(JSON.stringify(obj));

const createDefaultLevelMap = (items: CatalogItem[], value = 1): Record<string, number> =>
  Object.fromEntries(items.map((entry) => [String(entry.id), value]));

const createDefaultEnabledMap = (items: CatalogItem[], count = 0): Record<string, boolean> =>
  Object.fromEntries(items.map((entry, index) => [String(entry.id), index < count]));

const normalizeUpgradeId = (prefix: 'food' | 'training', id: string): string => {
  if (id.startsWith(`${prefix}_`)) {
    return id;
  }
  return `${prefix}_${id}`;
};

const normalizeBerryUpgradeIds = (ids: string[]): string[] => ids.map((id) => normalizeUpgradeId('food', id));
const normalizeTrainingUpgradeIds = (ids: string[]): string[] =>
  ids.map((id) => normalizeUpgradeId('training', id));

const selectedUpgradeIds = (items: Record<string, boolean>): string[] =>
  Object.entries(items)
    .filter((entry): entry is [string, true] => entry[1] === true)
    .map(([id]) => id);

async function loadJson<T>(path: string): Promise<T[]> {
  const response = await fetch(path);
  if (!response.ok) {
    throw new Error(`Failed to load ${path}`);
  }
  return response.json() as Promise<T[]>;
}

type StartStateScalarKey =
  | 'player_rank'
  | 'gold'
  | 'diamonds'
  | 'league'
  | 'competition'
  | 'generation'
  | 'retirements'
  | 'magikarp_level'
  | 'magikarp_kp'
  | 'candy'
  | 'training_sodas'
  | 'skill_herbs'
  | 'league_aids';

type PolicyStateScalarKey =
  | 'purchase_plan'
  | 'allow_training_sodas'
  | 'allow_skill_herbs'
  | 'allow_support_upgrades'
  | 'training_upgrade_share'
  | 'karpador_loss_risk_max_level_percent'
  | 'sessions_per_day';

interface SimulationWorkerMessageSuccess {
  type: 'success';
  result: RuntimeResult;
}

interface SimulationWorkerMessageFailure {
  type: 'failure';
  error: string;
}

type SimulationWorkerMessage = SimulationWorkerMessageSuccess | SimulationWorkerMessageFailure;

function App() {
  const [language, setLanguage] = useState<Language>('de');
  const [options, setOptions] = useState<OptionData>({
    supports: [],
    decors: [],
    berries: [],
    trainings: [],
    leagues: [],
    competitionPositions: {},
    leagueCompetitionCounts: {},
    maxPlayerRank: 99,
    maxMagikarpRank: 200,
    maxBerryLevel: 17,
    maxTrainingLevel: 17,
  });

  const [form, setForm] = useState<FormState | null>(null);
  const [customSupportPlan, setCustomSupportPlan] = useState<string[]>([]);
  const [copiedToClipboard, setCopiedToClipboard] = useState(false);
  const [loadError, setLoadError] = useState('');
  const [runtimeApi, setRuntimeApi] = useState<RuntimeApi | null>(null);
  const [runtimeStatus, setRuntimeStatus] = useState<string>('unavailable');
  const [runtimeLoadError, setRuntimeLoadError] = useState('');
  const [simulationRunning, setSimulationRunning] = useState(false);
  const [simulationProgressDays, setSimulationProgressDays] = useState(0);
  const [simulationError, setSimulationError] = useState('');
  const [simulationResult, setSimulationResult] = useState<RuntimeResult | null>(null);
  const simulationProgressTimerRef = useRef<number | null>(null);
  const simulationWorkerRef = useRef<Worker | null>(null);

  useEffect(() => {
    return () => {
      if (simulationProgressTimerRef.current != null) {
        clearInterval(simulationProgressTimerRef.current);
        simulationProgressTimerRef.current = null;
      }
      if (simulationWorkerRef.current) {
        simulationWorkerRef.current.terminate();
        simulationWorkerRef.current = null;
      }
    };
  }, []);

  useEffect(() => {
    let active = true;
    const loadRuntime = async () => {
      setRuntimeStatus('loading');
      try {
        const api = await loadRuntimeApi();
        if (active) {
          setRuntimeApi(api);
          setRuntimeStatus('ready');
          setRuntimeLoadError('');
        }
      } catch (error) {
        if (active) {
          setRuntimeApi(null);
          setRuntimeStatus('unavailable');
          setRuntimeLoadError(String(error));
        }
      }
    };
    loadRuntime();
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    const load = async () => {
      try {
        const [
          supports,
          decors,
          berries,
          trainings,
          competitions,
          extraCompetitions,
          breederRanks,
          magikarpRanks,
          leagues,
          extraLeagues,
        ] =
          await Promise.all([
            loadJson<CatalogItem>('/master_data/support_pokemon.json'),
            loadJson<CatalogItem>('/master_data/decoration.json'),
            loadJson<CatalogItem>('/master_data/food_base_data.json'),
            loadJson<CatalogItem>('/master_data/training_base_data.json'),
            loadJson<CompetitionRow>('/master_data/competition_list.json'),
            loadJson<CompetitionRow>('/master_data/extra_competition_list.json'),
            loadJson<RankRow>('/master_data/breeder_rank.json'),
            loadJson<RankRow>('/master_data/magikarp_rank.json'),
            loadJson<LeagueListRow>('/master_data/league_list.json'),
            loadJson<LeagueListRow>('/master_data/extra_league_list.json'),
        ]);

        const leagueRows = [...leagues, ...extraLeagues];
        const competitionRows = [...competitions, ...extraCompetitions];
        const leagueIdList = leagueRows
          .map((row) => toInt(row.id, -1))
          .filter((id) => Number.isFinite(id))
          .sort((a, b) => a - b)
          .filter((value, index, source) => source.indexOf(value) === index)
          .map((id) => String(id));
        const leagueById = new Map(leagueRows.map((row) => [row.id, row]));
        const leagueIndexById = new Map(leagueIdList.map((id, index) => [id, index]));
        const leagueCompetitionCounts: Record<number, number> = {};
        const competitionPositions: Record<string, { league: number; competition: number }> = {};
        const leagueRunningCounts = new Map<number, number>();
        competitionRows.forEach((row) => {
          const leagueIndex = leagueIndexById.get(String(toInt(row.league_id, -1)));
          if (leagueIndex === undefined) {
            return;
          }
          const prevCount = leagueRunningCounts.get(leagueIndex) ?? 0;
          competitionPositions[row.id] = {
            league: leagueIndex,
            competition: prevCount,
          };
          leagueRunningCounts.set(leagueIndex, prevCount + 1);
          leagueCompetitionCounts[leagueIndex] = prevCount + 1;
        });

        const leaguesWithMeta: CatalogLeague[] = leagueIdList.map((id) => ({
          id,
          name_memo: leagueById.get(id)?.name_memo || `League ${id}`,
        }));

        const sortedRanks = [...breederRanks].map((row) => toInt(row.rank)).sort((a, b) => a - b);
        const maxPlayerRank = sortedRanks.at(-1) ?? 99;
        const maxMagikarpRank =
          [...magikarpRanks].map((row) => toInt(row.rank)).sort((a, b) => a - b).at(-1) ?? 200;
        const initialLeague = 4;
        const initialCompetitionRaw = 2;
        const initialCompetition = clampNumber(
          initialCompetitionRaw,
          0,
          Math.max(0, toInt(leagueCompetitionCounts[initialLeague], 5) - 1),
        );
        const unlockedSupports = supports
          .filter((support) => isBattleRewardUnlocked(support, initialLeague, initialCompetition, competitionPositions))
          .map((support) => support.id);
        const unlockedDecors = decors
          .filter((decor) => isBattleRewardUnlocked(decor, initialLeague, initialCompetition, competitionPositions))
          .map((decor) => decor.id);

        setOptions({
          supports,
          decors,
          berries,
          trainings,
          leagueCompetitionCounts,
          competitionPositions,
          maxPlayerRank,
          maxMagikarpRank,
          maxBerryLevel: 17,
          maxTrainingLevel: 17,
          leagues: leaguesWithMeta,
        });

        setForm({
          start_state: {
            player_rank: 25,
            gold: 12345,
            diamonds: 300,
            league: initialLeague,
            competition: initialCompetition,
            generation: 18,
            retirements: 17,
            magikarp_level: 31,
            magikarp_kp: 0,
            candy: 4,
            training_sodas: 2,
            skill_herbs: 1,
            league_aids: 0,
            owned_supports: Array.from(
              new Set([
                'pikachu',
                'charizard',
                ...unlockedSupports.map((id) => toSupportTargetId(id)),
              ]),
            ),
            owned_decors: Array.from(new Set(['shaymin_planter', ...unlockedDecors.map((id) => toDecorTargetId(id))])),
            berry_levels: createDefaultLevelMap(berries, 1),
            training_levels: createDefaultLevelMap(trainings, 1),
            berry_enabled: createDefaultEnabledMap(berries, 2),
            training_enabled: createDefaultEnabledMap(trainings, 2),
          },
          policy: {
            allowed_berry_upgrades: selectedUpgradeIds(createDefaultEnabledMap(berries, 2)),
            allowed_training_upgrades: selectedUpgradeIds(createDefaultEnabledMap(trainings, 2)),
            purchase_plan: 'custom',
            allow_training_sodas: true,
            allow_skill_herbs: true,
            allow_support_upgrades: true,
            training_upgrade_share: 2500,
            karpador_loss_risk_max_level_percent: 60,
            sessions_per_day: 10,
          },
        });
      } catch (err) {
        setLoadError(`Fehler beim Laden der Master-Daten: ${String(err)}`);
      }
    };

    load();
  }, []);

  const leagueOptions = useMemo(() => {
    const keys = Object.keys(options.leagueCompetitionCounts)
      .map((k) => Number(k))
      .filter((k) => Number.isFinite(k));
    const maxLeague = keys.length > 0 ? Math.max(...keys) : options.leagues.length - 1;
    return range(0, Math.max(maxLeague, options.leagues.length - 1));
  }, [options.leagueCompetitionCounts, options.leagues.length]);

  const competitionOptions = (league: number): number[] => {
    const count = toInt(options.leagueCompetitionCounts[league], 5);
    return range(0, Math.max(0, count - 1));
  };

  const getLeagueLabel = (leagueIndex: number): string => {
    const league = options.leagues[leagueIndex];
    if (!league) {
      return `Liga ${leagueIndex}`;
    }
    return (
      LEAGUE_LOCALES[league.id]?.[language] ?? LEAGUE_LOCALES[league.id]?.en ?? league.name_memo ?? `Liga ${leagueIndex}`
    );
  };

  const trainingLabel = (training: CatalogItem): string =>
    resolveLocalizedName(training.id, training.name_memo || training.name, TRAINING_LOCALES, language);

  const berryLabel = (berry: CatalogItem): string =>
    resolveLocalizedName(berry.id, berry.name_memo || berry.name, FOOD_LOCALES, language);

  const config = useMemo(() => {
    if (!form) {
      return '{\n  "status": "loading"\n}';
    }

    const allowedBerries = form.start_state.berry_enabled || {};
    const allowedTrainings = form.start_state.training_enabled || {};
    const activeBerryUpgrades = selectedUpgradeIds(allowedBerries);
    const activeTrainingUpgrades = selectedUpgradeIds(allowedTrainings);
    const currentUnlocked = getUnlockedItemsForLeagueCompetition(
      options,
      toInt(form.start_state.league, 0),
      toInt(form.start_state.competition, 0),
    );
    const unlockedPlanItemIds = new Set(
      [...currentUnlocked.ownedSupports, ...currentUnlocked.ownedDecors]
        .map(normalizePurchasePlanTargetId)
        .filter(Boolean),
    );
    const purchasablePlanItemSet = new Set(
      [
        ...options.supports.filter((support) => resolvePurchasePrice(support) > 0).map((support) => toSupportTargetId(support.id)),
        ...options.decors.filter((decor) => resolvePurchasePrice(decor) > 0).map((decor) => toDecorTargetId(decor.id)),
      ]
        .map(normalizePurchasePlanTargetId)
        .filter((id) => id !== '' && !unlockedPlanItemIds.has(id)),
    );
    const unlockedSupportSet = new Set(currentUnlocked.ownedSupports.map(toSupportTargetId));
    const unlockedDecorSet = new Set(currentUnlocked.ownedDecors.map(toDecorTargetId));

    const startStatePayload = {
      player_rank: toInt(form.start_state.player_rank, 1),
      gold: toInt(form.start_state.gold, 0),
      diamonds: toInt(form.start_state.diamonds, 0),
      league: toInt(form.start_state.league, 0),
      competition: toInt(form.start_state.competition, 0),
      generation: toInt(form.start_state.generation, 1),
      retirements: toInt(form.start_state.retirements, 0),
      magikarp_level: toInt(form.start_state.magikarp_level, 1),
      magikarp_kp: toInt(form.start_state.magikarp_kp, 0),
      candy: toInt(form.start_state.candy, 0),
      training_sodas: toInt(form.start_state.training_sodas, 0),
      skill_herbs: toInt(form.start_state.skill_herbs, 0),
      league_aids: toInt(form.start_state.league_aids, 0),
      owned_supports: [
        ...new Set([
          ...form.start_state.owned_supports.filter((id) => !unlockedSupportSet.has(id)),
          ...currentUnlocked.ownedSupports.map(toSupportTargetId),
        ]),
      ],
      owned_decors: [
        ...new Set([
          ...form.start_state.owned_decors.filter((id) => !unlockedDecorSet.has(id)),
          ...currentUnlocked.ownedDecors.map(toDecorTargetId),
        ]),
      ],
      berry_levels: Object.fromEntries(
        Object.entries(form.start_state.berry_levels).filter(([key]) => allowedBerries[key]),
      ),
      training_levels: Object.fromEntries(
        Object.entries(form.start_state.training_levels).filter(([key]) => allowedTrainings[key]),
      ),
    };

    startStatePayload.berry_levels = Object.fromEntries(
      Object.entries(startStatePayload.berry_levels).map(([key, value]) => [
        normalizeUpgradeId('food', key),
        toInt(value, 1),
      ]),
    );
    startStatePayload.training_levels = Object.fromEntries(
      Object.entries(startStatePayload.training_levels).map(([key, value]) => [
        normalizeUpgradeId('training', key),
        toInt(value, 1),
      ]),
    );

    const policyState = safeCopyValue(form.policy);
  const policyPayload: PolicyState = {
      ...policyState,
      training_upgrade_share: toInt(policyState.training_upgrade_share, 0),
      sessions_per_day: clampNumber(toInt(policyState.sessions_per_day, 0), 1, 255),
      karpador_loss_risk_max_level_percent: toInt(policyState.karpador_loss_risk_max_level_percent, 0),
      allowed_berry_upgrades: Array.from(
        new Set([...normalizeBerryUpgradeIds(policyState.allowed_berry_upgrades.filter(Boolean)), ...normalizeBerryUpgradeIds(activeBerryUpgrades)]),
      ),
      allowed_training_upgrades: Array.from(
        new Set([
          ...normalizeTrainingUpgradeIds(policyState.allowed_training_upgrades.filter(Boolean)),
          ...normalizeTrainingUpgradeIds(activeTrainingUpgrades),
        ]),
      ),
    };

    policyPayload.purchase_plan = JSON.stringify(
      normalizePurchasePlanIds(customSupportPlan)
        .filter((id) => purchasablePlanItemSet.has(id))
        .slice(0, 5),
    );

    return JSON.stringify(
      {
        start_state: startStatePayload,
        policy: policyPayload,
      },
      null,
      2,
    );
  }, [customSupportPlan, form, options, options.maxMagikarpRank]);

  const handleStartStateChange = (key: StartStateScalarKey, value: number): void => {
    setForm((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        start_state: {
          ...prev.start_state,
          [key]: value,
        },
      };
    });
  };

  const handlePolicyChange = (
    key: PolicyStateScalarKey,
    value: boolean | number | string | string[],
  ): void => {
    setForm((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        policy: {
          ...prev.policy,
          [key]: value,
        },
      };
    });
  };

  const handleArraySelect = (
    field: 'owned_supports' | 'owned_decors',
    event: ChangeEvent<HTMLSelectElement>,
  ): void => {
    const values = Array.from(event.target.selectedOptions).map((item) => item.value);
    const toCanonicalId = (id: string): string =>
      field === 'owned_supports' ? toSupportTargetId(id) : toDecorTargetId(id);
    setForm((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        start_state: {
          ...prev.start_state,
          [field]: values.map(toCanonicalId),
        },
      };
    });
  };

  const toggleBerry = (id: string): void => {
    setForm((prev) => {
      if (!prev) return prev;
      const nextEnabled = !prev.start_state.berry_enabled[id];
      return {
        ...prev,
        start_state: {
          ...prev.start_state,
          berry_enabled: {
            ...prev.start_state.berry_enabled,
            [id]: nextEnabled,
          },
        },
        policy: {
          ...prev.policy,
          allowed_berry_upgrades: nextEnabled
            ? [...new Set([...prev.policy.allowed_berry_upgrades.filter(Boolean), id])]
            : prev.policy.allowed_berry_upgrades.filter(Boolean).filter((entry) => entry !== id),
        },
      };
    });
  };

  const toggleTraining = (id: string): void => {
    setForm((prev) => {
      if (!prev) return prev;
      const nextEnabled = !prev.start_state.training_enabled[id];
      return {
        ...prev,
        start_state: {
          ...prev.start_state,
          training_enabled: {
            ...prev.start_state.training_enabled,
            [id]: nextEnabled,
          },
        },
        policy: {
          ...prev.policy,
          allowed_training_upgrades: nextEnabled
            ? [...new Set([...prev.policy.allowed_training_upgrades.filter(Boolean), id])]
            : prev.policy.allowed_training_upgrades.filter(Boolean).filter((entry) => entry !== id),
        },
      };
    });
  };

  const updateBerryLevel = (id: string, level: number): void => {
    setForm((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        start_state: {
          ...prev.start_state,
          berry_levels: {
            ...prev.start_state.berry_levels,
            [id]: toInt(level, 1),
          },
        },
      };
    });
  };

  const updateTrainingLevel = (id: string, level: number): void => {
    setForm((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        start_state: {
          ...prev.start_state,
          training_levels: {
            ...prev.start_state.training_levels,
            [id]: toInt(level, 1),
          },
        },
      };
    });
  };

  const updateCustomSupportPlanSlot = (index: number, value: string): void => {
    setCustomSupportPlan((prev) => {
      const next = [...prev];
      while (next.length < 5) {
        next.push('');
      }
      next[index] = normalizePurchasePlanTargetId(value);
      return next.slice(0, 5);
    });
  };

  const availablePurchasePlanOptionsForSlot = (index: number): PurchasePlanOption[] => {
    if (!form) {
      return [];
    }
    const unlockedItems = getUnlockedItemsForLeagueCompetition(
      options,
      form.start_state.league,
      form.start_state.competition,
    );
    const normalizedUnlockedItemIds = new Set(
      [
        ...unlockedItems.ownedSupports.map(toSupportTargetId),
        ...unlockedItems.ownedDecors.map(toDecorTargetId),
      ],
    );
    const otherSelected = new Set(
      customSupportPlan
        .map((entry, entryIndex) => (entryIndex === index ? '' : entry))
        .map(normalizePurchasePlanTargetId)
        .filter((entry) => entry !== ''),
    );
    const uniqueOptions = new Map<string, PurchasePlanOption>();
    for (const support of options.supports) {
      const option = {
        id: toSupportTargetId(support.id),
        label: formatPurchasePlanOptionLabel(
          supportLabel(support, language),
          resolvePurchasePrice(support),
          language,
        ),
        cost: resolvePurchasePrice(support),
      };
      if (option.cost <= 0 || uniqueOptions.has(option.id)) {
        continue;
      }
      uniqueOptions.set(option.id, option);
    }
    for (const decor of options.decors) {
      const option = {
        id: toDecorTargetId(decor.id),
        label: formatPurchasePlanOptionLabel(decorLabel(decor, language), resolvePurchasePrice(decor), language),
        cost: resolvePurchasePrice(decor),
      };
      if (option.cost <= 0 || uniqueOptions.has(option.id)) {
        continue;
      }
      uniqueOptions.set(option.id, option);
    }

    return [...uniqueOptions.values()]
      .filter((entry) => !otherSelected.has(entry.id) && !normalizedUnlockedItemIds.has(entry.id))
      .sort((a, b) => a.cost - b.cost || a.label.localeCompare(b.label));
  };

  const copyToClipboard = async () => {
    await navigator.clipboard.writeText(config);
    setCopiedToClipboard(true);
    setTimeout(() => setCopiedToClipboard(false), 1500);
  };

  const stopSimulationProgressTracking = (): void => {
    if (simulationProgressTimerRef.current != null) {
      clearInterval(simulationProgressTimerRef.current);
      simulationProgressTimerRef.current = null;
    }
    if (simulationWorkerRef.current) {
      simulationWorkerRef.current.terminate();
      simulationWorkerRef.current = null;
    }
  };

  const runSimulationInBrowser = async (): Promise<void> => {
    if (!runtimeApi || !form) {
      return;
    }
    setSimulationRunning(true);
    setSimulationError('');
    setSimulationProgressDays(0);
    setSimulationResult(null);

    const start = performance.now();
    const maxDays = SIMULATION_MAX_DAYS;
    simulationProgressTimerRef.current = window.setInterval(() => {
      const elapsed = performance.now() - start;
      const targetProgress = Math.min(maxDays - 1, Math.floor((elapsed / 7000) * maxDays));
      setSimulationProgressDays((current) => Math.min(maxDays - 1, Math.max(current, targetProgress)));
    }, 100);

    const worker = new Worker(new URL('./runtimeWorker.ts', import.meta.url), { type: 'module' });
    simulationWorkerRef.current = worker;

    try {
    const careerTargetLeague = Math.max(0, options.leagues.length - 1);
      const result = await new Promise<RuntimeResult>((resolve, reject) => {
        const onMessage = (event: MessageEvent<SimulationWorkerMessage>): void => {
          if (event.data.type === 'success') {
            resolve(event.data.result);
            return;
          }
          if (event.data.type === 'failure') {
            reject(new Error(event.data.error));
          }
        };

        worker.onmessage = onMessage;
        worker.onerror = () => {
          reject(new Error('Worker failed while running simulation.'));
        };
        worker.postMessage({
          type: 'run',
          payload: {
            config,
            seed: 42,
            maxActions: 100_000,
            maxDays,
            sessionsPerDay: toInt(form.policy.sessions_per_day, 10),
            targetLeague: careerTargetLeague,
          },
        });
      });

      const summaryDays = result.summary?.wall_days;
      if (summaryDays != null) {
        setSimulationProgressDays(clampNumber(summaryDays, 0, maxDays));
      } else {
        setSimulationProgressDays(maxDays);
      }
      setSimulationResult(result);
    } catch (error) {
      if (error instanceof Error) {
        console.error('Browser simulation failed:', error);
        console.error('Stacktrace:', error.stack);
      } else {
        console.error('Browser simulation failed:', error);
      }
      setSimulationError(String(error));
      setSimulationResult(null);
    } finally {
      stopSimulationProgressTracking();
      setSimulationRunning(false);
    }
  };

  const downloadConfig = () => {
    const blob = new Blob([config], { type: 'application/json;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'start_config.json';
    a.click();
    URL.revokeObjectURL(url);
  };

  const applyLeagueCompetitionSelection = (nextLeague: number, nextCompetition: number): void => {
    const maxCompetition = competitionOptions(nextLeague).at(-1) || 0;
    const clampedCompetition = clampNumber(nextCompetition, 0, maxCompetition);
    const unlockedItems = getUnlockedItemsForLeagueCompetition(
      options,
      nextLeague,
      clampedCompetition,
    );
    const nextUnlockedItemSet = new Set(
      [...unlockedItems.ownedSupports.map(toSupportTargetId), ...unlockedItems.ownedDecors.map(toDecorTargetId)],
    );
    const purchasablePlanItemSet = new Set(
      [
        ...options.supports.filter((support) => resolvePurchasePrice(support) > 0).map((support) => toSupportTargetId(support.id)),
        ...options.decors.filter((decor) => resolvePurchasePrice(decor) > 0).map((decor) => toDecorTargetId(decor.id)),
      ]
        .filter((id) => !nextUnlockedItemSet.has(id)),
    );
    setForm((prev) => {
      if (!prev) return prev;
      const previousUnlockedItems = getUnlockedItemsForLeagueCompetition(
        options,
        prev.start_state.league,
        prev.start_state.competition,
      );
      const previousUnlockedSupportSet = new Set(previousUnlockedItems.ownedSupports.map(toSupportTargetId));
      const previousUnlockedDecorSet = new Set(previousUnlockedItems.ownedDecors.map(toDecorTargetId));
      const keepOwnSupports = prev.start_state.owned_supports.filter((id) => !previousUnlockedSupportSet.has(id));
      const keepOwnDecors = prev.start_state.owned_decors.filter((id) => !previousUnlockedDecorSet.has(id));
      return {
        ...prev,
        start_state: {
          ...prev.start_state,
          league: nextLeague,
          competition: clampedCompetition,
          owned_supports: Array.from(
            new Set([...keepOwnSupports, ...unlockedItems.ownedSupports.map((id) => toSupportTargetId(id))]),
          ),
          owned_decors: Array.from(
            new Set([...keepOwnDecors, ...unlockedItems.ownedDecors.map((id) => toDecorTargetId(id))]),
          ),
        },
      };
    });
    setCustomSupportPlan((prev) =>
      Array.from(
        new Set(
          normalizePurchasePlanIds(prev).filter((supportId) =>
            purchasablePlanItemSet.has(supportId),
          ),
        ),
      ).slice(0, 5),
    );
  };

  if (loadError) {
    return <main className="app error">{loadError}</main>;
  }

  if (!form) {
    return <main className="app">{t('loading', language)}</main>;
  }

  const runtimeStatusText =
    runtimeStatus === 'loading'
      ? t('runtimeLoading', language)
      : runtimeStatus === 'ready'
        ? t('runtimeReady', language)
        : t('runtimeUnavailable', language);

  return (
    <main className="app">
      <header>
        <h1>{t('appTitle', language)}</h1>
        <label className="language-switch">
          <span>{t('languageLabel', language)}</span>
          <select value={language} onChange={(e) => setLanguage(e.target.value as Language)}>
            {LANGUAGE_OPTIONS.map((entry) => (
              <option key={entry.code} value={entry.code}>
                {entry.label}
              </option>
            ))}
          </select>
        </label>
        <p>
          {t('headerPath', language)} <code>simulator/examples/start_config.json</code> -{' '}
          {t('headerNote', language)}
        </p>
      </header>

      <section className="grid">
        <article>
          <h2>{t('startStateSection', language)}</h2>
          <NumberInput
            language={language}
            label={t('startStatePlayerRank', language)}
            value={form.start_state.player_rank}
            min={1}
            max={options.maxPlayerRank}
            onCommit={(value) => handleStartStateChange('player_rank', value)}
          />

          <NumberInput
            language={language}
            label={t('startStateGold', language)}
            value={form.start_state.gold}
            min={0}
            max={1_000_000}
            onCommit={(value) => handleStartStateChange('gold', value)}
          />

          <NumberInput
            language={language}
            label={t('startStateDiamonds', language)}
            value={form.start_state.diamonds}
            min={0}
            max={100_000}
            onCommit={(value) => handleStartStateChange('diamonds', value)}
          />

            <label>
              <span>{t('startStateLeague', language)}</span>
              <select
                value={form.start_state.league}
                onChange={(e) => applyLeagueCompetitionSelection(toInt(e.target.value, 0), form.start_state.competition)}
              >
              {leagueOptions.map((value) => (
                <option key={value} value={value}>
                  {getLeagueLabel(value)}
                </option>
              ))}
            </select>
            </label>

            <label>
              <span>{t('startStateCompetition', language)}</span>
              <select
                value={form.start_state.competition}
                onChange={(e) =>
                  applyLeagueCompetitionSelection(form.start_state.league, toInt(e.target.value, 0))
                }
              >
              {competitionOptions(form.start_state.league).map((value) => (
                <option key={value} value={value}>
                  {value}
                </option>
              ))}
            </select>
          </label>

          <NumberInput
            language={language}
            label={t('startStateGeneration', language)}
            value={form.start_state.generation}
            min={1}
            max={30}
            onCommit={(value) => handleStartStateChange('generation', value)}
          />

          <NumberInput
            language={language}
            label={t('startStateRetirements', language)}
            value={form.start_state.retirements}
            min={0}
            max={80}
            onCommit={(value) => handleStartStateChange('retirements', value)}
          />

          <NumberInput
            language={language}
            label={t('startStateMagikarpLevel', language)}
            value={form.start_state.magikarp_level}
            min={1}
            max={options.maxMagikarpRank}
            onCommit={(value) => handleStartStateChange('magikarp_level', value)}
          />

          <NumberInput
            language={language}
            label={t('startStateMagikarpKp', language)}
            value={form.start_state.magikarp_kp}
            min={0}
            max={5_000_000}
            onCommit={(value) => handleStartStateChange('magikarp_kp', value)}
          />

          <NumberInput
            language={language}
            label={t('startStateCandy', language)}
            value={form.start_state.candy}
            min={0}
            max={1_000}
            onCommit={(value) => handleStartStateChange('candy', value)}
          />

          <NumberInput
            language={language}
            label={t('startStateTrainingSodas', language)}
            value={form.start_state.training_sodas}
            min={0}
            max={3000}
            onCommit={(value) => handleStartStateChange('training_sodas', value)}
          />

          <NumberInput
            language={language}
            label={t('startStateSkillHerbs', language)}
            value={form.start_state.skill_herbs}
            min={0}
            max={3000}
            onCommit={(value) => handleStartStateChange('skill_herbs', value)}
          />

          <NumberInput
            language={language}
            label={t('startStateLeagueAids', language)}
            value={form.start_state.league_aids}
            min={0}
            max={3000}
            onCommit={(value) => handleStartStateChange('league_aids', value)}
          />

          <label>
            <span>
              {t('startStateOwnedSupports', language)} (owned_supports)
            </span>
            <select
              multiple
              value={form.start_state.owned_supports}
              onChange={(e) => handleArraySelect('owned_supports', e)}
              size={Math.min(options.supports.length, 7)}
            >
              {options.supports.map((support) => (
                <option key={support.id} value={toSupportTargetId(support.id)}>
                  {supportLabel(support, language)}
                </option>
              ))}
            </select>
            <small>{t('multiSelectHint', language)}</small>
          </label>

          <label>
            <span>
              {t('startStateOwnedDecors', language)} (owned_decors)
            </span>
            <select
              multiple
              value={form.start_state.owned_decors}
              onChange={(e) => handleArraySelect('owned_decors', e)}
              size={Math.min(options.decors.length, 7)}
            >
              {options.decors.map((decor) => (
                <option key={decor.id} value={toDecorTargetId(decor.id)}>
                  {decorLabel(decor, language)}
                </option>
              ))}
            </select>
          </label>

          <div className="grid-subtitle">{t('berryLevelsTitle', language)}</div>
          <div className="grid-checkbox-list">
            {options.berries.map((berry) => (
              <div className="checkbox-row" key={berry.id}>
                <label>
                  <input
                    type="checkbox"
                    checked={!!form.start_state.berry_enabled[berry.id]}
                    onChange={() => toggleBerry(berry.id)}
                  />
                  <span>
                    {berry.id} - {berryLabel(berry)}
                  </span>
                </label>
                <NumberInput
                  language={language}
                  label={`${t('berryLevelLabel', language)} ${berry.id}`}
                  value={form.start_state.berry_levels[berry.id] || 1}
                  disabled={!form.start_state.berry_enabled[berry.id]}
                  min={1}
                  max={options.maxBerryLevel}
                  onCommit={(value) => updateBerryLevel(berry.id, value)}
                />
              </div>
            ))}
          </div>

          <div className="grid-subtitle">{t('trainingLevelsTitle', language)}</div>
          <div className="grid-checkbox-list">
            {options.trainings.map((training) => (
              <div className="checkbox-row" key={training.id}>
                <label>
                  <input
                    type="checkbox"
                    checked={!!form.start_state.training_enabled[training.id]}
                    onChange={() => toggleTraining(training.id)}
                  />
                  <span>
                    {training.id} - {trainingLabel(training)}
                  </span>
                </label>
                <NumberInput
                  language={language}
                  label={`${t('trainingLevelLabel', language)} ${training.id}`}
                  value={form.start_state.training_levels[training.id] || 1}
                  disabled={!form.start_state.training_enabled[training.id]}
                  min={1}
                  max={options.maxTrainingLevel}
                  onCommit={(value) => updateTrainingLevel(training.id, value)}
                />
              </div>
            ))}
          </div>
        </article>

        <article>
          <h2>{t('policySection', language)}</h2>
          <label>
            <span>
              {t('policyPurchasePlan', language)} ({t('customPlanOption', language)})
            </span>
            <select value="custom" disabled>
              <option value="custom">{t('customPlanOption', language)}</option>
            </select>
          </label>

          <label>
            <span>{t('policyCustomPlan', language)}</span>
            <div className="purchase-plan-slots">
              {Array.from({ length: 5 }, (_, index) => {
                const value = customSupportPlan[index] || '';
                const availablePurchasePlanItems = availablePurchasePlanOptionsForSlot(index);
                return (
                  <select
                    key={`custom-plan-slot-${index}`}
                    value={value}
                    onChange={(event) => updateCustomSupportPlanSlot(index, event.target.value)}
                  >
                    <option value="">{''}</option>
                    {availablePurchasePlanItems.map((item) => (
                      <option key={item.id} value={item.id}>
                        {item.label}
                      </option>
                    ))}
                  </select>
                );
              })}
            </div>
            <small>{t('policyCustomPlanHint', language)}</small>
          </label>

          <label>
            <span>{t('policyAllowTrainingSodas', language)}</span>
            <input
              type="checkbox"
              checked={form.policy.allow_training_sodas}
              onChange={(e) => handlePolicyChange('allow_training_sodas', e.target.checked)}
            />
          </label>

          <label>
            <span>{t('policyAllowSkillHerbs', language)}</span>
            <input
              type="checkbox"
              checked={form.policy.allow_skill_herbs}
              onChange={(e) => handlePolicyChange('allow_skill_herbs', e.target.checked)}
            />
          </label>

          <label>
            <span>{t('policyAllowSupportUpgrades', language)}</span>
            <input
              type="checkbox"
              checked={form.policy.allow_support_upgrades}
              onChange={(e) => handlePolicyChange('allow_support_upgrades', e.target.checked)}
            />
          </label>

          <NumberInput
            language={language}
            label={t('policyTrainingUpgradeShare', language)}
            value={form.policy.training_upgrade_share}
            min={0}
            max={10000}
            step={250}
            onCommit={(value) => handlePolicyChange('training_upgrade_share', value)}
          />

          <label>
            <span>
              {t('policyAllowedBerryUpgrades', language)} (allowed_berry_upgrades)
            </span>
            <select
              multiple
              value={form.policy.allowed_berry_upgrades}
              onChange={(e: ChangeEvent<HTMLSelectElement>) =>
                handlePolicyChange(
                  'allowed_berry_upgrades',
                  Array.from(e.target.selectedOptions).map((item) => item.value),
                )
              }
              size={Math.min(options.berries.length, 7)}
            >
              {options.berries.map((berry) => (
                <option key={berry.id} value={berry.id}>
                  {berry.id} - {berryLabel(berry)}
                </option>
              ))}
            </select>
          </label>

          <label>
            <span>
              {t('policyAllowedTrainingUpgrades', language)} (allowed_training_upgrades)
            </span>
            <select
              multiple
              value={form.policy.allowed_training_upgrades}
              onChange={(e: ChangeEvent<HTMLSelectElement>) =>
                handlePolicyChange(
                  'allowed_training_upgrades',
                  Array.from(e.target.selectedOptions).map((item) => item.value),
                )
              }
              size={Math.min(options.trainings.length, 7)}
            >
              {options.trainings.map((training) => (
                <option key={training.id} value={training.id}>
                  {training.id} - {trainingLabel(training)}
                </option>
              ))}
            </select>
          </label>

          <NumberInput
            language={language}
            label={t('policyKarpadorLossRisk', language)}
            value={form.policy.karpador_loss_risk_max_level_percent}
            min={0}
            max={100}
            onCommit={(value) => handlePolicyChange('karpador_loss_risk_max_level_percent', value)}
          />

          <NumberInput
            language={language}
            label={t('policySessionsPerDay', language)}
            value={form.policy.sessions_per_day}
            min={1}
            max={255}
            onCommit={(value) => handlePolicyChange('sessions_per_day', value)}
          />
        </article>
      </section>

      <section>
        <h2>{t('runtimeSection', language)}</h2>
        <p className="status-line">
          {runtimeStatusText}
          {runtimeLoadError && (
            <>
              {' '}
              ({t('runtimeStatusError', language)}: {runtimeLoadError})
            </>
          )}
        </p>
        {simulationRunning && (
          <div className="progress-wrap">
            <div className="progress-meta">
              <span>
                {t('runtimeProgress', language)}: {simulationProgressDays}/{SIMULATION_MAX_DAYS} {t('runtimeDays', language)}
              </span>
            </div>
            <div className="progress-track">
              <div
                className="progress-fill"
                style={{ width: `${(simulationProgressDays / SIMULATION_MAX_DAYS) * 100}%` }}
              />
            </div>
          </div>
        )}
        <div className="actions">
          <button onClick={runSimulationInBrowser} disabled={runtimeStatus !== 'ready' || simulationRunning}>
            {runtimeStatus === 'loading' || simulationRunning ? t('runtimeRunning', language) : t('runtimeRun', language)}
          </button>
        </div>
        {simulationError && <p className="error-text">{t('runtimeErrorLabel', language)}: {simulationError}</p>}
        {simulationResult && (
          <div className="runtime-output">
            <h3>{t('runtimeResult', language)}</h3>
            <textarea className="json-output runtime-json" value={simulationResult.payload} readOnly rows={14} />
            {simulationResult.summary && (
              <>
                <h4>{t('runtimeSummaryTitle', language)}</h4>
                <pre>{JSON.stringify(simulationResult.summary, null, 2)}</pre>
              </>
            )}
          </div>
        )}
      </section>

      <section>
        <h2>{t('resultJsonTitle', language)}</h2>
        <textarea className="json-output" value={config} readOnly rows={24} />
        <div className="actions">
          <button onClick={copyToClipboard}>
            {copiedToClipboard ? t('copiedToClipboard', language) : t('copyToClipboard', language)}
          </button>
          <button onClick={downloadConfig}>{t('downloadConfig', language)}</button>
        </div>
      </section>
    </main>
  );
}

export default App;
