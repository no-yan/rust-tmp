use rand::prelude::SliceRandom;
use std::cmp::{max, min};
use std::ops::Deref;
use std::ops::DerefMut;

// TODO:
// 1. 手役を実装する
//  - OnePair, TwoPair, ThreeCard, Straight, Flash,
//  - FullHouse, FourCard, StraightFlash, LoyalStraghtFlash,
// 2. 対話的に引き直しを実装する
fn main() {
    let mut deck = Deck::new();
    let mut hands = Hands::new_from_deck(&mut deck);

    println!("{hands:?}");
    // TODO: 2回交換できる
    hands.exchange(&mut deck, hands[0]);

    let rank = hands.rank();
    println!("{hands:?}");
    println!("{rank:?}");
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Card {
    Clover(u8),
    Diamond(u8),
    Heart(u8),
    Spade(u8),
}

impl Card {
    fn number(self) -> usize {
        match self {
            Card::Clover(n) => n,
            Card::Diamond(n) => n,
            Card::Heart(n) => n,
            Card::Spade(n) => n,
        }
        .into()
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
    Straight,
    NoRank,
}

impl Rank {
    fn evaluate(hands: &Hands) -> Rank {
        if Self::is_straight(hands) {
            return Rank::Straight;
        }

        Rank::NoRank
    }

    fn is_straight(hands: &Hands) -> bool {
        // ストレートは連続した数値からなる5枚のカードから作られる
        // これは、以下の条件と同値である:
        // 1. 手札の5枚の最大値と最小値の差が4
        // 2. 手札に同じ値が存在しない
        //
        // エッジケースとして、Aは1,2,3,4,5と10,11,12,13,1の二種類のストーレートに含まれるため、最大値のAは手計算
        let mut seen: [bool; 14] = Default::default();
        let (mut mn, mut mx) = (255, 0);

        for card in hands.0 {
            let number = card.number();
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
