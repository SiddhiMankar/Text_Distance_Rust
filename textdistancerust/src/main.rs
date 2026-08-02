use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use textdistancerust::*;

#[derive(Deserialize)]
struct FuzzRequest {
    alg: String,
    s1: String,
    s2: String,
    mat: Option<Vec<(String, String, f64)>>,
    qval: Option<usize>,
    as_set: Option<bool>,
    alpha: Option<f64>,
    beta: Option<f64>,
    bias: Option<f64>,
}

#[derive(Serialize)]
struct FuzzResponse {
    similarity: Option<f64>,
    distance: Option<f64>,
    normalized_similarity: Option<f64>,
    normalized_distance: Option<f64>,
    subsequence: Option<String>,
    error: Option<String>,
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let req: FuzzRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = FuzzResponse {
                    similarity: None,
                    distance: None,
                    normalized_similarity: None,
                    normalized_distance: None,
                    subsequence: None,
                    error: Some(format!("JSON parse error: {}", e)),
                };
                serde_json::to_writer(&mut handle, &resp)?;
                handle.write_all(b"\n")?;
                handle.flush()?;
                continue;
            }
        };

        let resp = process_request(&req);
        serde_json::to_writer(&mut handle, &resp)?;
        handle.write_all(b"\n")?;
        handle.flush()?;
    }

    Ok(())
}

