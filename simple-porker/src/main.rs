use rand::prelude::SliceRandom;
use std::fmt;
use std::fmt::Display;
use std::ops::Deref;
use std::ops::DerefMut;

mod io;

use crate::io::{DiscardAction, prompt_discard};

fn main() {
    let mut deck = Deck::new();
    let mut hands = Hands::new_from_deck(&mut deck);

    for _ in 0..2 {
        let action = prompt_discard(&hands);
        match action {
            DiscardAction::Stand => break,
            DiscardAction::Discard(v) => {
                for i in v {
                    hands.exchange(&mut deck, hands[i]);
                }
            }
        }
    }

    let rank = hands.rank();
    println!("{hands}");
    println!("{rank:?}");
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Suit {
    Clover,
    Diamond,
    Heart,
    Spade,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Card {
    suit: Suit,
    number: u8,
}

impl Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suit_emoji = match self.suit {
            Suit::Clover => "♣️",
            Suit::Diamond => "♦️",
            Suit::Heart => "❤️",
            Suit::Spade => "♠️",
        };

        let num_str: &str = match self.number {
            1 => "A",
            11 => "J",
            12 => "Q",
            13 => "K",
            _ => return write!(f, "{}{}", suit_emoji, self.number),
        };
        write!(f, "{}{}", suit_emoji, num_str)
    }
}

impl Card {
    /// const で実行され、範囲外はコンパイルエラーになる。
    pub const fn new(suit: Suit, number: u8) -> Self {
        if !(1 <= number && number <= 13) {
            panic!("card number must be 1..=13");
        }
        Self { number, suit }
    }
}

#[allow(dead_code)]
const fn card(suit: Suit, number: u8) -> Card {
    Card::new(suit, number)
}

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        use Suit::*;

        let mut cards: Vec<_> = (1..=13)
            .flat_map(|i| {
                [
                    Card::new(Clover, i),
                    Card::new(Diamond, i),
                    Card::new(Heart, i),
                    Card::new(Spade, i),
                ]
            })
            .collect();

        let mut rng = rand::rng();
        cards.shuffle(&mut rng);

        debug_assert!(cards.len() == 52);

        Deck { cards }
    }

    pub fn draw(&mut self) -> Card {
        self.cards.pop().unwrap()
    }
}

#[derive(Debug, PartialEq)]
pub enum Rank {
    HighCard(u8),
    OnePair,
    TwoPair,
    ThreeCard,
    Straight,
    Flush,
    FullHouse,
    FourCard,
    StraightFlush,
    RoyalStraightFlush,
}

impl Rank {
    fn evaluate(hands: &Hands) -> Rank {
        let stats = HandStats::from(hands);

        if stats.is_royal_straight_flush() {
            return Rank::RoyalStraightFlush;
        }
        if stats.is_straight_flush() {
            return Rank::StraightFlush;
        }
        if stats.is_four_card() {
            return Rank::FourCard;
        }
        if stats.is_full_house() {
            return Rank::FullHouse;
        }
        if stats.is_flush() {
            return Rank::Flush;
        }
        if stats.is_straight() {
            return Rank::Straight;
        }
        if stats.is_three_card() {
            return Rank::ThreeCard;
        }
        if stats.is_two_pair() {
            return Rank::TwoPair;
        }
        if stats.is_one_pair() {
            return Rank::OnePair;
        }

        Rank::HighCard(stats.highest)
    }
}

#[derive(Debug, Clone)]
struct HandStats {
    counts: [u8; 14],
    highest: u8,
    flush: bool,
    straight: bool,
    pairs: u8,
    triples: u8,
    quads: u8,
}

impl HandStats {
    fn from(hands: &Hands) -> Self {
        let mut counts: [u8; 14] = [0; 14]; // 0 は未使用、1..13 を利用
        let mut suit_counts: [u8; 4] = [0; 4];
        let mut numbers: [u8; 5] = [0; 5];

        for (idx, card) in hands.iter().enumerate() {
            counts[card.number as usize] += 1;
            suit_counts[card.suit as usize] += 1;
            numbers[idx] = card.number;
        }

        numbers.sort_unstable();
        let flush = suit_counts.iter().any(|&c| c == 5);
        let highest = *numbers.last().unwrap();

        let mut pairs = 0u8;
        let mut triples = 0u8;
        let mut quads = 0u8;
        for &c in counts.iter().skip(1) {
            match c {
                2 => pairs += 1,
                3 => triples += 1,
                4 => quads += 1,
                _ => {}
            }
        }

        HandStats {
            counts,
            highest,
            flush,
            straight: Self::calc_straight(numbers),
            pairs,
            triples,
            quads,
        }
    }

    fn is_one_pair(&self) -> bool {
        self.pairs == 1
    }

    fn is_two_pair(&self) -> bool {
        self.pairs == 2
    }

    fn is_three_card(&self) -> bool {
        self.triples == 1
    }

    fn is_four_card(&self) -> bool {
        self.quads == 1
    }

    fn is_full_house(&self) -> bool {
        self.triples == 1 && self.pairs == 1
    }

    fn is_flush(&self) -> bool {
        self.flush
    }

    fn is_straight(&self) -> bool {
        self.straight
    }

