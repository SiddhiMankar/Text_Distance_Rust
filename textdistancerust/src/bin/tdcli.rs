// tdcli — Interactive CUI for textdistancerust (zero external deps)
// Usage: tdcli <COMMAND> [ARGS...]
//
//   list                              List all algorithms
//   compare --alg <alg[,alg]> S1 S2  Compare two strings
//   all S1 S2                         Run every algorithm on two strings
//   bench                             Native latency benchmark
//   interactive                       REPL mode

use std::env;
use std::io::{self, Write};
use std::time::Instant;
use textdistancerust::*;

// ─────────────────────────────────────────────────────────────────────────────
// ANSI colour helpers
// ─────────────────────────────────────────────────────────────────────────────
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const BLUE: &str = "\x1b[34m";
const RED: &str = "\x1b[31m";
const WHITE: &str = "\x1b[97m";
const BG_DARK: &str = "\x1b[48;5;236m";

// ─────────────────────────────────────────────────────────────────────────────
// Algorithm registry
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy)]
struct AlgInfo {
    name: &'static str,
    category: &'static str,
    emoji: &'static str,
    description: &'static str,
}

const ALGORITHMS: &[AlgInfo] = &[
    AlgInfo { name: "identity",            category: "Simple",      emoji: "🔹", description: "Exact equality check" },
    AlgInfo { name: "length",              category: "Simple",      emoji: "🔹", description: "Absolute length difference" },
    AlgInfo { name: "prefix",              category: "Simple",      emoji: "🔹", description: "Common prefix fraction" },
    AlgInfo { name: "postfix",             category: "Simple",      emoji: "🔹", description: "Common postfix fraction" },
    AlgInfo { name: "matrix",              category: "Matrix",      emoji: "🧮", description: "Custom score matrix matching" },
    AlgInfo { name: "hamming",             category: "Edit",        emoji: "📐", description: "Positional mismatch count" },
    AlgInfo { name: "levenshtein",         category: "Edit",        emoji: "📐", description: "Classic edit distance (ins/del/sub)" },
    AlgInfo { name: "damerau_levenshtein", category: "Edit",        emoji: "📐", description: "Levenshtein + transpositions (OSA)" },
    AlgInfo { name: "jaro",               category: "Edit",        emoji: "📐", description: "Jaro character-window similarity" },
    AlgInfo { name: "jaro_winkler",       category: "Edit",        emoji: "📐", description: "Jaro with prefix bonus" },
    AlgInfo { name: "needleman_wunsch",    category: "Alignment",   emoji: "🧬", description: "Global sequence alignment" },
    AlgInfo { name: "smith_waterman",      category: "Alignment",   emoji: "🧬", description: "Local sequence alignment" },
    AlgInfo { name: "gotoh",              category: "Alignment",   emoji: "🧬", description: "Global alignment with affine gaps" },
    AlgInfo { name: "lcsseq",             category: "Sequence",    emoji: "🔗", description: "Longest common subsequence" },
    AlgInfo { name: "lcsstr",             category: "Sequence",    emoji: "🔗", description: "Longest common substring" },
    AlgInfo { name: "ratcliff_obershelp", category: "Sequence",    emoji: "🔗", description: "Ratcliff/Obershelp gestalt matching" },
    AlgInfo { name: "mlipns",             category: "Edit",        emoji: "📐", description: "Bounded mismatch iterative metric" },
    AlgInfo { name: "jaccard",            category: "Token",       emoji: "🎯", description: "Jaccard set/multiset similarity" },
    AlgInfo { name: "overlap",            category: "Token",       emoji: "🎯", description: "Overlap coefficient" },
    AlgInfo { name: "cosine",             category: "Token",       emoji: "🎯", description: "Cosine / Ochiai coefficient" },
    AlgInfo { name: "tanimoto",           category: "Token",       emoji: "🎯", description: "Logarithmic Tanimoto similarity" },
    AlgInfo { name: "sorensen",           category: "Token",       emoji: "🎯", description: "Sorensen-Dice coefficient" },
    AlgInfo { name: "tversky",            category: "Token",       emoji: "🎯", description: "Asymmetric Tversky index" },
    AlgInfo { name: "bag",               category: "Token",       emoji: "🎯", description: "Multiset difference distance" },
    AlgInfo { name: "mra",               category: "Phonetic",    emoji: "🔤", description: "Match Rating Approach" },
    AlgInfo { name: "strcmp95",           category: "Edit",        emoji: "📐", description: "Jaro-Winkler strcmp95 variant" },
    AlgInfo { name: "editex",            category: "Phonetic",    emoji: "🔤", description: "Phonetic-group edit distance" },
    AlgInfo { name: "rle_ncd",           category: "Compression", emoji: "📦", description: "Run-Length Encoding NCD" },
    AlgInfo { name: "arith_ncd",         category: "Compression", emoji: "📦", description: "Arithmetic Coding NCD" },
    AlgInfo { name: "sqrt_ncd",          category: "Compression", emoji: "📦", description: "Square-Root NCD" },
];