fn process_request(req: &FuzzRequest) -> FuzzResponse {
    let s1_chars = to_char_vec(&req.s1);
    let s2_chars = to_char_vec(&req.s2);

    match req.alg.as_str() {
        "identity" => {
            let ident = Identity::new();
            match (
                ident.similarity(&s1_chars, &s2_chars),
                ident.distance(&s1_chars, &s2_chars),
                ident.normalized_similarity(&s1_chars, &s2_chars),
                ident.normalized_distance(&s1_chars, &s2_chars),
            ) {
                (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                    similarity: Some(sim),
                    distance: Some(dist),
                    normalized_similarity: Some(norm_sim),
                    normalized_distance: Some(norm_dist),
                    subsequence: None,
                    error: None,
                },
                _ => FuzzResponse {
                    similarity: None,
                    distance: None,
                    normalized_similarity: None,
                    normalized_distance: None,
                    subsequence: None,
                    error: Some("Calculation failed".to_string()),
                },
            }
        }
        "length" => {
            let len_metric = Length::new();
            match (
                len_metric.distance(&s1_chars, &s2_chars),
                len_metric.similarity(&s1_chars, &s2_chars),
                len_metric.normalized_distance(&s1_chars, &s2_chars),
                len_metric.normalized_similarity(&s1_chars, &s2_chars),
            ) {
                (Ok(dist), Ok(sim), Ok(norm_dist), Ok(norm_sim)) => FuzzResponse {
                    similarity: Some(sim),
                    distance: Some(dist),
                    normalized_similarity: Some(norm_sim),
                    normalized_distance: Some(norm_sim),
                    subsequence: None,
                    error: None,
                },
                _ => FuzzResponse {
                    similarity: None,
                    distance: None,
                    normalized_similarity: None,
                    normalized_distance: None,
                    subsequence: None,
                    error: Some("Calculation failed".to_string()),
                },
            }
        }
        "prefix" => {
            let pref_metric = Prefix::new();
            let subseq: String = pref_metric.prefix(&s1_chars, &s2_chars).iter().collect();
            match (
                pref_metric.similarity(&s1_chars, &s2_chars),
                pref_metric.distance(&s1_chars, &s2_chars),
                pref_metric.normalized_similarity(&s1_chars, &s2_chars),
                pref_metric.normalized_distance(&s1_chars, &s2_chars),
            ) {
                (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                    similarity: Some(sim),
                    distance: Some(dist),
                    normalized_similarity: Some(norm_sim),
                    normalized_distance: Some(norm_dist),
                    subsequence: Some(subseq),
                    error: None,
                },
                _ => FuzzResponse {
                    similarity: None,
                    distance: None,
                    normalized_similarity: None,
                    normalized_distance: None,
                    subsequence: None,
                    error: Some("Calculation failed".to_string()),
                },
            }
        }
        "postfix" => {
            let post_metric = Postfix::new();
            let subseq: String = post_metric.postfix(&s1_chars, &s2_chars).iter().collect();
            match (
                post_metric.similarity(&s1_chars, &s2_chars),
                post_metric.distance(&s1_chars, &s2_chars),
                post_metric.normalized_similarity(&s1_chars, &s2_chars),
                post_metric.normalized_distance(&s1_chars, &s2_chars),
            ) {
                (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                    similarity: Some(sim),
                    distance: Some(dist),
                    normalized_similarity: Some(norm_sim),
                    normalized_distance: Some(norm_dist),
                    subsequence: Some(subseq),
                    error: None,
                },
                _ => FuzzResponse {
                    similarity: None,
                    distance: None,
                    normalized_similarity: None,
                    normalized_distance: None,
                    subsequence: None,
                    error: Some("Calculation failed".to_string()),
                },
            }
        }
        "matrix" => {
            if let Some(ref raw_mat) = req.mat {
                let map: HashMap<(String, String), f64> = raw_mat
                    .iter()
                    .map(|(a, b, v)| ((a.clone(), b.clone()), *v))
                    .collect();
                let mat_metric = Matrix::with_config(Some(map), 1.0, 0.0, true);
                let s1_str = vec![req.s1.clone()];
                let s2_str = vec![req.s2.clone()];
                match (
                    mat_metric.similarity(&s1_str, &s2_str),
                    mat_metric.distance(&s1_str, &s2_str),
                    mat_metric.normalized_similarity(&s1_str, &s2_str),
                    mat_metric.normalized_distance(&s1_str, &s2_str),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            } else {
                let mat_metric = Matrix::<char>::new();
                match (
                    mat_metric.similarity(&s1_chars, &s2_chars),
                    mat_metric.distance(&s1_chars, &s2_chars),
                    mat_metric.normalized_similarity(&s1_chars, &s2_chars),
                    mat_metric.normalized_distance(&s1_chars, &s2_chars),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            }
        }
        "jaccard" => {
            let qval = req.qval.unwrap_or(1);
            let as_set = req.as_set.unwrap_or(false);
            let jaccard_metric = Jaccard::with_config(qval, as_set);

            if req.s1 == req.s2 {
                return FuzzResponse {
                    similarity: Some(1.0),
                    distance: Some(0.0),
                    normalized_similarity: Some(1.0),
                    normalized_distance: Some(0.0),
                    subsequence: None,
                    error: None,
                };
            }

            if req.s1.is_empty() || req.s2.is_empty() {
                return FuzzResponse {
                    similarity: Some(0.0),
                    distance: Some(1.0),
                    normalized_similarity: Some(0.0),
                    normalized_distance: Some(1.0),
                    subsequence: None,
                    error: None,
                };
            }

            if qval == 0 {
                let s1_words = to_word_vec(&req.s1);
                let s2_words = to_word_vec(&req.s2);
                match (
                    jaccard_metric.similarity(&s1_words, &s2_words),
                    jaccard_metric.distance(&s1_words, &s2_words),
                    jaccard_metric.normalized_similarity(&s1_words, &s2_words),
                    jaccard_metric.normalized_distance(&s1_words, &s2_words),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            } else if qval > 1 {
                let s1_ngrams = find_ngrams(&s1_chars, qval);
                let s2_ngrams = find_ngrams(&s2_chars, qval);
                match (
                    jaccard_metric.similarity(&s1_ngrams, &s2_ngrams),
                    jaccard_metric.distance(&s1_ngrams, &s2_ngrams),
                    jaccard_metric.normalized_similarity(&s1_ngrams, &s2_ngrams),
                    jaccard_metric.normalized_distance(&s1_ngrams, &s2_ngrams),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            } else {
                match (
                    jaccard_metric.similarity(&s1_chars, &s2_chars),
                    jaccard_metric.distance(&s1_chars, &s2_chars),
                    jaccard_metric.normalized_similarity(&s1_chars, &s2_chars),
                    jaccard_metric.normalized_distance(&s1_chars, &s2_chars),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            }
        }
        "overlap" => {
            let qval = req.qval.unwrap_or(1);
            let as_set = req.as_set.unwrap_or(false);
            let overlap_metric = Overlap::with_config(qval, as_set);

            if req.s1 == req.s2 {
                return FuzzResponse {
                    similarity: Some(1.0),
                    distance: Some(0.0),
                    normalized_similarity: Some(1.0),
                    normalized_distance: Some(0.0),
                    subsequence: None,
                    error: None,
                };
            }

            if req.s1.is_empty() || req.s2.is_empty() {
                return FuzzResponse {
                    similarity: Some(0.0),
                    distance: Some(1.0),
                    normalized_similarity: Some(0.0),
                    normalized_distance: Some(1.0),
                    subsequence: None,
                    error: None,
                };
            }

            if qval == 0 {
                let s1_words = to_word_vec(&req.s1);
                let s2_words = to_word_vec(&req.s2);
                match (
                    overlap_metric.similarity(&s1_words, &s2_words),
                    overlap_metric.distance(&s1_words, &s2_words),
                    overlap_metric.normalized_similarity(&s1_words, &s2_words),
                    overlap_metric.normalized_distance(&s1_words, &s2_words),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            } else if qval > 1 {
                let s1_ngrams = find_ngrams(&s1_chars, qval);
                let s2_ngrams = find_ngrams(&s2_chars, qval);
                match (
                    overlap_metric.similarity(&s1_ngrams, &s2_ngrams),
                    overlap_metric.distance(&s1_ngrams, &s2_ngrams),
                    overlap_metric.normalized_similarity(&s1_ngrams, &s2_ngrams),
                    overlap_metric.normalized_distance(&s1_ngrams, &s2_ngrams),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            } else {
                match (
                    overlap_metric.similarity(&s1_chars, &s2_chars),
                    overlap_metric.distance(&s1_chars, &s2_chars),
                    overlap_metric.normalized_similarity(&s1_chars, &s2_chars),
                    overlap_metric.normalized_distance(&s1_chars, &s2_chars),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            }
        }
        "cosine" => {
            let qval = req.qval.unwrap_or(1);
            let as_set = req.as_set.unwrap_or(false);
            let cosine_metric = Cosine::with_config(qval, as_set);

            if req.s1 == req.s2 {
                return FuzzResponse {
                    similarity: Some(1.0),
                    distance: Some(0.0),
                    normalized_similarity: Some(1.0),
                    normalized_distance: Some(0.0),
                    subsequence: None,
                    error: None,
                };
            }

            if req.s1.is_empty() || req.s2.is_empty() {
                return FuzzResponse {
                    similarity: Some(0.0),
                    distance: Some(1.0),
                    normalized_similarity: Some(0.0),
                    normalized_distance: Some(1.0),
                    subsequence: None,
                    error: None,
                };
            }

            if qval == 0 {
                let s1_words = to_word_vec(&req.s1);
                let s2_words = to_word_vec(&req.s2);
                match (
                    cosine_metric.similarity(&s1_words, &s2_words),
                    cosine_metric.distance(&s1_words, &s2_words),
                    cosine_metric.normalized_similarity(&s1_words, &s2_words),
                    cosine_metric.normalized_distance(&s1_words, &s2_words),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            } else if qval > 1 {
                let s1_ngrams = find_ngrams(&s1_chars, qval);
                let s2_ngrams = find_ngrams(&s2_chars, qval);
                match (
                    cosine_metric.similarity(&s1_ngrams, &s2_ngrams),
                    cosine_metric.distance(&s1_ngrams, &s2_ngrams),
                    cosine_metric.normalized_similarity(&s1_ngrams, &s2_ngrams),
                    cosine_metric.normalized_distance(&s1_ngrams, &s2_ngrams),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            } else {
                match (
                    cosine_metric.similarity(&s1_chars, &s2_chars),
                    cosine_metric.distance(&s1_chars, &s2_chars),
                    cosine_metric.normalized_similarity(&s1_chars, &s2_chars),
                    cosine_metric.normalized_distance(&s1_chars, &s2_chars),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            }
        }
        "tanimoto" => {
            let qval = req.qval.unwrap_or(1);
            let as_set = req.as_set.unwrap_or(false);
            let tanimoto_metric = Tanimoto::with_config(qval, as_set);

            if req.s1 == req.s2 {
                return FuzzResponse {
                    similarity: Some(0.0),
                    distance: Some(1.0),
                    normalized_similarity: Some(0.0),
                    normalized_distance: Some(1.0),
                    subsequence: None,
                    error: None,
                };
            }

            if req.s1.is_empty() || req.s2.is_empty() {
                return FuzzResponse {
                    similarity: Some(f64::NEG_INFINITY),
                    distance: Some(f64::INFINITY),
                    normalized_similarity: Some(f64::NEG_INFINITY),
                    normalized_distance: Some(f64::INFINITY),
                    subsequence: None,
                    error: None,
                };
            }

            if qval == 0 {
                let s1_words = to_word_vec(&req.s1);
                let s2_words = to_word_vec(&req.s2);
                match (
                    tanimoto_metric.similarity(&s1_words, &s2_words),
                    tanimoto_metric.distance(&s1_words, &s2_words),
                    tanimoto_metric.normalized_similarity(&s1_words, &s2_words),
                    tanimoto_metric.normalized_distance(&s1_words, &s2_words),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            } else if qval > 1 {
                let s1_ngrams = find_ngrams(&s1_chars, qval);
                let s2_ngrams = find_ngrams(&s2_chars, qval);
                match (
                    tanimoto_metric.similarity(&s1_ngrams, &s2_ngrams),
                    tanimoto_metric.distance(&s1_ngrams, &s2_ngrams),
                    tanimoto_metric.normalized_similarity(&s1_ngrams, &s2_ngrams),
                    tanimoto_metric.normalized_distance(&s1_ngrams, &s2_ngrams),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            } else {
                match (
                    tanimoto_metric.similarity(&s1_chars, &s2_chars),
                    tanimoto_metric.distance(&s1_chars, &s2_chars),
                    tanimoto_metric.normalized_similarity(&s1_chars, &s2_chars),
                    tanimoto_metric.normalized_distance(&s1_chars, &s2_chars),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            }
        }
        "sorensen" => {
            let qval = req.qval.unwrap_or(1);
            let as_set = req.as_set.unwrap_or(false);
            let sorensen_metric = Sorensen::with_config(qval, as_set);

            if req.s1 == req.s2 {
                return FuzzResponse {
                    similarity: Some(1.0),
                    distance: Some(0.0),
                    normalized_similarity: Some(1.0),
                    normalized_distance: Some(0.0),
                    subsequence: None,
                    error: None,
                };
            }

            if req.s1.is_empty() || req.s2.is_empty() {
                return FuzzResponse {
                    similarity: Some(0.0),
                    distance: Some(1.0),
                    normalized_similarity: Some(0.0),
                    normalized_distance: Some(1.0),
                    subsequence: None,
                    error: None,
                };
            }

            if qval == 0 {
                let s1_words = to_word_vec(&req.s1);
                let s2_words = to_word_vec(&req.s2);
                match (
                    sorensen_metric.similarity(&s1_words, &s2_words),
                    sorensen_metric.distance(&s1_words, &s2_words),
                    sorensen_metric.normalized_similarity(&s1_words, &s2_words),
                    sorensen_metric.normalized_distance(&s1_words, &s2_words),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            } else if qval > 1 {
                let s1_ngrams = find_ngrams(&s1_chars, qval);
                let s2_ngrams = find_ngrams(&s2_chars, qval);
                match (
                    sorensen_metric.similarity(&s1_ngrams, &s2_ngrams),
                    sorensen_metric.distance(&s1_ngrams, &s2_ngrams),
                    sorensen_metric.normalized_similarity(&s1_ngrams, &s2_ngrams),
                    sorensen_metric.normalized_distance(&s1_ngrams, &s2_ngrams),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            } else {
                match (
                    sorensen_metric.similarity(&s1_chars, &s2_chars),
                    sorensen_metric.distance(&s1_chars, &s2_chars),
                    sorensen_metric.normalized_similarity(&s1_chars, &s2_chars),
                    sorensen_metric.normalized_distance(&s1_chars, &s2_chars),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            }
        }
        "tversky" => {
            let alpha = req.alpha.unwrap_or(1.0);
            let beta = req.beta.unwrap_or(1.0);
            let bias = req.bias;
            let qval = req.qval.unwrap_or(1);
            let as_set = req.as_set.unwrap_or(false);
            let tversky_metric = Tversky::with_config(alpha, beta, bias, qval, as_set);

            if req.s1 == req.s2 {
                return FuzzResponse {
                    similarity: Some(1.0),
                    distance: Some(0.0),
                    normalized_similarity: Some(1.0),
                    normalized_distance: Some(0.0),
                    subsequence: None,
                    error: None,
                };
            }

            if req.s1.is_empty() || req.s2.is_empty() {
                return FuzzResponse {
                    similarity: Some(0.0),
                    distance: Some(1.0),
                    normalized_similarity: Some(0.0),
                    normalized_distance: Some(1.0),
                    subsequence: None,
                    error: None,
                };
            }

            if qval == 0 {
                let s1_words = to_word_vec(&req.s1);
                let s2_words = to_word_vec(&req.s2);
                match (
                    tversky_metric.similarity(&s1_words, &s2_words),
                    tversky_metric.distance(&s1_words, &s2_words),
                    tversky_metric.normalized_similarity(&s1_words, &s2_words),
                    tversky_metric.normalized_distance(&s1_words, &s2_words),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            } else if qval > 1 {
                let s1_ngrams = find_ngrams(&s1_chars, qval);
                let s2_ngrams = find_ngrams(&s2_chars, qval);
                match (
                    tversky_metric.similarity(&s1_ngrams, &s2_ngrams),
                    tversky_metric.distance(&s1_ngrams, &s2_ngrams),
                    tversky_metric.normalized_similarity(&s1_ngrams, &s2_ngrams),
                    tversky_metric.normalized_distance(&s1_ngrams, &s2_ngrams),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            } else {
                match (
                    tversky_metric.similarity(&s1_chars, &s2_chars),
                    tversky_metric.distance(&s1_chars, &s2_chars),
                    tversky_metric.normalized_similarity(&s1_chars, &s2_chars),
                    tversky_metric.normalized_distance(&s1_chars, &s2_chars),
                ) {
                    (Ok(sim), Ok(dist), Ok(norm_sim), Ok(norm_dist)) => FuzzResponse {
                        similarity: Some(sim),
                        distance: Some(dist),
                        normalized_similarity: Some(norm_sim),
                        normalized_distance: Some(norm_dist),
                        subsequence: None,
                        error: None,
                    },
                    _ => FuzzResponse {
                        similarity: None,
                        distance: None,
                        normalized_similarity: None,
                        normalized_distance: None,
                        subsequence: None,
                        error: Some("Calculation failed".to_string()),
                    },
                }
            }
        }
        _ => FuzzResponse {
            similarity: None,
            distance: None,
            normalized_similarity: None,
            normalized_distance: None,
            subsequence: None,
            error: Some(format!("Unknown algorithm: {}", req.alg)),
        },
    }
}
