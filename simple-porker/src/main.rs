use rand::prelude::SliceRandom;
use std::cmp::max;
use std::fmt;
use std::fmt::Display;
use std::ops::Deref;
use std::ops::DerefMut;

mod io;

use crate::io::{DiscardAction, prompt_discard};

// TODO:
// - [ ] 未実装の手役を実装する
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
        // if Self::is_loyal_straght_flush(hands) {
        //     return Rank::RoyalStraightFlush;
        // }
        // if Self::is_straight_flush(hands) {
        //     return Rank::StraightFlush;
        // }
        // if Self::is_four_card(hands) {
        //     return Rank::FourCard;
        // }
        // if Self::is_full_house(hands) {
        //     return Rank::FullHouse;
        // }
        // if Self::is_flush(hands) {
        //     return Rank::Flush;
        // }
        if Self::is_straight(hands) {
            return Rank::Straight;
        }
        // if Self::is_three_card(hands) {
        //     return Rank::ThreeCard;
        // }
        // if Self::is_two_pair(hands) {
        //     return Rank::TwoPair;
        // }
        // if Self::is_one_pair(hands) {
        //     return Rank::OnePair;
        // }

        let highest = hands.iter().map(|card| card.number).max().unwrap_or(0);
        Rank::HighCard(highest)
    }

    #[allow(dead_code, unused_variables)]
    fn is_one_pair(hands: &Hands) -> bool {
        let mut seen: [bool; 14] = Default::default();
        let mut max_pair: i8 = -1;
        for card in hands.0 {
            let number: usize = card.number.into();
            if seen[number] {
                max_pair = max(max_pair, number as i8);
            }
            seen[number] = true;
        }

        max_pair >= 0
    }

    #[allow(dead_code, unused_variables)]
    fn is_two_pair(hands: &Hands) -> bool {
        unimplemented!();
    }
    #[allow(dead_code, unused_variables)]
    fn is_three_card(hands: &Hands) -> bool {
        unimplemented!();
    }
    #[allow(dead_code, unused_variables)]
    fn is_straight(hands: &Hands) -> bool {
        // 手札が連続した5つの数値であるか確認する
        let mut seen: [bool; 15] = Default::default();

        for card in hands.0 {
            let number: usize = card.number.into();
            if seen[number] {
                return false;
            }
            seen[number] = true;
        }

        // Aで終わるストレートを考慮するため、10..14をチェックする
        seen[14] = seen[1];

        let mut sum: usize = seen[0..5].iter().filter(|&&b| b).count();
        for i in 0..10 {
            sum -= seen[i] as usize;
            sum += seen[i + 5] as usize;

            if sum == 5 {
                return true;
            }
        }

        false
    }

    #[allow(dead_code, unused_variables)]
    fn is_flush(hands: &Hands) -> bool {
        unimplemented!();
    }
    #[allow(dead_code, unused_variables)]
    fn is_full_house(hands: &Hands) -> bool {
        unimplemented!();
    }
    #[allow(dead_code, unused_variables)]
    fn is_four_card(hands: &Hands) -> bool {
        unimplemented!();
    }
    #[allow(dead_code, unused_variables)]
    fn is_straight_flush(hands: &Hands) -> bool {
        unimplemented!();
    }
    #[allow(dead_code, unused_variables)]
    fn is_loyal_straght_flush(hands: &Hands) -> bool {
        unimplemented!();
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
        Hands(
            [
                deck.draw(),
                deck.draw(),
                deck.draw(),
                deck.draw(),
                deck.draw(),

            ]
        )
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
        let hands = Hands::straight(Suit::Heart, 1);
        let rank = hands.rank();

        assert_eq!(rank, Rank::Straight);
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

        assert_eq!(rank, Rank::Straight);
    }
}
