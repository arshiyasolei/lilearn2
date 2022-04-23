
// constants from prev codebase
const PAWN_WHITE: i32  =  2;
const PAWN_BLACK: i32  = 1;
const ROOK_WHITE: i32  = 10;
const ROOK_BLACK: i32  = 5;
const KNIGHT_WHITE: i32  = 11;
const KNIGHT_BLACK: i32  = 4;
const QUEEN_WHITE: i32  = 13;
const QUEEN_BLACK: i32 =  6;
const BISHOP_WHITE: i32  = 12;
const BISHOP_BLACK: i32 =  3;
const KING_BLACK: i32 =  7;
const KING_WHITE: i32  = 14;
const STAR_VALUE: i32  = 99;

pub struct MovePiece {
    i: usize,
    j: usize,
    goal_i: usize,
    goal_j: usize
}

struct LiBoard {
    // 8x8 board
    board: [[i32; 8]; 8] 
}

impl LiBoard {

    // set up board randomly with n stars and choice piece 
    pub fn new(star_cnt: u8, choice_piece: i32) -> LiBoard {
        let mut b = [[0; 8] ; 8];
        LiBoard { board: b }
    }

    pub fn is_jumping_over_piece(&self, m_piece: &MovePiece) -> i32 {
        // get start piece pos
        let i = m_piece.i as i32;
        let j = m_piece.j as i32;

        // determine movement type
        // vertical
        if (m_piece.goal_i as i32 - i).abs() == 0 {
            if j < m_piece.goal_j as i32 {
                let mut start = j + 1;
                while start <= m_piece.goal_j as i32 {
                    if self.board[i as usize][start as usize] != 0 {
                        if start as usize == m_piece.goal_j &&
                            self.board[m_piece.goal_i][m_piece.goal_j] == 99 {
                            return 0;
                        }
                        return 1;
                    }
                    start += 1;
                }
                return 0;
            } else {
                let mut start = j - 1; 
                while  start >= m_piece.goal_j as i32 {
                    if self.board[i as usize][start as usize] != 0 {
                        if start == m_piece.goal_j as i32 &&
                            self.board[m_piece.goal_i][m_piece.goal_j] == 99 {
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
        else if (m_piece.goal_j as i32 - j).abs() == 0 {
            if i < m_piece.goal_i as i32 {
                let mut start = i + 1;
                while start <= m_piece.goal_i as i32 {
                    if self.board[start as usize][j as usize] != 0 {
                        if start == m_piece.goal_i as i32 &&
                            self.board[m_piece.goal_i][m_piece.goal_j] == 99 {
                            return 0;
                        }
                        return 1;
                    }
                    start += 1;
                }
                return 0;
            } else {
                let mut start = i - 1;
                while start >= m_piece.goal_i as i32 {
                    if self.board[start as usize][j as usize] != 0 {
                        if start == m_piece.goal_i as i32 &&
                            self.board[m_piece.goal_i][m_piece.goal_j] == 99 {
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
            if (j < m_piece.goal_j as i32) && (i < m_piece.goal_i as i32) {
                temp = i + 1;
                let mut start = j + 1;
                while  start <= m_piece.goal_j as i32 {
                    if self.board[temp as usize][start as usize] != 0 {
                        if start == m_piece.goal_j as i32 &&
                            self.board[m_piece.goal_i][m_piece.goal_j] == 99 {
                            return 0;
                        }
                        return 1;
                    }
                    temp += 1;
                    start += 1;
                }
                return 0;
            } else if (j < m_piece.goal_j as i32) && (i > m_piece.goal_i as i32) {
                temp = i - 1;
                let mut start = j + 1;
                while start <= m_piece.goal_j as i32 {
                    if self.board[temp as usize][start as usize] != 0 {
                        if start == m_piece.goal_j as i32 &&
                            self.board[m_piece.goal_i][m_piece.goal_j] == 99 {
                            return 0;
                        }
                        return 1;
                    }
                    temp -= 1;
                    start += 1;
                }
                return 0;
            } else if (j > m_piece.goal_j as i32) && (i < m_piece.goal_i as i32) {
                temp = i + 1;
                let mut start = j - 1;
                while start >= m_piece.goal_j as i32 {
                    if self.board[temp as usize][start as usize] != 0 {
                        if start == m_piece.goal_j as i32 &&
                            self.board[m_piece.goal_i][m_piece.goal_j] == 99 {
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
                while start >= m_piece.goal_j as i32 {
                    if self.board[temp as usize][start as usize] != 0 {
                        if start == m_piece.goal_j as i32 &&
                            self.board[m_piece.goal_i][m_piece.goal_j] == 99 {
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
    pub fn validate_move_rook(&self,m_piece: &MovePiece) -> i32 {
        // make sure goal_i , goal_j reaches the first blocking piece
        let i = m_piece.i as i32;
        let j = m_piece.j as i32;

        if ((m_piece.goal_i as i32 - i).abs() == 0) || ((m_piece.goal_j as i32 - j).abs() == 0) {
            if self.is_jumping_over_piece(m_piece) == 0 {
                return 1;
            }
        }
        return 0;
    }
    pub fn validate_move_bishop(&self,m_piece: &MovePiece) -> i32 {
        let i = m_piece.i as i32;
        let j = m_piece.j as i32;
        // part one of validity
        if (m_piece.goal_i as i32 - i).abs() == (m_piece.goal_j as i32 - j).abs() {
            if self.is_jumping_over_piece(m_piece) == 0 {
                return 1;
            }
        }
        0
    }
    pub fn validate_move_queen(&self,m_piece: &MovePiece) -> i32 {
        let i = m_piece.i as i32;
        let j = m_piece.j as i32;
        // part one of validity
        if ((m_piece.goal_i as i32 - i).abs() == 0) || ((m_piece.goal_j as i32 - j).abs() == 0) ||
            ((m_piece.goal_i as i32 - i).abs() == (m_piece.goal_j as i32 - j).abs()) {
            if self.is_jumping_over_piece(m_piece) == 0 {
                return 1;
            } 
        }
        0
    }


    pub fn validate_move_knight(&self,m_piece: &MovePiece) -> i32 {
        let i = m_piece.i as i32;
        let j = m_piece.j as i32;
        // part one of validity
        // TODO refactor
        if ((m_piece.goal_i as i32- i).abs() == 2 && (m_piece.goal_j as i32 - j).abs() == 1) ||
            ((m_piece.goal_i as i32 - i).abs() == 1 && (m_piece.goal_j as i32 - j).abs() == 2) {
                return 50;
        }
        0
    }

    pub fn validate_move(&self,m_piece: &MovePiece) -> i32 {
        // leap of faith
        // if the piece that we are trying to move exists
        let mut move_was_valid = 0;
        if self.board[m_piece.i][m_piece.j] > 0 {
            match self.board[m_piece.i][m_piece.j] {
                1 | 2 => return 1,
                    // move_was_valid = validateMovePawn(m_piece);
                3 | 12 => move_was_valid = self.validate_move_bishop(m_piece),

                4 | 11 => move_was_valid = self.validate_move_knight(m_piece),
                
                5 | 10 => move_was_valid = self.validate_move_rook(m_piece),

                7 | 14 => return 1,

                6 | 13 => move_was_valid = self.validate_move_queen(m_piece),

                _ => ()
            }

            if move_was_valid != 0 {
                return 1;
            } else {
                return 0;
            }
        }
        0
    }
    pub fn update_board(&mut self,m_piece: &MovePiece) {
        let temp = self.board[m_piece.i][m_piece.j];
        self.board[m_piece.goal_i][m_piece.goal_j] = temp;
        self.board[m_piece.i][m_piece.j] = 0;
    }
}

impl Default for LiBoard {
    fn default() -> Self {
        Self::new()
    }
}