fn category_color(cat: &str) -> &'static str {
    match cat {
        "Simple"      => BLUE,
        "Edit"        => GREEN,
        "Alignment"   => MAGENTA,
        "Sequence"    => CYAN,
        "Token"       => YELLOW,
        "Phonetic"    => RED,
        "Compression" => WHITE,
        "Matrix"      => DIM,
        _             => RESET,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Result of running one algorithm
// ─────────────────────────────────────────────────────────────────────────────
struct AlgResult {
    name:       String,
    category:   String,
    emoji:      String,
    similarity: Option<f64>,
    distance:   Option<f64>,
    norm_sim:   Option<f64>,
    norm_dist:  Option<f64>,
    error:      Option<String>,
}

fn compute_alg(name: &str, s1: &str, s2: &str) -> AlgResult {
    let info = ALGORITHMS.iter().find(|a| a.name == name)
        .map(|a| (a.category, a.emoji))
        .unwrap_or(("Unknown", "❓"));

    let s1c = to_char_vec(s1);
    let s2c = to_char_vec(s2);

    macro_rules! from_dist {
        ($m:expr) => {{
            let m = $m;
            AlgResult {
                name: name.to_string(), category: info.0.to_string(), emoji: info.1.to_string(),
                similarity: DistanceMetric::similarity(&m, &s1c, &s2c).ok(),
                distance:   DistanceMetric::distance(&m, &s1c, &s2c).ok(),
                norm_sim:   DistanceMetric::normalized_similarity(&m, &s1c, &s2c).ok(),
                norm_dist:  DistanceMetric::normalized_distance(&m, &s1c, &s2c).ok(),
                error: None,
            }
        }};
    }
    macro_rules! from_sim {
        ($m:expr) => {{
            let m = $m;
            AlgResult {
                name: name.to_string(), category: info.0.to_string(), emoji: info.1.to_string(),
                similarity: SimilarityMetric::similarity(&m, &s1c, &s2c).ok(),
                distance:   SimilarityMetric::distance(&m, &s1c, &s2c).ok(),
                norm_sim:   SimilarityMetric::normalized_similarity(&m, &s1c, &s2c).ok(),
                norm_dist:  SimilarityMetric::normalized_distance(&m, &s1c, &s2c).ok(),
                error: None,
            }
        }};
    }

    match name {
        "identity"            => from_sim!(Identity::new()),
        "length"              => from_dist!(Length::new()),
        "prefix"              => from_sim!(Prefix::new()),
        "postfix"             => from_sim!(Postfix::new()),
        "hamming"             => from_dist!(Hamming::new()),
        "levenshtein"         => from_dist!(Levenshtein::new()),
        "damerau_levenshtein" => from_dist!(DamerauLevenshtein::new()),
        "jaro"                => from_sim!(Jaro::new()),
        "jaro_winkler"        => from_sim!(JaroWinkler::new()),
        "needleman_wunsch"    => from_sim!(NeedlemanWunsch::new()),
        "smith_waterman"      => from_sim!(SmithWaterman::new()),
        "gotoh"               => from_sim!(Gotoh::new()),
        "lcsseq"              => from_sim!(LcsSeq::new()),
        "lcsstr"              => from_sim!(LcsStr::new()),
        "ratcliff_obershelp"  => from_sim!(RatcliffObershelp::new()),
        "mlipns"              => from_sim!(Mlipns::new()),
        "jaccard"             => from_sim!(Jaccard::new()),
        "overlap"             => from_sim!(Overlap::new()),
        "cosine"              => from_sim!(Cosine::new()),
        "tanimoto"            => from_sim!(Tanimoto::new()),
        "sorensen"            => from_sim!(Sorensen::new()),
        "tversky"             => from_sim!(Tversky::new()),
        "bag"                 => from_dist!(Bag::new()),
        "rle_ncd"             => from_sim!(RlenCd::new()),
        "arith_ncd"           => from_sim!(ArithNcd::new()),
        "sqrt_ncd"            => from_sim!(SqrtNcd::new()),
        "matrix"              => {
            let m = Matrix::<char>::new();
            AlgResult { name: name.to_string(), category: info.0.to_string(), emoji: info.1.to_string(),
                similarity: m.similarity(&s1c, &s2c).ok(), distance: m.distance(&s1c, &s2c).ok(),
                norm_sim: m.normalized_similarity(&s1c, &s2c).ok(), norm_dist: m.normalized_distance(&s1c, &s2c).ok(), error: None }
        }
        "mra" => {
            let m = Mra::new();
            AlgResult { name: name.to_string(), category: info.0.to_string(), emoji: info.1.to_string(),
                similarity: m.similarity(s1, s2).ok(), distance: m.distance(s1, s2).ok(),
                norm_sim: m.normalized_similarity(s1, s2).ok(), norm_dist: m.normalized_distance(s1, s2).ok(), error: None }
        }
        "strcmp95" => {
            let m = StrCmp95::new();
            AlgResult { name: name.to_string(), category: info.0.to_string(), emoji: info.1.to_string(),
                similarity: m.similarity(s1, s2).ok(), distance: m.distance(s1, s2).ok(),
                norm_sim: m.normalized_similarity(s1, s2).ok(), norm_dist: m.normalized_distance(s1, s2).ok(), error: None }
        }
        "editex" => {
            let m = Editex::new();
            AlgResult { name: name.to_string(), category: info.0.to_string(), emoji: info.1.to_string(),
                similarity: Some(m.similarity(s1, s2) as f64), distance: Some(m.distance(s1, s2) as f64),
                norm_sim: Some(m.normalized_similarity(s1, s2)), norm_dist: Some(m.normalized_distance(s1, s2)), error: None }
        }
        _ => AlgResult {
            name: name.to_string(), category: info.0.to_string(), emoji: info.1.to_string(),
            similarity: None, distance: None, norm_sim: None, norm_dist: None,
            error: Some(format!("Unknown algorithm '{name}'.\nRun 'tdcli list' to see all available algorithms.")),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Formatting helpers
// ─────────────────────────────────────────────────────────────────────────────
fn fmt_val(v: Option<f64>) -> String {
    match v {
        Some(x) if x.is_nan()              => format!("{DIM}NaN{RESET}"),
        Some(x) if x == f64::INFINITY      => format!("{YELLOW}+∞{RESET}"),
        Some(x) if x == f64::NEG_INFINITY  => format!("{YELLOW}-∞{RESET}"),
        Some(x)                            => format!("{:.6}", x),
        None                               => format!("{DIM}—{RESET}"),
    }
}

fn sim_bar(sim: Option<f64>) -> String {
    let raw = sim.unwrap_or(0.0);
    let v   = raw.max(0.0).min(1.0);
    let filled = (v * 20.0).round() as usize;
    let empty  = 20 - filled;
    let color  = if v > 0.75 { GREEN } else if v > 0.4 { YELLOW } else { RED };
    format!("{color}[{}{DIM}{}{RESET}{color}]{RESET} {:.0}%",
        "█".repeat(filled), "░".repeat(empty), v * 100.0)
}

fn print_banner() {
    println!();
    println!("{BOLD}{CYAN}╔════════════════════════════════════════════════════════╗{RESET}");
    println!("{BOLD}{CYAN}║  {WHITE}📏  textdistancerust  tdcli  {DIM}v0.1.0{RESET}{BOLD}{CYAN}                  ║{RESET}");
    println!("{BOLD}{CYAN}║  {DIM}High-performance Rust text distance toolkit{RESET}{BOLD}{CYAN}          ║{RESET}");
    println!("{BOLD}{CYAN}╚════════════════════════════════════════════════════════╝{RESET}");
    println!();
}

fn print_section(title: &str) {
    println!("{BOLD}{YELLOW}▶ {title}{RESET}");
    println!("{DIM}{}{RESET}", "─".repeat(56));
}

fn print_result_table(results: &[AlgResult]) {
    let w_name = 24usize;
    let w_val  = 11usize;
    // header
    println!("{BOLD}{BG_DARK} {:<w_name$}│ {:<w_val$}│ {:<w_val$}│ {:<w_val$}│ {:<w_val$}│ Visual{RESET}",
        "Algorithm", "Similarity", "Distance", "Norm.Sim", "Norm.Dist");
    println!("{DIM}{}{RESET}", "─".repeat(106));

    for r in results {
        if let Some(ref e) = r.error {
            println!("{RED}  ✗ {} — {e}{RESET}", r.name);
            continue;
        }
        let cc = category_color(&r.category);
        let label = format!("{} {}", r.emoji, r.name);
        println!("{cc}{BOLD}{:<w_name$}{RESET}│ {:<w_val$}│ {:<w_val$}│ {:<w_val$}│ {:<w_val$}│ {}",
            label,
            fmt_val(r.similarity),
            fmt_val(r.distance),
            fmt_val(r.norm_sim),
            fmt_val(r.norm_dist),
            sim_bar(r.norm_sim));
    }
    println!();
}

// ─────────────────────────────────────────────────────────────────────────────
// Commands
// ─────────────────────────────────────────────────────────────────────────────
fn cmd_list() {
    print_banner();
    print_section("Available Algorithms (30 total)");
    let mut last_cat = "";
    for alg in ALGORITHMS {
        if alg.category != last_cat {
            println!();
            let c = category_color(alg.category);
            println!("{BOLD}{c}  ── {} ──{RESET}", alg.category);
            last_cat = alg.category;
        }
        let c = category_color(alg.category);
        println!("  {c}{}{RESET}  {BOLD}{:<24}{RESET} {DIM}{}{RESET}",
            alg.emoji, alg.name, alg.description);
    }
    println!();
    println!("{DIM}─────────────────────────────────────────────────────{RESET}");
    println!("{DIM}Usage examples:{RESET}");
    println!("  {CYAN}tdcli compare --alg hamming \"hello\" \"world\"{RESET}");
    println!("  {CYAN}tdcli compare --alg levenshtein,jaccard,cosine \"kitten\" \"sitting\"{RESET}");
    println!("  {CYAN}tdcli all \"MARTHA\" \"MARHTA\"{RESET}");
    println!("  {CYAN}tdcli bench{RESET}");
    println!("  {CYAN}tdcli interactive{RESET}");
    println!();
}

fn cmd_compare(alg_str: &str, s1: &str, s2: &str) {
    print_banner();
    let algs: Vec<&str> = alg_str.split(',').map(str::trim).collect();
    println!("{BOLD}  Comparing:{RESET}");
    println!("    {CYAN}s1 = \"{s1}\"{RESET}");
    println!("    {CYAN}s2 = \"{s2}\"{RESET}");
    println!("  {BOLD}Algorithm(s):{RESET} {YELLOW}{alg_str}{RESET}");
    println!();
    let results: Vec<AlgResult> = algs.iter().map(|a| compute_alg(a, s1, s2)).collect();
    print_result_table(&results);
}

fn cmd_all(s1: &str, s2: &str) {
    print_banner();
    println!("{BOLD}  All algorithms on:{RESET}");
    println!("    {CYAN}s1 = \"{s1}\"{RESET}");
    println!("    {CYAN}s2 = \"{s2}\"{RESET}");
    println!();

    let mut results: Vec<AlgResult> = ALGORITHMS.iter()
        .map(|a| compute_alg(a.name, s1, s2))
        .collect();

    results.sort_by(|a, b| {
        b.norm_sim.unwrap_or(f64::NEG_INFINITY)
            .partial_cmp(&a.norm_sim.unwrap_or(f64::NEG_INFINITY))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    print_section(&format!("{} algorithms — ranked by normalized similarity", results.len()));
    print_result_table(&results);
}

fn cmd_bench() {
    print_banner();
    print_section("Native In-Process Benchmark (no IPC overhead)");

    let test_pairs = [
        ("hello", "world"),
        ("kitten", "sitting"),
        ("algorithm", "altruism"),
        ("MARTHA", "MARHTA"),
        ("lorem ipsum dolor sit amet", "lorem ipsum dolor sit amet consecutur"),
    ];
    let pairs_c: Vec<(Vec<char>, Vec<char>)> = test_pairs.iter()
        .map(|(a, b)| (to_char_vec(a), to_char_vec(b))).collect();

    const N: usize = 500;
    let total = (N * test_pairs.len()) as f64;

    struct R { name: &'static str, us: f64 }

    macro_rules! td {
        ($name:literal, $m:expr) => {{
            let m = $m; let t = Instant::now();
            for _ in 0..N { for (a,b) in &pairs_c { let _ = DistanceMetric::distance(&m, a, b); } }
            R { name: $name, us: t.elapsed().as_micros() as f64 / total }
        }};
    }
    macro_rules! ts {
        ($name:literal, $m:expr) => {{
            let m = $m; let t = Instant::now();
            for _ in 0..N { for (a,b) in &pairs_c { let _ = SimilarityMetric::similarity(&m, a, b); } }
            R { name: $name, us: t.elapsed().as_micros() as f64 / total }
        }};
    }

    let results = vec![
        ts!("identity",            Identity::new()),
        td!("length",              Length::new()),
        ts!("prefix",              Prefix::new()),
        ts!("postfix",             Postfix::new()),
        td!("hamming",             Hamming::new()),
        td!("levenshtein",         Levenshtein::new()),
        td!("damerau_levenshtein", DamerauLevenshtein::new()),
        ts!("jaro",                Jaro::new()),
        ts!("jaro_winkler",        JaroWinkler::new()),
        ts!("needleman_wunsch",    NeedlemanWunsch::new()),
        ts!("smith_waterman",      SmithWaterman::new()),
        ts!("gotoh",               Gotoh::new()),
        ts!("lcsseq",              LcsSeq::new()),
        ts!("lcsstr",              LcsStr::new()),
        ts!("ratcliff_obershelp",  RatcliffObershelp::new()),
        ts!("mlipns",              Mlipns::new()),
        ts!("jaccard",             Jaccard::new()),
        ts!("overlap",             Overlap::new()),
        ts!("cosine",              Cosine::new()),
        ts!("tanimoto",            Tanimoto::new()),
        ts!("sorensen",            Sorensen::new()),
        ts!("tversky",             Tversky::new()),
        td!("bag",                 Bag::new()),
        ts!("rle_ncd",             RlenCd::new()),
        ts!("arith_ncd",           ArithNcd::new()),
        ts!("sqrt_ncd",            SqrtNcd::new()),
    ];

    let max_us = results.iter().map(|r| r.us).fold(0.0f64, f64::max);

    println!("{BOLD}{BG_DARK}  {:<24} │ {:>12} │ Relative latency{RESET}", "Algorithm", "µs / call");
    println!("{DIM}{}{RESET}", "─".repeat(72));

    for r in &results {
        let bar = ((r.us / max_us) * 30.0).round() as usize;
        let color = if r.us < 1.0 { GREEN } else if r.us < 20.0 { YELLOW } else { RED };
        println!("  {BOLD}{:<24}{RESET} │ {color}{:>12.3}{RESET} µs │ {color}{}{RESET}",
            r.name, r.us, "▊".repeat(bar));
    }
    println!();
    println!("{DIM}  {} iterations × {} pairs per algorithm | direct in-process calls{RESET}",
        N, test_pairs.len());
    println!();
}

fn cmd_interactive() {
    print_banner();
    print_section("Interactive REPL Mode");
    println!("{DIM}  Commands: <algorithm>, 'all', 'list', 'bench', 'quit'{RESET}");
    println!();

    let stdin = io::stdin();
    loop {
        print!("{BOLD}{CYAN}  alg (or all/list/bench/quit) ▶{RESET} ");
        io::stdout().flush().unwrap();
        let mut alg_input = String::new();
        if stdin.read_line(&mut alg_input).unwrap_or(0) == 0 { break; }
        let alg_input = alg_input.trim().to_lowercase();

        match alg_input.as_str() {
            "quit" | "exit" | "q" => break,
            "list"  => { cmd_list(); continue; }
            "bench" => { cmd_bench(); continue; }
            "" => continue,
            _ => {}
        }

        print!("{BOLD}{GREEN}  s1 ▶{RESET} ");
        io::stdout().flush().unwrap();
        let mut s1 = String::new();
        if stdin.read_line(&mut s1).unwrap_or(0) == 0 { break; }
        let s1 = s1.trim_end_matches(['\n', '\r']).to_string();

        print!("{BOLD}{GREEN}  s2 ▶{RESET} ");
        io::stdout().flush().unwrap();
        let mut s2 = String::new();
        if stdin.read_line(&mut s2).unwrap_or(0) == 0 { break; }
        let s2 = s2.trim_end_matches(['\n', '\r']).to_string();

        println!();
        if alg_input == "all" {
            cmd_all(&s1, &s2);
        } else {
            cmd_compare(&alg_input, &s1, &s2);
        }
    }
    println!("{BOLD}{CYAN}  Goodbye! 👋{RESET}");
    println!();
}

fn print_help() {
    print_banner();
    println!("{BOLD}USAGE:{RESET}");
    println!("  {CYAN}tdcli list{RESET}");
    println!("    List all algorithms with categories and descriptions");
    println!();
    println!("  {CYAN}tdcli compare --alg <alg[,alg,...]> <s1> <s2>{RESET}");
    println!("    Compare two strings with specific algorithm(s)");
    println!("    e.g. tdcli compare --alg hamming,jaccard \"hello\" \"world\"");
    println!();
    println!("  {CYAN}tdcli all <s1> <s2>{RESET}");
    println!("    Run ALL algorithms on two strings, sorted by similarity");
    println!();
    println!("  {CYAN}tdcli bench{RESET}");
    println!("    In-process latency benchmark for all algorithms");
    println!();
    println!("  {CYAN}tdcli interactive{RESET}");
    println!("    Enter interactive REPL mode");
    println!();
}

// ─────────────────────────────────────────────────────────────────────────────
// MAIN
// ─────────────────────────────────────────────────────────────────────────────
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "list" => cmd_list(),
        "bench" => cmd_bench(),
        "interactive" | "repl" => cmd_interactive(),
        "help" | "--help" | "-h" => print_help(),
        "all" => {
            if args.len() < 4 {
                eprintln!("{RED}Usage: tdcli all <s1> <s2>{RESET}");
                std::process::exit(1);
            }
            cmd_all(&args[2], &args[3]);
        }
        "compare" => {
            // parse: compare [--alg <algs>] <s1> <s2>
            let mut alg = "levenshtein".to_string();
            let mut pos_args: Vec<&str> = Vec::new();
            let mut i = 2usize;
            while i < args.len() {
                if args[i] == "--alg" || args[i] == "-a" {
                    i += 1;
                    if i < args.len() { alg = args[i].clone(); }
                } else {
                    pos_args.push(&args[i]);
                }
                i += 1;
            }
            if pos_args.len() < 2 {
                eprintln!("{RED}Usage: tdcli compare [--alg <alg>] <s1> <s2>{RESET}");
                std::process::exit(1);
            }
            cmd_compare(&alg, pos_args[0], pos_args[1]);
        }
        unknown => {
            eprintln!("{RED}Unknown command '{unknown}'. Run 'tdcli help' for usage.{RESET}");
            std::process::exit(1);
        }
    }
}
