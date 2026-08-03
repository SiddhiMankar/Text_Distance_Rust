pub fn to_char_vec(s: &str) -> Vec<char> {
    s.chars().collect()
}

pub fn to_word_vec(s: &str) -> Vec<&str> {
    s.split_whitespace().collect()
}

pub fn find_ngrams<T: Clone>(input: &[T], n: usize) -> Vec<Vec<T>> {
    if n == 0 || input.len() < n {
        return Vec::new();
    }
    input.windows(n).map(|w| w.to_vec()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_char_vec() {
        assert_eq!(to_char_vec("hello"), vec!['h', 'e', 'l', 'l', 'o']);
        assert_eq!(to_char_vec("🔥🔑"), vec!['🔥', '🔑']);
        assert_eq!(to_char_vec(""), Vec::<char>::new());
    }

    #[test]
    fn test_to_word_vec() {
        assert_eq!(
            to_word_vec("hello world text"),
            vec!["hello", "world", "text"]
        );
        assert_eq!(to_word_vec("   spaced   out  "), vec!["spaced", "out"]);
        assert_eq!(to_word_vec(""), Vec::<&str>::new());
    }

    #[test]
    fn test_find_ngrams() {
        let chars = vec!['a', 'b', 'c', 'd'];
        assert_eq!(
            find_ngrams(&chars, 2),
            vec![vec!['a', 'b'], vec!['b', 'c'], vec!['c', 'd']]
        );
        assert_eq!(find_ngrams(&chars, 5), Vec::<Vec<char>>::new());
    }
}
