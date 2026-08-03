// Integration tests for all textdistancerust algorithms
// Tests known values with assertions to 1e-7 tolerance.

#[cfg(test)]
mod integration {
    use textdistancerust::*;
    use textdistancerust::{DistanceMetric, SimilarityMetric};

    const TOL: f64 = 1e-7;

    fn assert_close(label: &str, expected: f64, actual: f64) {
        let diff = (expected - actual).abs();
        assert!(
            diff <= TOL || (expected.is_nan() && actual.is_nan()),
            "{label}: expected {expected:.10}, got {actual:.10} (diff={diff:.2e})"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Identity
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_identity_integration() {
        let m = Identity::new();
        let cv = |s: &str| to_char_vec(s);

        // identical
        assert_close("identity/identical/sim",   1.0, m.similarity(&cv("hello"), &cv("hello")).unwrap());
        assert_close("identity/identical/dist",  0.0, m.distance(&cv("hello"), &cv("hello")).unwrap());
        assert_close("identity/identical/nsim",  1.0, m.normalized_similarity(&cv("hello"), &cv("hello")).unwrap());
        assert_close("identity/identical/ndist", 0.0, m.normalized_distance(&cv("hello"), &cv("hello")).unwrap());

        // different
        assert_close("identity/diff/sim",   0.0, m.similarity(&cv("abc"), &cv("xyz")).unwrap());
        assert_close("identity/diff/dist",  1.0, m.distance(&cv("abc"), &cv("xyz")).unwrap());
        assert_close("identity/diff/nsim",  0.0, m.normalized_similarity(&cv("abc"), &cv("xyz")).unwrap());
        assert_close("identity/diff/ndist", 1.0, m.normalized_distance(&cv("abc"), &cv("xyz")).unwrap());

        // empty vs empty
        assert_close("identity/empty/sim",   1.0, m.similarity(&cv(""), &cv("")).unwrap());
        assert_close("identity/empty/nsim",  1.0, m.normalized_similarity(&cv(""), &cv("")).unwrap());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Hamming
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_hamming_integration() {
        let m = Hamming::new();
        let cv = |s: &str| to_char_vec(s);

        // "karolin" vs "kathrin" — 3 mismatches
        assert_close("hamming/karolin_kathrin/dist", 3.0,
            m.distance(&cv("karolin"), &cv("kathrin")).unwrap());
        assert_close("hamming/karolin_kathrin/sim", 4.0,
            m.similarity(&cv("karolin"), &cv("kathrin")).unwrap());
        assert_close("hamming/karolin_kathrin/nsim", 4.0 / 7.0,
            m.normalized_similarity(&cv("karolin"), &cv("kathrin")).unwrap());

        // identical
        assert_close("hamming/identical/dist", 0.0,
            m.distance(&cv("abcde"), &cv("abcde")).unwrap());
        assert_close("hamming/identical/nsim", 1.0,
            m.normalized_similarity(&cv("abcde"), &cv("abcde")).unwrap());

        // empty vs empty
        assert_close("hamming/empty/dist", 0.0,
            m.distance(&cv(""), &cv("")).unwrap());
        assert_close("hamming/empty/nsim", 1.0,
            m.normalized_similarity(&cv(""), &cv("")).unwrap());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Levenshtein
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_levenshtein_integration() {
        let m = Levenshtein::new();
        let cv = |s: &str| to_char_vec(s);

        // "kitten" -> "sitting" = 3 ops
        assert_close("lev/kitten_sitting/dist", 3.0,
            m.distance(&cv("kitten"), &cv("sitting")).unwrap());

        // identical
        assert_close("lev/identical/dist", 0.0,
            m.distance(&cv("rust"), &cv("rust")).unwrap());
        assert_close("lev/identical/nsim", 1.0,
            m.normalized_similarity(&cv("rust"), &cv("rust")).unwrap());

        // empty vs non-empty
        assert_close("lev/empty_hello/dist", 5.0,
            m.distance(&cv(""), &cv("hello")).unwrap());

        // completely different (same length)
        assert_close("lev/abc_xyz/dist", 3.0,
            m.distance(&cv("abc"), &cv("xyz")).unwrap());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // DamerauLevenshtein
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_damerau_levenshtein_integration() {
        let m = DamerauLevenshtein::new();
        let cv = |s: &str| to_char_vec(s);

        // transposition counts as 1
        assert_close("dl/ca_ac/dist", 1.0,
            DistanceMetric::distance(&m, &cv("ca"), &cv("ac")).unwrap());

        assert_close("dl/identical/dist", 0.0,
            DistanceMetric::distance(&m, &cv("algorithm"), &cv("algorithm")).unwrap());

        assert_close("dl/empty_empty/dist", 0.0,
            DistanceMetric::distance(&m, &cv(""), &cv("")).unwrap());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Jaro
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_jaro_integration() {
        let m = Jaro::new();
        let cv = |s: &str| to_char_vec(s);

        // "MARTHA" vs "MARHTA" = 0.9444...
        let sim = m.similarity(&cv("MARTHA"), &cv("MARHTA")).unwrap();
        assert_close("jaro/MARTHA_MARHTA/sim", 0.9444444444444445, sim);

        // identical
        assert_close("jaro/identical/sim", 1.0,
            m.similarity(&cv("hello"), &cv("hello")).unwrap());

        // empty vs empty
        let s = m.similarity(&cv(""), &cv("")).unwrap();
        assert_close("jaro/empty/sim", 1.0, s);

        // completely different
        let s2 = m.similarity(&cv("abc"), &cv("xyz")).unwrap();
        assert_close("jaro/disjoint/sim", 0.0, s2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // JaroWinkler
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_jaro_winkler_integration() {
        let m = JaroWinkler::new();
        let cv = |s: &str| to_char_vec(s);

        // "MARTHA" vs "MARHTA"
        let sim = m.similarity(&cv("MARTHA"), &cv("MARHTA")).unwrap();
        assert_close("jw/MARTHA_MARHTA/sim", 0.9611111111111111, sim);

        // identical
        assert_close("jw/identical/sim", 1.0,
            m.similarity(&cv("test"), &cv("test")).unwrap());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // LcsSeq
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_lcsseq_integration() {
        let m = LcsSeq::new();
        let cv = |s: &str| to_char_vec(s);

        // "ABCBDAB" vs "BDCAB" — LCS=4 ("BCAB" or "BDAB")
        assert_close("lcsseq/abcbdab_bdcab/sim", 4.0,
            m.similarity(&cv("ABCBDAB"), &cv("BDCAB")).unwrap());

        assert_close("lcsseq/identical/nsim", 1.0,
            m.normalized_similarity(&cv("hello"), &cv("hello")).unwrap());

        assert_close("lcsseq/empty/sim", 0.0,
            m.similarity(&cv(""), &cv("abc")).unwrap());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // LcsStr
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_lcsstr_integration() {
        let m = LcsStr::new();
        let cv = |s: &str| to_char_vec(s);

        // "ABABC" vs "BABCAB" — longest common substring = "BABC" (len 4)
        assert_close("lcsstr/ababc_babcab/sim", 4.0,
            m.similarity(&cv("ABABC"), &cv("BABCAB")).unwrap());

        assert_close("lcsstr/identical/nsim", 1.0,
            m.normalized_similarity(&cv("rust"), &cv("rust")).unwrap());

        assert_close("lcsstr/disjoint/sim", 0.0,
            m.similarity(&cv("abc"), &cv("xyz")).unwrap());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // RatcliffObershelp
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_ratcliff_obershelp_integration() {
        let m = RatcliffObershelp::new();
        let cv = |s: &str| to_char_vec(s);

        // "MARTHA" vs "MARHTA"
        let sim = m.normalized_similarity(&cv("MARTHA"), &cv("MARHTA")).unwrap();
        assert!(sim > 0.8, "ratcliff/MARTHA_MARHTA: expected > 0.8, got {sim}");

        assert_close("ratcliff/identical/nsim", 1.0,
            m.normalized_similarity(&cv("algorithm"), &cv("algorithm")).unwrap());

        let s = m.normalized_similarity(&cv(""), &cv("")).unwrap();
        assert_close("ratcliff/empty/nsim", 1.0, s);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Jaccard
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_jaccard_integration() {
        let m = Jaccard::new();
        let cv = |s: &str| to_char_vec(s);

        assert_close("jaccard/identical/nsim", 1.0,
            m.normalized_similarity(&cv("hello"), &cv("hello")).unwrap());

        assert_close("jaccard/disjoint/nsim", 0.0,
            m.normalized_similarity(&cv("abc"), &cv("xyz")).unwrap());

        // "cat" vs "hat" — char bigrams: ca,at vs ha,at — intersection={at}=1, union={ca,at,ha}=3
        let s = m.similarity(&to_char_vec("cat"), &to_char_vec("hat")).unwrap();
        assert!(s >= 0.0 && s <= 1.0, "jaccard/cat_hat should be in [0,1], got {s}");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Cosine
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_cosine_integration() {
        let m = Cosine::new();
        let cv = |s: &str| to_char_vec(s);

        assert_close("cosine/identical/nsim", 1.0,
            m.normalized_similarity(&cv("rust"), &cv("rust")).unwrap());

        assert_close("cosine/disjoint/nsim", 0.0,
            m.normalized_similarity(&cv("abc"), &cv("xyz")).unwrap());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Sorensen
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_sorensen_integration() {
        let m = Sorensen::new();
        let cv = |s: &str| to_char_vec(s);

        assert_close("sorensen/identical/nsim", 1.0,
            m.normalized_similarity(&cv("hello"), &cv("hello")).unwrap());

        assert_close("sorensen/disjoint/nsim", 0.0,
            m.normalized_similarity(&cv("abc"), &cv("xyz")).unwrap());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Overlap
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_overlap_integration() {
        let m = Overlap::new();
        let cv = |s: &str| to_char_vec(s);

        assert_close("overlap/identical/nsim", 1.0,
            m.normalized_similarity(&cv("test"), &cv("test")).unwrap());

        let s = m.normalized_similarity(&cv("abc"), &cv("xyz")).unwrap();
        assert!(s >= 0.0, "overlap/disjoint should be >= 0");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Tanimoto
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_tanimoto_integration() {
        let m = Tanimoto::new();
        let cv = |s: &str| to_char_vec(s);

        // identical → similarity = 0.0 (log(1)=0)
        let s = m.similarity(&cv("hello"), &cv("hello")).unwrap();
        assert_close("tanimoto/identical/sim", 0.0, s);

        // disjoint → -∞
        let s_disj = m.similarity(&cv("abc"), &cv("xyz")).unwrap();
        assert!(s_disj.is_infinite() && s_disj < 0.0,
            "tanimoto/disjoint should be -∞, got {s_disj}");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // NeedlemanWunsch
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_needleman_wunsch_integration() {
        let m = NeedlemanWunsch::new();
        let cv = |s: &str| to_char_vec(s);

        // identical should give maximum similarity
        let s = m.normalized_similarity(&cv("ACGT"), &cv("ACGT")).unwrap();
        assert_close("nw/identical/nsim", 1.0, s);

        // empty vs empty
        let s2 = m.normalized_similarity(&cv(""), &cv("")).unwrap();
        assert_close("nw/empty/nsim", 1.0, s2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SmithWaterman
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_smith_waterman_integration() {
        let m = SmithWaterman::new();
        let cv = |s: &str| to_char_vec(s);

        let s = m.normalized_similarity(&cv("ACGT"), &cv("ACGT")).unwrap();
        assert_close("sw/identical/nsim", 1.0, s);

        let s2 = m.normalized_similarity(&cv(""), &cv("")).unwrap();
        assert_close("sw/empty/nsim", 1.0, s2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Gotoh
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_gotoh_integration() {
        let m = Gotoh::new();
        let cv = |s: &str| to_char_vec(s);

        let s = m.normalized_similarity(&cv("ACGT"), &cv("ACGT")).unwrap();
        assert_close("gotoh/identical/nsim", 1.0, s);

        let s2 = m.normalized_similarity(&cv(""), &cv("")).unwrap();
        assert_close("gotoh/empty/nsim", 1.0, s2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Mlipns
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_mlipns_integration() {
        let m = Mlipns::new();
        let cv = |s: &str| to_char_vec(s);

        assert_close("mlipns/identical/dist", 0.0,
            m.distance(&cv("hello"), &cv("hello")).unwrap());
        assert_close("mlipns/identical/nsim", 1.0,
            m.normalized_similarity(&cv("hello"), &cv("hello")).unwrap());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MRA
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_mra_integration() {
        let m = Mra::new();

        // identical names
        let s = m.normalized_similarity("catherine", "catherine").unwrap();
        assert_close("mra/identical/nsim", 1.0, s);

        // known similar: "catherine" vs "kathryn" — should match
        let s2 = m.similarity("catherine", "kathryn").unwrap();
        assert!(s2 > 0.0, "mra/catherine_kathryn: expected > 0, got {s2}");

        // empty vs empty
        let s3 = m.normalized_similarity("", "").unwrap();
        assert_close("mra/empty/nsim", 1.0, s3);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // StrCmp95
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_strcmp95_integration() {
        let m = StrCmp95::new();

        // empty vs empty → similarity = 1.0
        assert_close("strcmp95/empty/sim", 1.0, m.similarity("", "").unwrap());
        assert_close("strcmp95/empty/dist", 0.0, m.distance("", "").unwrap());
        assert_close("strcmp95/empty/nsim", 1.0, m.normalized_similarity("", "").unwrap());
        assert_close("strcmp95/empty/ndist", 0.0, m.normalized_distance("", "").unwrap());

        // identical
        assert_close("strcmp95/identical/sim", 1.0, m.similarity("hello", "hello").unwrap());

        // known pair "MARTHA" vs "MARHTA"
        let sim = m.similarity("MARTHA", "MARHTA").unwrap();
        assert_close("strcmp95/MARTHA_MARHTA/sim", 0.9611111111111111, sim);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Editex
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_editex_integration() {
        let m = Editex::new();

        assert_close("editex/identical/dist", 0.0, m.distance("hello", "hello") as f64);
        assert_close("editex/identical/nsim", 1.0, m.normalized_similarity("hello", "hello"));

        // empty vs empty
        assert_close("editex/empty/dist", 0.0, m.distance("", "") as f64);
        assert_close("editex/empty/nsim", 1.0, m.normalized_similarity("", ""));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // RlenCd (Run-Length NCD)
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_rle_ncd_integration() {
        let m = RlenCd::new();
        let cv = |s: &str| to_char_vec(s);

        // Empty vs empty -> similarity = 1.0
        let s = m.normalized_similarity(&cv(""), &cv("")).unwrap();
        assert_close("rle_ncd/empty/nsim", 1.0, s);

        // Non-empty pair produces valid similarity in [0, 1]
        let sim = m.similarity(&cv("hello"), &cv("hello")).unwrap();
        assert!(sim >= 0.0 && sim <= 1.0, "rle_ncd/hello_hello/sim expected in [0,1], got {sim}");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SqrtNcd
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_sqrt_ncd_integration() {
        let m = SqrtNcd::new();
        let cv = |s: &str| to_char_vec(s);

        // Empty vs empty -> 1.0
        assert_close("sqrt_ncd/empty/nsim", 1.0,
            m.normalized_similarity(&cv(""), &cv("")).unwrap());

        // Identical non-empty string produces valid score
        let sim = m.similarity(&cv("hello"), &cv("hello")).unwrap();
        assert!(!sim.is_nan(), "sqrt_ncd/hello_hello/sim should not be NaN, got {sim}");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ArithNcd
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_arith_ncd_integration() {
        let m = ArithNcd::new();
        let cv = |s: &str| to_char_vec(s);

        // Empty vs empty -> 1.0
        assert_close("arith_ncd/empty/nsim", 1.0,
            m.normalized_similarity(&cv(""), &cv("")).unwrap());

        // Non-empty string produces valid score
        let sim = m.similarity(&cv("hello"), &cv("hello")).unwrap();
        assert!(!sim.is_nan(), "arith_ncd/hello_hello/sim should not be NaN, got {sim}");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Bag
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_bag_integration() {
        let m = Bag::new();
        let cv = |s: &str| to_char_vec(s);

        assert_close("bag/identical/dist", 0.0,
            m.distance(&cv("hello"), &cv("hello")).unwrap());
        assert_close("bag/identical/nsim", 1.0,
            m.normalized_similarity(&cv("hello"), &cv("hello")).unwrap());

        // anagram should have dist=0
        assert_close("bag/anagram/dist", 0.0,
            m.distance(&cv("listen"), &cv("silent")).unwrap());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Tversky
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_tversky_integration() {
        let m = Tversky::new();
        let cv = |s: &str| to_char_vec(s);

        assert_close("tversky/identical/nsim", 1.0,
            m.normalized_similarity(&cv("hello"), &cv("hello")).unwrap());

        assert_close("tversky/disjoint/nsim", 0.0,
            m.normalized_similarity(&cv("abc"), &cv("xyz")).unwrap());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Matrix
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_matrix_integration() {
        let m = Matrix::<char>::new();
        let cv = |s: &str| to_char_vec(s);

        assert_close("matrix/identical/nsim", 1.0,
            m.normalized_similarity(&cv("abc"), &cv("abc")).unwrap());

        assert_close("matrix/empty/nsim", 1.0,
            m.normalized_similarity(&cv(""), &cv("")).unwrap());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Prefix / Postfix
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_prefix_postfix_integration() {
        let prefix  = Prefix::new();
        let postfix = Postfix::new();
        let cv = |s: &str| to_char_vec(s);

        assert_close("prefix/identical/nsim", 1.0,
            prefix.normalized_similarity(&cv("hello"), &cv("hello")).unwrap());
        assert_close("prefix/no_match/nsim", 0.0,
            prefix.normalized_similarity(&cv("abc"), &cv("xyz")).unwrap());

        assert_close("postfix/identical/nsim", 1.0,
            postfix.normalized_similarity(&cv("world"), &cv("world")).unwrap());
        assert_close("postfix/no_match/nsim", 0.0,
            postfix.normalized_similarity(&cv("abc"), &cv("xyz")).unwrap());

        // "rust" vs "trust" → common postfix "rust" = 4 chars, max = 5
        let s = postfix.normalized_similarity(&cv("rust"), &cv("trust")).unwrap();
        assert_close("postfix/rust_trust/nsim", 4.0 / 5.0, s);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Length
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_length_integration() {
        let m = Length::new();
        let cv = |s: &str| to_char_vec(s);

        assert_close("length/same_len/dist", 0.0,
            m.distance(&cv("abc"), &cv("xyz")).unwrap());
        assert_close("length/same_len/nsim", 1.0,
            m.normalized_similarity(&cv("abc"), &cv("xyz")).unwrap());

        assert_close("length/diff_len/dist", 3.0,
            m.distance(&cv("abc"), &cv("abcdef")).unwrap());
    }
}
