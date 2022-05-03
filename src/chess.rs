// constants from prev codebase
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

#[derive(Debug)]
pub struct MovePiece {
    pub i: usize,
    pub j: usize,
    pub goal_i: usize,
    pub goal_j: usize,
}

pub struct LiBoard {
    // 8x8 board
    pub board: [[i8; 8]; 8],
    pub main_piece: (i8, i8),
    pub num_star_cnt: i8,
}

impl LiBoard {
    // set up board randomly with n stars and choice piece
    pub fn new(star_cnt: i8, choice_piece: i8) -> LiBoard {
        use rand;
        let mut b = [[0; 8]; 8];
        let mut star_pairs = Vec::new();
        let mut already_added_stars = HashMap::new();

        let mut main_piece_i = rand::random::<u8>() % 8;
        let mut main_piece_j = rand::random::<u8>() % 8;
        already_added_stars.insert((main_piece_i, main_piece_j), 0);
        for v in 0..star_cnt {
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

    pub fn is_jumping_over_piece(&self, m_piece: &MovePiece) -> i8 {
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
                        if start as usize == m_piece.goal_j
                            && self.board[m_piece.goal_i][m_piece.goal_j] == 99
                        {
                            return 0;
                        }
                        return 1;
                    }
                    start += 1;
                }
                return 0;
            } else {
                let mut start = j - 1;
                while start >= m_piece.goal_j as i8 {
                    if self.board[i as usize][start as usize] != 0 {
                        if start == m_piece.goal_j as i8
                            && self.board[m_piece.goal_i][m_piece.goal_j] == 99
                        {
                            return 0;
                        }
                        return 1;
                    }
                    start -= 1;
                }
                return 0;
            }
        }
        // horizontal
        else if (m_piece.goal_j as i8 - j).abs() == 0 {
            if i < m_piece.goal_i as i8 {
                let mut start = i + 1;
                while start <= m_piece.goal_i as i8 {
                    if self.board[start as usize][j as usize] != 0 {
                        if start == m_piece.goal_i as i8
                            && self.board[m_piece.goal_i][m_piece.goal_j] == 99
                        {
                            return 0;
                        }
                        return 1;
                    }
                    start += 1;
                }
                return 0;
            } else {
                let mut start = i - 1;
                while start >= m_piece.goal_i as i8 {
                    if self.board[start as usize][j as usize] != 0 {
                        if start == m_piece.goal_i as i8
                            && self.board[m_piece.goal_i][m_piece.goal_j] == 99
                        {
                            return 0;
                        }
                        return 1;
                    }
                    start -= 1;
                }
                return 0;
            }
        }
        // everything else (diagonals)
        else {
            let mut temp = 0;
            if (j < m_piece.goal_j as i8) && (i < m_piece.goal_i as i8) {
                temp = i + 1;
                let mut start = j + 1;
                while start <= m_piece.goal_j as i8 {
                    if self.board[temp as usize][start as usize] != 0 {
                        if start == m_piece.goal_j as i8
                            && self.board[m_piece.goal_i][m_piece.goal_j] == 99
                        {
                            return 0;
                        }
                        return 1;
                    }
                    temp += 1;
                    start += 1;
                }
                return 0;
            } else if (j < m_piece.goal_j as i8) && (i > m_piece.goal_i as i8) {
                temp = i - 1;
                let mut start = j + 1;
                while start <= m_piece.goal_j as i8 {
                    if self.board[temp as usize][start as usize] != 0 {
                        if start == m_piece.goal_j as i8
                            && self.board[m_piece.goal_i][m_piece.goal_j] == 99
                        {
                            return 0;
                        }
                        return 1;
                    }
                    temp -= 1;
                    start += 1;
                }
                return 0;
            } else if (j > m_piece.goal_j as i8) && (i < m_piece.goal_i as i8) {
                temp = i + 1;
                let mut start = j - 1;
                while start >= m_piece.goal_j as i8 {
                    if self.board[temp as usize][start as usize] != 0 {
                        if start == m_piece.goal_j as i8
                            && self.board[m_piece.goal_i][m_piece.goal_j] == 99
                        {
                            return 0;
                        }
                        return 1;
                    }
                    temp += 1;
                    start -= 1;
                }
                return 0;
            } else {
                temp = i - 1;
                let mut start = j - 1;
                while start >= m_piece.goal_j as i8 {
                    if self.board[temp as usize][start as usize] != 0 {
                        if start == m_piece.goal_j as i8
                            && self.board[m_piece.goal_i][m_piece.goal_j] == 99
                        {
                            return 0;
                        }
                        return 1;
                    }
                    temp -= 1;
                    start -= 1;
                }
                return 0;
            }
        }
    }
    pub fn validate_move_rook(&self, m_piece: &MovePiece) -> i8 {
        // make sure goal_i , goal_j reaches the first blocking piece
        let i = m_piece.i as i8;
        let j = m_piece.j as i8;

        if ((m_piece.goal_i as i8 - i).abs() == 0) || ((m_piece.goal_j as i8 - j).abs() == 0) {
            if self.is_jumping_over_piece(m_piece) == 0 {
                return 1;
            }
        }
        return 0;
    }
    pub fn validate_move_bishop(&self, m_piece: &MovePiece) -> i8 {
        let i = m_piece.i as i8;
        let j = m_piece.j as i8;
        // part one of validity
        if (m_piece.goal_i as i8 - i).abs() == (m_piece.goal_j as i8 - j).abs() {
            if self.is_jumping_over_piece(m_piece) == 0 {
                return 1;
            }
        }
        0
    }
    pub fn validate_move_queen(&self, m_piece: &MovePiece) -> i8 {
        let i = m_piece.i as i8;
        let j = m_piece.j as i8;
        // part one of validity
        if ((m_piece.goal_i as i8 - i).abs() == 0)
            || ((m_piece.goal_j as i8 - j).abs() == 0)
            || ((m_piece.goal_i as i8 - i).abs() == (m_piece.goal_j as i8 - j).abs())
        {
            if self.is_jumping_over_piece(m_piece) == 0 {
                return 1;
            }
        }
        0
    }

    pub fn validate_move_knight(&self, m_piece: &MovePiece) -> i8 {
        let i = m_piece.i as i8;
        let j = m_piece.j as i8;
        // part one of validity
        // TODO refactor
        if ((m_piece.goal_i as i8 - i).abs() == 2 && (m_piece.goal_j as i8 - j).abs() == 1)
            || ((m_piece.goal_i as i8 - i).abs() == 1 && (m_piece.goal_j as i8 - j).abs() == 2)
        {
            return 50;
        }
        0
    }

    pub fn validate_move(&self, m_piece: &MovePiece) -> i8 {
        // leap of faith
        // if the piece that we are trying to move exists
        let mut move_was_valid = 0;
        if m_piece.i == m_piece.goal_i && m_piece.j == m_piece.goal_j {
            return 0;
        }
        // check for out of bounds
        if m_piece.goal_i >= 8 || m_piece.goal_j >= 8 {
            return 0;
        }
        if self.board[m_piece.i][m_piece.j] > 0 {
            match self.board[m_piece.i][m_piece.j] {
                1 | 2 => return 1,
                // move_was_valid = validateMovePawn(m_piece);
                3 | 12 => move_was_valid = self.validate_move_bishop(m_piece),

                4 | 11 => move_was_valid = self.validate_move_knight(m_piece),

                5 | 10 => move_was_valid = self.validate_move_rook(m_piece),

                7 | 14 => return 1,

                6 | 13 => move_was_valid = self.validate_move_queen(m_piece),

                _ => (),
            }

            if move_was_valid != 0 {
                return 1;
            } else {
                return 0;
            }
        }
        0
    }
    pub fn update_board(&mut self, m_piece: &MovePiece) {
        let temp = self.board[m_piece.i][m_piece.j];
        self.board[m_piece.goal_i][m_piece.goal_j] = temp;
        self.board[m_piece.i][m_piece.j] = 0;
    }
}

