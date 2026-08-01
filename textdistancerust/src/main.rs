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
