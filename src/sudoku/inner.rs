#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

#[derive(Debug, Default)]
pub struct Scorer {
    seen: u64,
    score: u8,
}

impl Scorer {
    #[inline]
    pub fn check(&mut self, digit: u8) {
        let bit = 1 << digit;

        if self.seen & bit != 0 {
            self.score += 1;
        } else {
            self.seen |= bit;
        }
    }

    #[inline]
    #[must_use]
    pub const fn score(self) -> u8 {
        self.score
    }
}

#[cfg(test)]
mod tests {
    use super::Scorer;

    #[test]
    fn test_scorer_no_duplicates() {
        let mut scorer = Scorer::default();

        // Since Scorer.seen is u64, it supports up to 49 before overflowing.
        for i in 1..=49 {
            scorer.check(i);
        }

        assert_eq!(0, scorer.score());
    }

    #[test]
    fn test_scorer_with_duplicates() {
        let mut scorer = Scorer::default();

        scorer.check(1);
        scorer.check(1); // One duplicate
        scorer.check(2);
        scorer.check(2); // Two duplicates
        scorer.check(2); // Three duplicates

        assert_eq!(3, scorer.score());
    }
}