use std::cmp;
use std::collections::VecDeque;
use std::{collections::HashMap, hash::Hash};

impl LiBoard {
    // returns all the possible board combinations
    // optimal way would be to see where we go with the current piece
    // slight optimization before that would be to just start from that point
    pub fn possible_moves(i: i8, j: i8) -> Vec<MovePiece> {
        let mut moves = Vec::new();
        for k in 0..8 {
            for l in 0..8 {
                if i != k || j != l {
                    moves.push(MovePiece {
                        i: i as usize,
                        j: j as usize,
                        goal_i: k as usize,
                        goal_j: l as usize,
                    });
                }
            }
        }
        return moves;
    }
    // calculates the number of moves to optimally collect all stars
    // The idea is to perform a breadth first search till the desired move is found
    // TODO: make bidirectional BFS
    pub fn num_optimal_moves_to_star(&self) -> i8 {
        // pair of (num stars collected , board)
        let mut max_stacksize = 1;
        let mut visited: HashMap<[[i8; 8]; 8], i8> = HashMap::new();
        let mut current_queue = VecDeque::new();

        current_queue.push_back((0, 0, self.board, self.main_piece.0, self.main_piece.1));
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
            if !visited.contains_key(&cur_board.board) {
                // add to visited
                let cur_starcount = current_queue.front().unwrap().0;
                let cur_move_count = current_queue.front().unwrap().1;
                let piece_ipos = current_queue.front().unwrap().3;
                let piece_jpos = current_queue.front().unwrap().4;
                visited.insert(cur_board.board, cur_move_count);
                if cur_starcount == self.num_star_cnt {
                    min_num = cmp::min(current_queue.front().unwrap().1, min_num);
                    return min_num;
                }
                current_queue.pop_front();
                for temp_move in LiBoard::possible_moves(piece_ipos, piece_jpos) {
                    if (temp_move.i != temp_move.goal_i || temp_move.j != temp_move.goal_j)
                        && (cur_board.board[temp_move.i][temp_move.j] != 0
                            && cur_board.board[temp_move.i][temp_move.j] != STAR_VALUE
                            && cur_board.validate_move(&temp_move) != 0)
                    {
                        // add this to currentQueue
                        let backup = LiBoard { ..cur_board };
                        let star_flag;
                        if cur_board.board[temp_move.goal_i][temp_move.goal_j] == STAR_VALUE {
                            star_flag = 1;
                        } else {
                            star_flag = 0;
                        }

                        cur_board.update_board(&temp_move);

                        current_queue.push_back((
                            cur_starcount + star_flag,
                            cur_move_count + 1,
                            cur_board.board,
                            temp_move.goal_i as i8,
                            temp_move.goal_j as i8,
                        ));

                        cur_board.board = backup.board;
                    }
                }
            } else {
                // if position of board is
                current_queue.pop_front();
            }
        }
        // println!("size of visited {}", visited.len());
        return min_num;
    }
}

impl Default for LiBoard {
    fn default() -> Self {
        Self::new(5, QUEEN_WHITE)
    }
}
