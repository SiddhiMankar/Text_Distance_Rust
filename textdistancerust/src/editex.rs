use std::cmp::max;
use std::collections::HashSet;

#[derive(Clone)]
pub struct Editex {
    pub local: bool,
    pub match_cost: usize,
    pub group_cost: usize,
    pub mismatch_cost: usize,
    pub groups: Vec<HashSet<char>>,
    pub ungrouped: HashSet<char>,
    pub grouped: HashSet<char>,
}

impl Default for Editex {
    fn default() -> Self {
        let groups_strs = vec![
            "AEIOUY", "BP", "CKQ", "DT", "LR", "MN", "GJ", "FPV", "SXZ", "CSZ",
        ];
        let mut groups = Vec::new();
        let mut grouped = HashSet::new();
        for s in groups_strs {
            let mut set = HashSet::new();
            for c in s.chars() {
                set.insert(c);
                grouped.insert(c);
            }
            groups.push(set);
        }
        let mut ungrouped = HashSet::new();
        ungrouped.insert('H');
        ungrouped.insert('W');

        Editex {
            local: false,
            match_cost: 0,
            group_cost: 1,
            mismatch_cost: 2,
            groups,
            ungrouped,
            grouped,
        }
    }
}

impl Editex {
    pub fn new() -> Self {
        Editex::default()
    }

    pub fn with_costs(
        match_cost: usize,
        group_cost: usize,
        mismatch_cost: usize,
    ) -> Self {
        let mut e = Editex::default();
        e.match_cost = match_cost;
        e.group_cost = max(group_cost, match_cost);
        e.mismatch_cost = max(mismatch_cost, e.group_cost);
        e
    }

    fn maximum(&self, s1: &str, s2: &str) -> usize {
        max(s1.chars().count(), s2.chars().count()) * self.mismatch_cost
    }

    fn r_cost(&self, c1: char, c2: char) -> usize {
        if c1 == c2 {
            return self.match_cost;
        }
        if !self.grouped.contains(&c1) || !self.grouped.contains(&c2) {
            return self.mismatch_cost;
        }
        for group in &self.groups {
            if group.contains(&c1) && group.contains(&c2) {
                return self.group_cost;
            }
        }
        self.mismatch_cost
    }

    fn d_cost(&self, c1: char, c2: char) -> usize {
        if c1 != c2 && self.ungrouped.contains(&c1) {
            return self.group_cost;
        }
        self.r_cost(c1, c2)
    }

    pub fn distance(&self, s1: &str, s2: &str) -> usize {
        if s1 == s2 {
            return 0;
        }
        if s1.is_empty() || s2.is_empty() {
            return self.maximum(s1, s2);
        }

        let max_length = self.maximum(s1, s2);

        // s1 = ' ' + s1.upper()
        // s2 = ' ' + s2.upper()
        let mut s1_chars: Vec<char> = vec![' '];
        for c in s1.chars() {
            for uc in c.to_uppercase() {
                s1_chars.push(uc);
            }
        }
        let mut s2_chars: Vec<char> = vec![' '];
        for c in s2.chars() {
            for uc in c.to_uppercase() {
                s2_chars.push(uc);
            }
        }

        let len_s1 = s1_chars.len() - 1;
        let len_s2 = s2_chars.len() - 1;

        let mut d_mat = vec![vec![0; len_s2 + 1]; len_s1 + 1];

        if !self.local {
            for i in 1..=len_s1 {
                d_mat[i][0] = d_mat[i - 1][0] + self.d_cost(s1_chars[i - 1], s1_chars[i]);
            }
        }
        for j in 1..=len_s2 {
            d_mat[0][j] = d_mat[0][j - 1] + self.d_cost(s2_chars[j - 1], s2_chars[j]);
        }

        for i in 1..=len_s1 {
            let cs1_prev = s1_chars[i - 1];
            let cs1_curr = s1_chars[i];
            for j in 1..=len_s2 {
                let cs2_prev = s2_chars[j - 1];
                let cs2_curr = s2_chars[j];

                let cost1 = d_mat[i - 1][j] + self.d_cost(cs1_prev, cs1_curr);
                let cost2 = d_mat[i][j - 1] + self.d_cost(cs2_prev, cs2_curr);
                let cost3 = d_mat[i - 1][j - 1] + self.r_cost(cs1_curr, cs2_curr);

                d_mat[i][j] = cost1.min(cost2).min(cost3);
            }
        }

        let distance = d_mat[len_s1][len_s2];
        distance.min(max_length)
    }

    pub fn similarity(&self, s1: &str, s2: &str) -> usize {
        self.maximum(s1, s2) - self.distance(s1, s2)
    }

    pub fn normalized_distance(&self, s1: &str, s2: &str) -> f64 {
        let max_len = self.maximum(s1, s2) as f64;
        if max_len == 0.0 {
            return 0.0;
        }
        self.distance(s1, s2) as f64 / max_len
    }

    pub fn normalized_similarity(&self, s1: &str, s2: &str) -> f64 {
        1.0 - self.normalized_distance(s1, s2)
    }
}
