use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::{Arc, Mutex};
use serde::Deserialize;
use rand::{Rng, SeedableRng};

const WALL: char = '⬜';
const UNDER_WALL: [char; 6] = ['⬜', '⬜', '⬜', '⬜', '⬜', '⬜'];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoardItem {
    Cookie,
    Milk,
    Empty,
}

impl BoardItem {
    fn from_str(s: &str) -> Result<Self, &'static str> {
        match s {
            "cookie" => Ok(Self::Cookie),
            "milk" => Ok(Self::Milk),
            _ => Err("Invalid board item"),
        }
    }

    const fn to_char(&self) -> char {
        match self {
            Self::Cookie => '🍪',
            Self::Milk => '🥛',
            Self::Empty => '⬛',
        }
    }
    
}

impl std::fmt::Display for BoardItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Board {
    pub board: [[BoardItem; 4]; 4],
}

impl Board {
    pub fn new() -> Self {
        let board = [
            [BoardItem::Empty, BoardItem::Empty, BoardItem::Empty, BoardItem::Empty],
            [BoardItem::Empty, BoardItem::Empty, BoardItem::Empty, BoardItem::Empty],
            [BoardItem::Empty, BoardItem::Empty, BoardItem::Empty, BoardItem::Empty],
            [BoardItem::Empty, BoardItem::Empty, BoardItem::Empty, BoardItem::Empty],
        ];
        Self { board }
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        for row in &self.board {
            s.push(WALL);
            for item in row {
                s.push(item.to_char());
            }
            s.push(WALL);
            s.push('\n');
        }
        s.push_str(&UNDER_WALL.iter().collect::<String>());
        s.push('\n');
        s
    }

    pub fn is_connect_4(&self, board_item: BoardItem) -> bool {
        for row in 0..4 {
            for col in 0..4 {
                if self.board[row][col] == board_item {
                    if col + 3 < 4 {
                        if self.board[row][col + 1] == board_item
                            && self.board[row][col + 2] == board_item
                            && self.board[row][col + 3] == board_item
                        {
                            return true;
                        }
                    }
                    if row + 3 < 4 {
                        if self.board[row + 1][col] == board_item
                            && self.board[row + 2][col] == board_item
                            && self.board[row + 3][col] == board_item
                        {
                            return true;
                        }
                    }
                    if row + 3 < 4 && col + 3 < 4 {
                        if self.board[row + 1][col + 1] == board_item
                            && self.board[row + 2][col + 2] == board_item
                            && self.board[row + 3][col + 3] == board_item
                        {
                            return true;
                        }
                    }
                    if row + 3 < 4 && col >= 3 {
                        if self.board[row + 1][col - 1] == board_item
                            && self.board[row + 2][col - 2] == board_item
                            && self.board[row + 3][col - 3] == board_item
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn is_no_winner(&self) -> bool {
        // すべてのマスが埋まっているかチェック
        let is_board_full = self.board.iter().all(|row| {
            row.iter().all(|item| *item != BoardItem::Empty)
        });
    
        // ボードが埋まっていて、かつどちらも勝利していない場合はno winner
        is_board_full 
            && !self.is_connect_4(BoardItem::Cookie) 
            && !self.is_connect_4(BoardItem::Milk)
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Clone)]
pub struct StateBoard {
    pub board: Arc<Mutex<Board>>,
    pub seed: Arc<Mutex<rand::rngs::StdRng>>,
}

pub async fn reset(
    State(state_board): State<StateBoard>,
) -> impl IntoResponse {
    let mut board = state_board.board.lock().unwrap();
    *board = Board::new();
    let mut seed = state_board.seed.lock().unwrap();
    *seed = rand::rngs::StdRng::seed_from_u64(2024);

    (StatusCode::OK, board.to_string())
}

#[derive(Deserialize)]
pub struct PlaceParams {
    pub team: String,
    pub column: i32,
}

pub async fn place(
    State(state_board): State<StateBoard>,
    Path(params): Path<PlaceParams>
) -> impl IntoResponse {
    let team = match BoardItem::from_str(&params.team) {
        Ok(team) => team,
        Err(_) => return (StatusCode::BAD_REQUEST, "".to_string()),
    };

    let column = if (0..4).contains(&(params.column - 1)) {
        (params.column - 1) as usize
    } else {
        return (StatusCode::BAD_REQUEST, "".to_string());
    };

    let mut board = state_board.board.lock().unwrap();

    if board.is_no_winner() {
        let result = format!("{}No winner.\n", board.to_string());
        return (StatusCode::SERVICE_UNAVAILABLE, result);
    }
    if board.is_connect_4(BoardItem::Cookie) {
        let result = format!("{}{} wins!\n", board.to_string(), BoardItem::Cookie.to_char());
        return (StatusCode::SERVICE_UNAVAILABLE, result);
    }
    if board.is_connect_4(BoardItem::Milk) {
        let result = format!("{}{} wins!\n", board.to_string(), BoardItem::Milk.to_char());
        return (StatusCode::SERVICE_UNAVAILABLE, result);
    }

    for row in (0..4).rev() {
        if board.board[row][column] == BoardItem::Empty {
            board.board[row][column] = team;
            if board.is_connect_4(team) {
                let result = format!("{}{} wins!\n", board.to_string(), team.to_char());
                return (StatusCode::OK, result);
            } else if board.is_no_winner() {
                let result = format!("{}No winner.\n", board.to_string());
                return (StatusCode::OK, result);
            } else {
                return (StatusCode::OK, board.to_string());
            }
        }
    }

    (StatusCode::SERVICE_UNAVAILABLE, "".to_string())

}

pub async fn get_board(
    State(state_board): State<StateBoard>,
) -> impl IntoResponse {
    let board = state_board.board.lock().unwrap();
    
    let result = if board.is_no_winner() {
        format!("{}No winner.\n", board.to_string())
    } else if board.is_connect_4(BoardItem::Cookie) {
        format!("{}{} wins!\n", board.to_string(), BoardItem::Cookie.to_char())
    } else if board.is_connect_4(BoardItem::Milk) {
        format!("{}{} wins!\n", board.to_string(), BoardItem::Milk.to_char())
    } else {
        board.to_string()
    };

    (StatusCode::OK, result)
}

pub async fn rand_board(
    State(state_board): State<StateBoard>
) -> impl IntoResponse {
    let mut seed = state_board.seed.lock().unwrap();
    
    let mut board = state_board.board.lock().unwrap();

    // 左上から行ごとにmilkとcookieをランダムに配置
    for row in 0..4 {
        for col in 0..4 {
            let random = seed.gen::<bool>();
            board.board[row][col] = match random {
                true => BoardItem::Cookie,
                false => BoardItem::Milk,
            };
        }
    }

    if board.is_connect_4(BoardItem::Cookie) {
        let result = format!("{}{} wins!\n", board.to_string(), BoardItem::Cookie.to_char());
        return (StatusCode::OK, result);
    } else if board.is_connect_4(BoardItem::Milk) {
        let result = format!("{}{} wins!\n", board.to_string(), BoardItem::Milk.to_char());
        return (StatusCode::OK, result);
    } else {
        return (StatusCode::OK, board.to_string());
    }
}