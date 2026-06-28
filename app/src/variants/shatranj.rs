//! Shatranj variant implementation.
//!
//! Implements the `Game` trait for shatranj using the meowshatranj library.

use crate::game::{GameOverReason, GameStatus, Side};
use crate::types::VariantType;
use crate::variants::{Game, GameMove};
use meowshatranj::core::Color;
use meowshatranj::position::{DecisiveType, DrawType, GameOutcome, Position};
use meowshatranj::shatranjmove::Move;
use std::any::Any;

fn to_san(_pos: &Position, mv: Move) -> String {
    //TODO
    format!("{}", mv)
}

fn to_lan(_pos: &Position, mv: Move) -> String {
    //TODO
    format!("{}", mv)
}

impl GameMove for Move {
    fn to_uci(&self) -> String {
        format!("{}", self)
    }

    fn to_san(&self, game: &dyn Game) -> Option<String> {
        let game = game.as_shatranj().unwrap();
        Some(to_san(game.inner(), *self))
    }

    fn to_lan(&self, game: &dyn Game) -> Option<String> {
        let game = game.as_shatranj().unwrap();
        Some(to_lan(game.inner(), *self))
    }

    fn clone_box(&self) -> Box<dyn GameMove> {
        Box::new(*self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A shatranj game with position tracking and automatic threefold repetition detection.
#[derive(Clone)]
pub struct ShatranjGame {
    pos: Position,
    hash_history: Vec<u64>,
    ply_count: u32,
}

impl Default for ShatranjGame {
    fn default() -> Self {
        Self::new()
    }
}

impl ShatranjGame {
    /// Creates a new game from the standard starting position.
    pub fn new() -> Self {
        let pos = Position::startpos();

        Self {
            pos,
            hash_history: Vec::new(),
            ply_count: 0,
        }
    }

    /// Creates a new game from a FEN string.
    pub fn from_fen(fen: &str) -> Option<Self> {
        let pos = match fen.parse::<Position>() {
            Ok(pos) => pos,
            Err(_) => return None,
        };

        Some(Self {
            pos,
            hash_history: Vec::new(),
            ply_count: 0,
        })
    }

    /// Returns a reference to the underlying meowshatranj position.
    pub fn inner(&self) -> &Position {
        &self.pos
    }

    /// Checks if the position is in check.
    pub fn is_check(&self) -> bool {
        self.pos.in_check()
    }

    /// Checks for threefold repetition.
    pub fn is_threefold_repetition(&self) -> bool {
        let curr_hash = self.pos.key();
        self.hash_history
            .iter()
            .rev()
            .skip(3)
            .step_by(2)
            .filter(|&&hash| hash == curr_hash)
            .count()
            >= 2
    }

    /// Parses a UCI move string.
    pub fn parse_uci_move(&self, uci: &str) -> Option<Move> {
        let mv = match uci.parse::<Move>() {
            Ok(mv) => mv,
            Err(_) => return None,
        };

        if !self.pos.is_legal(mv) {
            return None;
        }

        Some(mv)
    }

    /// Makes a move on the board.
    pub fn make_shatranj_move(&mut self, mv: Move) -> bool {
        // Verify the move is legal
        if !self.pos.is_legal(mv) {
            return false;
        }

        let hash = self.pos.key();
        self.hash_history.push(hash);

        self.pos = self.pos.apply_move(mv);
        self.ply_count += 1;

        true
    }

    /// Makes a move from a UCI string.
    pub fn make_uci_move(&mut self, uci: &str) -> bool {
        if let Ok(mv) = uci.parse() {
            self.make_shatranj_move(mv)
        } else {
            false
        }
    }

    /// Converts UCI to SAN.
    pub fn uci_to_san(&self, uci: &str) -> Option<String> {
        uci.parse().ok().map(|mv| to_san(&self.pos, mv))
    }

    /// Converts UCI to LAN.
    pub fn uci_to_lan(&self, uci: &str) -> Option<String> {
        uci.parse().ok().map(|mv| to_lan(&self.pos, mv))
    }
}

impl Game for ShatranjGame {
    fn clone_box(&self) -> Box<dyn Game> {
        Box::new(self.clone())
    }

    fn variant(&self) -> VariantType {
        VariantType::Shatranj
    }

    fn side_to_move(&self) -> Side {
        match self.pos.stm() {
            Color::White => Side::White,
            Color::Black => Side::Black,
        }
    }

    fn halfmove_clock(&self) -> u32 {
        self.pos.halfmove() as u32
    }

    fn ply_count(&self) -> u32 {
        self.ply_count
    }

    fn status(&self) -> GameStatus {
        if let Some(outcome) = self.pos.outcome() {
            return match outcome {
                GameOutcome::Win(win_type) => match win_type {
                    DecisiveType::Mate => unreachable!(),
                    DecisiveType::BareKing => GameStatus::new(GameOverReason::BareKingWin),
                },
                GameOutcome::Loss(loss_type) => match loss_type {
                    DecisiveType::Mate => GameStatus::new(GameOverReason::Checkmate),
                    DecisiveType::BareKing => GameStatus::new(GameOverReason::BareKingLoss),
                },
                GameOutcome::Draw(draw_type) => match draw_type {
                    DrawType::SeventyMoveRule => GameStatus::new(GameOverReason::SeventyMoveRule),
                    DrawType::InsufficientMaterial => {
                        GameStatus::new(GameOverReason::InsufficientMaterial)
                    }
                },
            };
        };

        if self.is_threefold_repetition() {
            return GameStatus::new(GameOverReason::Repetition);
        }

        GameStatus::ONGOING
    }

    fn parse_move(&self, notation: &str) -> Option<Box<dyn GameMove>> {
        self.parse_uci_move(notation)
            .map(|m| Box::new(m) as Box<dyn GameMove>)
    }

    fn make_move(&mut self, mv: &dyn GameMove) -> bool {
        if let Some(&shatranj_move) = mv.as_any().downcast_ref::<Move>() {
            self.make_shatranj_move(shatranj_move)
        } else {
            false
        }
    }

    fn make_move_notation(&mut self, notation: &str) -> bool {
        if let Some(mv) = self.parse_uci_move(notation) {
            self.make_shatranj_move(mv)
        } else {
            false
        }
    }

    fn fen(&self) -> String {
        self.pos.fen()
    }

    fn move_to_san(&self, notation: &str) -> Option<String> {
        self.uci_to_san(notation)
    }

    fn move_to_lan(&self, notation: &str) -> Option<String> {
        self.uci_to_lan(notation)
    }

    fn convert_move_to_san(&self, mv: &dyn GameMove) -> Option<String> {
        mv.as_any()
            .downcast_ref::<Move>()
            .map(|&mv| to_san(&self.pos, mv))
    }

    fn convert_move_to_lan(&self, mv: &dyn GameMove) -> Option<String> {
        mv.as_any()
            .downcast_ref::<Move>()
            .map(|&mv| to_lan(&self.pos, mv))
    }

    fn supports_syzygy(&self) -> bool {
        //TODO?
        false
    }

    fn as_chess(&self) -> Option<&crate::variants::chess::ChessGame> {
        None
    }

    fn as_shogi(&self) -> Option<&crate::variants::chess::ShogiGame> {
        None
    }

    fn as_shatranj(&self) -> Option<&ShatranjGame> {
        Some(self)
    }

    fn is_threefold_repetition(&self) -> bool {
        self.is_threefold_repetition()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let game = ShatranjGame::new();
        assert_eq!(game.ply_count(), 0);
        assert_eq!(game.halfmove_clock(), 0);
        assert!(!game.is_check());
        assert!(game.status().is_ongoing())
    }

    #[test]
    fn test_from_fen() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b - - 0 1";
        let game = ShatranjGame::from_fen(fen).unwrap();
        assert_eq!(game.side_to_move(), Side::Black);
    }

    #[test]
    fn test_make_move() {
        let mut game = ShatranjGame::new();
        assert!(game.make_move_notation("e2e3"));
        assert_eq!(game.side_to_move(), Side::Black);
        assert_eq!(game.ply_count(), 1);
    }

    #[test]
    fn test_game_trait() {
        let mut game: Box<dyn Game> = Box::new(ShatranjGame::new());
        assert_eq!(game.variant(), VariantType::Shatranj);
        assert!(game.make_move_notation("e2e3"));
        assert!(game.status().is_ongoing());
    }
}
