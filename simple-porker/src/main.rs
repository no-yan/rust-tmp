use rand::prelude::SliceRandom;
use std::cmp::{max, min};
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
pub enum Card {
    Clover(u8),
    Diamond(u8),
    Heart(u8),
    Spade(u8),
}

impl Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (suit, number) = match self {
            Card::Clover(n) => ("♣️", n),
            Card::Diamond(n) => ("♦️", n),
            Card::Heart(n) => ("❤️", n),
            Card::Spade(n) => ("♠️", n),
        };

        let num_str: &str = match number {
            1 => "A",
            11 => "J",
            12 => "Q",
            13 => "K",
            _ => &number.to_string(),
        };
        write!(f, "{}{}", suit, num_str)
    }
}

impl Card {
    fn number(&self) -> u8 {
        let n = match self {
            Card::Clover(n) => n,
            Card::Diamond(n) => n,
            Card::Heart(n) => n,
            Card::Spade(n) => n,
        };
        *n
    }
}

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        use Card::*;

        let mut cards: Vec<_> = (1..=13)
            .flat_map(|i| [Clover(i), Diamond(i), Heart(i), Spade(i)])
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
        if Self::is_loyal_straght_flush(hands) {
            return Rank::RoyalStraightFlush;
        }
        if Self::is_straight_flush(hands) {
            return Rank::StraightFlush;
        }
        if Self::is_four_card(hands) {
            return Rank::FourCard;
        }
        if Self::is_full_house(hands) {
            return Rank::FullHouse;
        }
        if Self::is_flush(hands) {
            return Rank::Flush;
        }
        if Self::is_straight(hands) {
            return Rank::Straight;
        }
        if Self::is_three_card(hands) {
            return Rank::ThreeCard;
        }
        if Self::is_two_pair(hands) {
            return Rank::TwoPair;
        }
        if Self::is_one_pair(hands) {
            return Rank::OnePair;
        }

        let highest = hands.iter().map(|card| card.number()).max().unwrap_or(0);
        Rank::HighCard(highest)
    }

    fn is_one_pair(hands: &Hands) -> bool {
        let mut seen: [bool; 14] = Default::default();
        let mut max_pair: i8 = -1;
        for card in hands.0 {
            let number: usize = card.number().into();
            if seen[number] {
                max_pair = max(max_pair, number as i8);
            }
            seen[number] = true;
        }

        max_pair >= 0
    }

    fn is_two_pair(hands: &Hands) -> bool {
        unimplemented!();
    }
    fn is_three_card(hands: &Hands) -> bool {
        unimplemented!();
    }
    fn is_straight(hands: &Hands) -> bool {
        // ストレートは連続した数値からなる5枚のカードから作られる
        // これは、以下の条件と同値である:
        // 1. 手札の5枚の最大値と最小値の差が4
        // 2. 手札に同じ値が存在しない
        //
        // エッジケースとして、Aは1,2,3,4,5と10,11,12,13,1の二種類のストーレートに含まれる。
        // 10, 11, 12, 13, 1は 最大値(13)と最小値(1)の差が1にならないため、個別に判定する
        let mut seen: [bool; 14] = Default::default();
        let (mut mn, mut mx) = (usize::MAX, 0);

        for card in hands.0 {
            let number: usize = card.number().into();
            if seen[number] {
                return false;
            }
            seen[number] = true;
            mn = min(mn, number);
            mx = max(mx, number);
        }

        mx - mn == 4
        // エッジケース: 10, 11,12,13, 1
       ||  (seen[1] && seen[10] && seen[11] && seen[12] && seen[13])
    }

    fn is_flush(hands: &Hands) -> bool {
        unimplemented!();
    }
    fn is_full_house(hands: &Hands) -> bool {
        unimplemented!();
    }
    fn is_four_card(hands: &Hands) -> bool {
        unimplemented!();
    }
    fn is_straight_flush(hands: &Hands) -> bool {
        unimplemented!();
    }
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
        let mut hands = Hands([Card::Clover(1); 5]);
        for i in 0..5 {
            hands[i] = deck.draw();
        }

        hands
    }

    fn exchange(&mut self, deck: &mut Deck, card: Card) {
        let i = self.iter().position(|&x| x == card).unwrap();

        self[i] = deck.draw();
    }

    fn rank(&self) -> Rank {
        Rank::evaluate(self)
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
    use super::Card::*;
    use super::*;

    #[test]
    fn straight() {
        let hands: Hands = Hands([Heart(1), Heart(2), Heart(3), Heart(4), Heart(5)]);
        let rank = hands.rank();

        assert_eq!(rank, Rank::Straight);
    }

    #[test]
    fn not_straight() {
        let hands: Hands = Hands([Heart(1), Heart(2), Heart(3), Heart(4), Heart(6)]);
        let rank = hands.rank();

        assert_ne!(rank, Rank::Straight);
    }

    #[test]
    fn straght_with_upper_a() {
        let hands: Hands = Hands([Heart(10), Heart(11), Heart(12), Heart(13), Heart(1)]);
        let rank = hands.rank();

        assert_eq!(rank, Rank::Straight);
    }
}
