use std::time::Instant;
use textdistancerust::*;

fn main() {
    let test_pairs = vec![
        ("hello", "world"),
        ("subsequence", "subsequence"),
        ("distance", "difference"),
        ("algorithm", "altruism"),
        ("lorem ipsum dolor sit amet", "lorem ipsum dolor sit amet con"),
        ("MARTHA", "MARHTA"),
        ("shackleford", "shackelford"),
    ];

    let test_pairs_chars: Vec<(Vec<char>, Vec<char>)> = test_pairs
        .iter()
        .map(|(a, b)| (to_char_vec(a), to_char_vec(b)))
        .collect();

    let n_runs = 2000usize;
    let total_calls = (n_runs * test_pairs.len()) as f64;

    let identity = Identity::new();
    let length = Length::new();
    let prefix = Prefix::new();
    let postfix = Postfix::new();
    let matrix = Matrix::<char>::new();
    let jaccard = Jaccard::new();
    let overlap = Overlap::new();
    let cosine = Cosine::new();
    let tanimoto = Tanimoto::new();
    let sorensen = Sorensen::new();
    let tversky = Tversky::new();
    let bag = Bag::new();
    let mra = Mra::new();
    let strcmp95 = StrCmp95::new();
    let editex = Editex::new();
    let hamming = Hamming::new();
    let dl = DamerauLevenshtein::new();
    let rle = RlenCd::new();
    let arith = ArithNcd::new();
    let sqrt = SqrtNcd::new();

    println!("ALG_START");

    // Identity
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs_chars {
            let _ = identity.similarity(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("identity:{:.4}", dur);

    // Length
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs_chars {
            let _ = length.distance(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("length:{:.4}", dur);

    // Prefix
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs_chars {
            let _ = prefix.similarity(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("prefix:{:.4}", dur);

    // Postfix
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs_chars {
            let _ = postfix.similarity(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("postfix:{:.4}", dur);

    // Matrix
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs_chars {
            let _ = matrix.similarity(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("matrix:{:.4}", dur);

    // Jaccard
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs_chars {
            let _ = jaccard.similarity(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("jaccard:{:.4}", dur);

    // Overlap
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs_chars {
            let _ = overlap.similarity(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("overlap:{:.4}", dur);

    // Cosine
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs_chars {
            let _ = cosine.similarity(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("cosine:{:.4}", dur);

    // Tanimoto
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs_chars {
            let _ = tanimoto.similarity(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("tanimoto:{:.4}", dur);

    // Sorensen
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs_chars {
            let _ = sorensen.similarity(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("sorensen:{:.4}", dur);

    // Tversky
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs_chars {
            let _ = tversky.similarity(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("tversky:{:.4}", dur);

    // Bag
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs_chars {
            let _ = bag.distance(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("bag:{:.4}", dur);

    // MRA
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs {
            let _ = mra.distance(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("mra:{:.4}", dur);

    // StrCmp95
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs {
            let _ = strcmp95.similarity(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("strcmp95:{:.4}", dur);

    // Editex
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs {
            let _ = editex.distance(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("editex:{:.4}", dur);

    // Hamming
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs_chars {
            let _ = hamming.distance(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("hamming:{:.4}", dur);

    // DamerauLevenshtein
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs_chars {
            let _ = DistanceMetric::distance(&dl, s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("damerau_levenshtein:{:.4}", dur);

    // RLENCD
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs_chars {
            let _ = rle.similarity(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("rle_ncd:{:.4}", dur);

    // ArithNCD
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs_chars {
            let _ = arith.similarity(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("arith_ncd:{:.4}", dur);

    // SqrtNCD
    let start = Instant::now();
    for _ in 0..n_runs {
        for (s1, s2) in &test_pairs_chars {
            let _ = sqrt.similarity(s1, s2);
        }
    }
    let dur = start.elapsed().as_secs_f64() * 1e6 / total_calls;
    println!("sqrt_ncd:{:.4}", dur);
}
