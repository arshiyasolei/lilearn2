use serde::{Deserialize, Serialize};

pub const PAWN_WHITE: i8 = 2;
pub const PAWN_BLACK: i8 = 1;
pub const ROOK_WHITE: i8 = 10;
pub const ROOK_BLACK: i8 = 5;
pub const KNIGHT_WHITE: i8 = 11;
pub const KNIGHT_BLACK: i8 = 4;
pub const QUEEN_WHITE: i8 = 13;
pub const QUEEN_BLACK: i8 = 6;
pub const BISHOP_WHITE: i8 = 12;
pub const BISHOP_BLACK: i8 = 3;
pub const KING_BLACK: i8 = 7;
pub const KING_WHITE: i8 = 14;
pub const STAR_VALUE: i8 = 99;
#[derive(Debug, Clone)]
pub struct MovePiece {
    pub i: usize,
    pub j: usize,
    pub goal_i: usize,
    pub goal_j: usize,
}

pub enum MoveStatus {
    Valid,
    Invalid,
}

impl MoveStatus {
    #[inline]
    pub const fn is_valid(&self) -> bool {
        match self {
            Self::Valid => true,
            Self::Invalid => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LiBoard {
    // 8x8 board
    pub board: [[i8; 8]; 8],
    // Position of the player moving piece
    pub main_piece: (i8, i8),
    // How many stars on the board at the start
    pub num_star_cnt: i8,
}

impl LiBoard {
    // set up board randomly with n stars and choice piece
    pub fn new(star_cnt: i8, choice_piece: i8) -> LiBoard {
        let mut b = [[0; 8]; 8];
        let mut star_pairs = Vec::new();
        let mut already_added_stars = HashMap::new();

        let main_piece_i = rand::random::<u8>() % 8;
        let main_piece_j = rand::random::<u8>() % 8;
        already_added_stars.insert((main_piece_i, main_piece_j), 0);
        for _v in 0..star_cnt {
            let mut sample = (rand::random::<u8>() % 8, rand::random::<u8>() % 8);
            while already_added_stars.contains_key(&sample) {
                sample = (rand::random::<u8>() % 8, rand::random::<u8>() % 8);
            }
            already_added_stars.insert(sample, 0);
            star_pairs.push(sample);
        }

        for i in 0..8 {
            for j in 0..8 {
                if i == main_piece_i && j == main_piece_j {
                    b[i as usize][j as usize] = choice_piece;
                } else {
                    b[i as usize][j as usize] = 0;
                }
            }
        }

        for i in 0..star_cnt {
            b[star_pairs[i as usize].0 as usize][star_pairs[i as usize].1 as usize] = STAR_VALUE;
        }

        LiBoard {
            board: b,
            main_piece: (main_piece_i as i8, main_piece_j as i8),
            num_star_cnt: star_cnt as i8,
        }
    }

    /// Checks to see if we are jumping over a star
    pub fn is_jumping_over_piece(&self, m_piece: &MovePiece) -> bool {
        // get start piece pos
        let i = m_piece.i as i8;
        let j = m_piece.j as i8;

        // determine movement type
        // vertical
        if (m_piece.goal_i as i8 - i).abs() == 0 {
            if j < m_piece.goal_j as i8 {
                let mut start = j + 1;
                while start <= m_piece.goal_j as i8 {
                    if self.board[i as usize][start as usize] != 0 {
                        if start as usize == m_piece.goal_j && self.board[m_piece.goal_i][m_piece.goal_j] == STAR_VALUE {
                            return false;
                        }
                        return true;
                    }
                    start += 1;
                }
                false
            } else {
                let mut start = j - 1;
                while start >= m_piece.goal_j as i8 {
                    if self.board[i as usize][start as usize] != 0 {
                        if start == m_piece.goal_j as i8 && self.board[m_piece.goal_i][m_piece.goal_j] == STAR_VALUE {
                            return false;
                        }
                        return true;
                    }
                    start -= 1;
                }
                false
            }
        }
        // horizontal
        else if (m_piece.goal_j as i8 - j).abs() == 0 {
            if i < m_piece.goal_i as i8 {
                let mut start = i + 1;
                while start <= m_piece.goal_i as i8 {
                    if self.board[start as usize][j as usize] != 0 {
                        if start == m_piece.goal_i as i8 && self.board[m_piece.goal_i][m_piece.goal_j] == STAR_VALUE {
                            return false;
                        }
                        return true;
                    }
                    start += 1;
                }
                false
            } else {
                let mut start = i - 1;
                while start >= m_piece.goal_i as i8 {
                    if self.board[start as usize][j as usize] != 0 {
                        if start == m_piece.goal_i as i8 && self.board[m_piece.goal_i][m_piece.goal_j] == STAR_VALUE {
                            return false;
                        }
                        return true;
                    }
                    start -= 1;
                }
                false
            }
        }
        // everything else (diagonals)
        else if (j < m_piece.goal_j as i8) && (i < m_piece.goal_i as i8) {
            let mut temp = i + 1;
            let mut start = j + 1;
            while start <= m_piece.goal_j as i8 {
                if self.board[temp as usize][start as usize] != 0 {
                    if start == m_piece.goal_j as i8 && self.board[m_piece.goal_i][m_piece.goal_j] == STAR_VALUE {
                        return false;
                    }
                    return true;
                }
                temp += 1;
                start += 1;
            }
            false
        } else if (j < m_piece.goal_j as i8) && (i > m_piece.goal_i as i8) {
            let mut temp = i - 1;
            let mut start = j + 1;
            while start <= m_piece.goal_j as i8 {
                if self.board[temp as usize][start as usize] != 0 {
                    if start == m_piece.goal_j as i8 && self.board[m_piece.goal_i][m_piece.goal_j] == STAR_VALUE {
                        return false;
                    }
                    return true;
                }
                temp -= 1;
                start += 1;
            }
            false
        } else if (j > m_piece.goal_j as i8) && (i < m_piece.goal_i as i8) {
            let mut temp = i + 1;
            let mut start = j - 1;
            while start >= m_piece.goal_j as i8 {
                if self.board[temp as usize][start as usize] != 0 {
                    if start == m_piece.goal_j as i8 && self.board[m_piece.goal_i][m_piece.goal_j] == STAR_VALUE {
                        return false;
                    }
                    return true;
                }
                temp += 1;
                start -= 1;
            }
            false
        } else {
            let mut temp = i - 1;
            let mut start = j - 1;
            while start >= m_piece.goal_j as i8 {
                if self.board[temp as usize][start as usize] != 0 {
                    if start == m_piece.goal_j as i8 && self.board[m_piece.goal_i][m_piece.goal_j] == STAR_VALUE {
                        return false;
                    }
                    return true;
                }
                temp -= 1;
                start -= 1;
            }
            false
        }
    }
    pub fn validate_move_rook(&self, m_piece: &MovePiece) -> MoveStatus {
        // make sure goal_i , goal_j reaches the first blocking piece
        let i = m_piece.i as i8;
        let j = m_piece.j as i8;

        if (((m_piece.goal_i as i8 - i).abs() == 0) || ((m_piece.goal_j as i8 - j).abs() == 0)) && !self.is_jumping_over_piece(m_piece) {
            return MoveStatus::Valid;
        }
        MoveStatus::Invalid
    }
    pub fn validate_move_bishop(&self, m_piece: &MovePiece) -> MoveStatus {
        let i = m_piece.i as i8;
        let j = m_piece.j as i8;
        // part one of validity
        if (m_piece.goal_i as i8 - i).abs() == (m_piece.goal_j as i8 - j).abs() && !self.is_jumping_over_piece(m_piece) {
            return MoveStatus::Valid;
        }
        MoveStatus::Invalid
    }
    pub fn validate_move_queen(&self, m_piece: &MovePiece) -> MoveStatus {
        let i = m_piece.i as i8;
        let j = m_piece.j as i8;
        // part one of validity
        if (((m_piece.goal_i as i8 - i).abs() == 0) || ((m_piece.goal_j as i8 - j).abs() == 0) || ((m_piece.goal_i as i8 - i).abs() == (m_piece.goal_j as i8 - j).abs()))
            && !self.is_jumping_over_piece(m_piece)
        {
            return MoveStatus::Valid;
        }
        MoveStatus::Invalid
    }

    pub fn validate_move_knight(&self, m_piece: &MovePiece) -> MoveStatus {
        let i = m_piece.i as i8;
        let j = m_piece.j as i8;
        if ((m_piece.goal_i as i8 - i).abs() == 2 && (m_piece.goal_j as i8 - j).abs() == 1) || ((m_piece.goal_i as i8 - i).abs() == 1 && (m_piece.goal_j as i8 - j).abs() == 2) {
            return MoveStatus::Valid;
        }
        MoveStatus::Invalid
    }

    pub fn validate_move(&self, m_piece: &MovePiece) -> MoveStatus {
        // leap of faith
        // if the piece that we are trying to move exists
        if m_piece.i == m_piece.goal_i && m_piece.j == m_piece.goal_j {
            return MoveStatus::Invalid;
        }
        // check for out of bounds
        if m_piece.goal_i >= 8 || m_piece.goal_j >= 8 {
            return MoveStatus::Invalid;
        }
        if self.board[m_piece.i][m_piece.j] > 0 {
            return match self.board[m_piece.i][m_piece.j] {
                PAWN_BLACK | PAWN_WHITE => panic!("Pawns not supported!"),

                BISHOP_BLACK | BISHOP_WHITE => self.validate_move_bishop(m_piece),

                KNIGHT_BLACK | KNIGHT_WHITE => self.validate_move_knight(m_piece),

                ROOK_BLACK | ROOK_WHITE => self.validate_move_rook(m_piece),

                KING_BLACK | KING_WHITE => panic!("Kings not supported!"),

                QUEEN_BLACK | QUEEN_WHITE => self.validate_move_queen(m_piece),

                _ => panic!("Unexpected case!"),
            };
        }
        MoveStatus::Invalid
    }
    pub fn update_board(&mut self, m_piece: &MovePiece) {
        let temp = self.board[m_piece.i][m_piece.j];
        self.board[m_piece.goal_i][m_piece.goal_j] = temp;
        self.board[m_piece.i][m_piece.j] = 0;
    }
}

use std::cmp;
use std::collections::HashMap;
use std::collections::VecDeque;

pub type SolutionPath = rpds::Vector<MovePiece>;
impl LiBoard {
    // calculates the number of moves to optimally collect all stars
    // The idea is to perform a breadth first search till the desired move is found
    // TODO: make bidirectional BFS
    pub fn num_optimal_moves_to_star(&self) -> (i8, SolutionPath) {
        // pair of (num stars collected , board)
        let mut max_stacksize = 1;
        let mut visited: HashMap<[[i8; 8]; 8], i8> = HashMap::new();
        let mut current_queue = VecDeque::new();
        current_queue.reserve(100_000);
        let sol_path = rpds::Vector::new();

        current_queue.push_back((0, 0, self.board, self.main_piece.0, self.main_piece.1, sol_path.clone()));
        let mut min_num = i8::MAX;
        while !current_queue.is_empty() {
            max_stacksize = cmp::max(max_stacksize, current_queue.len());
            // get current board
            let mut cur_board = LiBoard {
                main_piece: (0, 0), // doesn't matter here
                num_star_cnt: 0,    // doesn't matter either
                board: current_queue.front().unwrap().2,
            };
            // if board is not in visited
            if let std::collections::hash_map::Entry::Vacant(e) = visited.entry(cur_board.board) {
                // add to visited
                let cur_starcount = current_queue.front().unwrap().0;
                let cur_move_count = current_queue.front().unwrap().1;
                let piece_ipos = current_queue.front().unwrap().3;
                let piece_jpos = current_queue.front().unwrap().4;
                let path = current_queue.front().unwrap().5.clone();
                e.insert(cur_move_count);
                if cur_starcount == self.num_star_cnt {
                    min_num = cmp::min(cur_move_count, min_num);
                    return (min_num, path);
                }
                current_queue.pop_front();
                use itertools::iproduct;
                for (k, l) in iproduct!(0..8, 0..8) {
                    let temp_move = MovePiece {
                        i: piece_ipos as usize,
                        j: piece_jpos as usize,
                        goal_i: k as usize,
                        goal_j: l as usize,
                    };
                    if (temp_move.i != temp_move.goal_i || temp_move.j != temp_move.goal_j)
                        && (cur_board.board[temp_move.i][temp_move.j] != 0 && cur_board.board[temp_move.i][temp_move.j] != STAR_VALUE && cur_board.validate_move(&temp_move).is_valid())
                    {
                        // add this to currentQueue
                        let backup = LiBoard { ..cur_board };
                        let star_flag = cur_board.board[temp_move.goal_i][temp_move.goal_j] == STAR_VALUE;

                        cur_board.update_board(&temp_move);

                        current_queue.push_back((
                            cur_starcount + star_flag as i8,
                            cur_move_count + 1,
                            cur_board.board,
                            temp_move.goal_i as i8,
                            temp_move.goal_j as i8,
                            path.push_back(temp_move.clone()).clone(),
                        ));

                        cur_board.board = backup.board;
                    }
                }
            } else {
                // Ignore if board has already been visited
                current_queue.pop_front();
            }
        }
        (min_num, sol_path)
    }
}

impl Default for LiBoard {
    fn default() -> Self {
        Self::new(5, QUEEN_WHITE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_optimal_calc() {
        let board = LiBoard {
            board: [
                [QUEEN_WHITE, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, STAR_VALUE],
            ],
            num_star_cnt: 1,
            main_piece: (0, 0),
        };
        assert_eq!(1, board.num_optimal_moves_to_star().0)
    }

    #[test]
    fn test_optimal_calc_2() {
        let board = LiBoard {
            board: [
                [QUEEN_WHITE, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, STAR_VALUE, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, STAR_VALUE],
            ],
            num_star_cnt: 2,
            main_piece: (0, 0),
        };
        assert_eq!(3, board.num_optimal_moves_to_star().0)
    }

    #[test]
    fn test_optimal_calc_3() {
        let board = LiBoard {
            board: [
                [QUEEN_WHITE, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, STAR_VALUE, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, STAR_VALUE, 0, 0, 0, STAR_VALUE, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, STAR_VALUE],
            ],
            num_star_cnt: 4,
            main_piece: (0, 0),
        };
        assert_eq!(5, board.num_optimal_moves_to_star().0)
    }

    #[test]
    fn test_optimal_calc_4() {
        let board = LiBoard {
            board: [
                [KNIGHT_WHITE, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, STAR_VALUE, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [STAR_VALUE, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, STAR_VALUE, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, STAR_VALUE],
            ],
            num_star_cnt: 4,
            main_piece: (0, 0),
        };
        assert_eq!(10, board.num_optimal_moves_to_star().0)
    }

    #[test]
    fn test_optimal_calc_5() {
        let board = LiBoard {
            board: [
                [QUEEN_WHITE, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, STAR_VALUE],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, STAR_VALUE, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, STAR_VALUE, STAR_VALUE, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [STAR_VALUE, 0, 0, 0, 0, 0, STAR_VALUE, 0],
            ],
            num_star_cnt: 6,
            main_piece: (0, 0),
        };
        assert_eq!(6, board.num_optimal_moves_to_star().0)
    }
}