    fn is_straight_flush(&self) -> bool {
        self.is_flush() && self.is_straight()
    }

    fn is_royal_straight_flush(&self) -> bool {
        self.is_straight_flush()
            && self.counts[1] == 1
            && self.counts[10] == 1
            && self.counts[11] == 1
            && self.counts[12] == 1
            && self.counts[13] == 1
    }

    fn calc_straight(nums: [u8; 5]) -> bool {
        // 通常ストレート: 連続差がすべて 1
        let consecutive = nums.windows(2).all(|w| w[1] == w[0] + 1);
        if consecutive {
            return true;
        }

        // 例外パターン: ホイール A2345、ブロードウェイ TJQKA
        nums == [1, 2, 3, 4, 5] || nums == [1, 10, 11, 12, 13]
    }
}

#[derive(Debug)]
pub struct Hands([Card; 5]);

impl Deref for Hands {
    type Target = [Card; 5];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Hands {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Hands {
    fn new_from_deck(deck: &mut Deck) -> Self {
        Hands([
            deck.draw(),
            deck.draw(),
            deck.draw(),
            deck.draw(),
            deck.draw(),
        ])
    }

    fn exchange(&mut self, deck: &mut Deck, card: Card) {
        let i = self.iter().position(|&x| x == card).unwrap();

        self[i] = deck.draw();
    }

    fn rank(&self) -> Rank {
        Rank::evaluate(self)
    }

    /// 連続した5枚を生成する。10 を渡すとロイヤル (10,J,Q,K,A) になる。
    pub const fn straight(suit: Suit, start: u8) -> Self {
        const fn wrap(n: u8) -> u8 {
            ((n - 1) % 13) + 1
        }

        Hands([
            card(suit, wrap(start)),
            card(suit, wrap(start + 1)),
            card(suit, wrap(start + 2)),
            card(suit, wrap(start + 3)),
            card(suit, wrap(start + 4)),
        ])
    }

    /// A,2,3,4,5 のストレート（ホイール）。
    pub const fn wheel(suit: Suit) -> Self {
        Hands::straight(suit, 1)
    }

    /// 10,J,Q,K,A のロイヤルストレート。
    pub const fn royal(suit: Suit) -> Self {
        Hands::straight(suit, 10)
    }
}

impl Display for Hands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, card) in self.0.iter().enumerate() {
            write!(f, "{}. ", i + 1)?; // 1-indexed;
            card.fmt(f)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // テスト用の簡潔な手札リテラル。配列長が 5 であることは型で保証される。
    macro_rules! hand {
        ( $( $suit:ident $num:expr ),+ $(,)? ) => {
            Hands([ $( card(Suit::$suit, $num) ),+ ])
        };
    }

    #[test]
    fn straight() {
        let hands = hand![Heart 1, Spade 2, Clover 3, Diamond 4, Heart 5];
        assert_eq!(hands.rank(), Rank::Straight);
    }

    #[test]
    fn straight_flush() {
        let hands = Hands::straight(Suit::Spade, 5);
        assert_eq!(hands.rank(), Rank::StraightFlush);
    }

    #[test]
    fn royal_straight_flush() {
        let hands = Hands::royal(Suit::Diamond);
        assert_eq!(hands.rank(), Rank::RoyalStraightFlush);
    }

    #[test]
    fn four_card() {
        let hands = hand![Heart 9, Spade 9, Clover 9, Diamond 9, Heart 2];
        assert_eq!(hands.rank(), Rank::FourCard);
    }

    #[test]
    fn full_house() {
        let hands = hand![Heart 3, Spade 3, Clover 3, Diamond 8, Heart 8];
        assert_eq!(hands.rank(), Rank::FullHouse);
    }

    #[test]
    fn flush() {
        let hands = hand![Spade 2, Spade 6, Spade 9, Spade 11, Spade 13];
        assert_eq!(hands.rank(), Rank::Flush);
    }

    #[test]
    fn not_straight() {
        let hands = hand![Heart 1, Heart 2, Heart 3, Heart 4, Heart 6];
        let rank = hands.rank();

        assert_ne!(rank, Rank::Straight);
    }

    #[test]
    fn straght_with_upper_a() {
        let hands = Hands::royal(Suit::Heart);
        let rank = hands.rank();

        assert_eq!(rank, Rank::RoyalStraightFlush);
    }

    #[test]
    fn three_card() {
        let hands = hand![Heart 4, Spade 4, Diamond 4, Clover 7, Heart 9];
        assert_eq!(hands.rank(), Rank::ThreeCard);
    }

    #[test]
    fn two_pair() {
        let hands = hand![Heart 5, Spade 5, Diamond 12, Clover 12, Heart 3];
        assert_eq!(hands.rank(), Rank::TwoPair);
    }

    #[test]
    fn one_pair() {
        let hands = hand![Heart 5, Spade 5, Diamond 7, Clover 9, Heart 11];
        assert_eq!(hands.rank(), Rank::OnePair);
    }

    #[test]
    fn high_card() {
        let hands = hand![Heart 2, Spade 5, Diamond 7, Clover 9, Heart 12];
        assert_eq!(hands.rank(), Rank::HighCard(12));
    }
}
