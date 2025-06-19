use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

enum Error {
    InvalidMove(&'static str),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Error::InvalidMove(msg) => (StatusCode::BAD_REQUEST, msg),
        };
        (status, error_message).into_response()
    }
}

static WINNING_LINES: [[(usize, usize); 3]; 8] = [
    [(0, 0), (0, 1), (0, 2)],
    [(1, 0), (1, 1), (1, 2)],
    [(2, 0), (2, 1), (2, 2)], // Rows
    [(0, 0), (1, 0), (2, 0)],
    [(0, 1), (1, 1), (2, 1)],
    [(0, 2), (1, 2), (2, 2)], // Columns
    [(0, 0), (1, 1), (2, 2)],
    [(0, 2), (1, 1), (2, 0)], // Diagonals
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Player {
    X,
    O,
}

impl Player {
    fn opponent(&self) -> Player {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Cell {
    Empty,
    Occupied(Player),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum GameStatus {
    InProgress,
    Draw,
    Win(Player),
}

type GameBoard = [[Cell; 3]; 3];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct GameState {
    board: GameBoard,
    status: GameStatus,
    to_play: Player,
}

impl std::fmt::Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.board {
            for cell in row {
                let symbol = match cell {
                    Cell::Empty => ".",
                    Cell::Occupied(Player::X) => "X",
                    Cell::Occupied(Player::O) => "O",
                };
                write!(f, "{} ", symbol)?;
            }
            writeln!(f)?;
        }
        writeln!(f, "Status: {:?}", self.status)?;
        writeln!(f, "Next to play: {:?}", self.to_play)
    }
}

impl GameState {
    fn check_status(&self) -> GameStatus {
        for line in &WINNING_LINES {
            let cells_in_line = [
                self.board[line[0].0][line[0].1],
                self.board[line[1].0][line[1].1],
                self.board[line[2].0][line[2].1],
            ];
            if cells_in_line[0] != Cell::Empty
                && cells_in_line[0] == cells_in_line[1]
                && cells_in_line[1] == cells_in_line[2]
            {
                if let Cell::Occupied(player) = cells_in_line[0] {
                    return GameStatus::Win(player);
                }
            }
        }

        if self
            .board
            .iter()
            .all(|row| row.iter().all(|&cell| cell != Cell::Empty))
        {
            return GameStatus::Draw;
        }

        GameStatus::InProgress
    }
}

#[derive(Debug, Deserialize, Copy, Clone)]
struct PlayerMove {
    row: usize,
    col: usize,
}

type AppState = Arc<RwLock<GameState>>;

fn try_move(
    game_state: &mut GameState,
    player: Player,
    player_move: PlayerMove,
) -> Result<(), Error> {
    if game_state.status != GameStatus::InProgress {
        return Err(Error::InvalidMove("Game is not in progress"));
    }

    if game_state.to_play != player {
        return Err(Error::InvalidMove("Not your turn"));
    }

    let target_cell = &mut game_state.board[player_move.row][player_move.col];
    if *target_cell != Cell::Empty {
        return Err(Error::InvalidMove("Cell already occupied"));
    }

    *target_cell = Cell::Occupied(game_state.to_play);
    game_state.to_play = game_state.to_play.opponent();
    game_state.status = game_state.check_status();

    Ok(())
}

fn minimax(game_state: &GameState) -> (i32, Option<PlayerMove>) {
    match game_state.check_status() {
        GameStatus::Win(winner) => {
            return if winner == Player::X {
                (10, None)
            } else {
                (-10, None)
            };
        }
        GameStatus::Draw => return (0, None),
        GameStatus::InProgress => (),
    }

    let mut moves = Vec::new();

    for r in 0..3 {
        for c in 0..3 {
            if game_state.board[r][c] == Cell::Empty {
                let mut new_state = *game_state;
                new_state.board[r][c] = Cell::Occupied(new_state.to_play);
                new_state.to_play = new_state.to_play.opponent();
                let (score, _) = minimax(&new_state);
                moves.push((score, PlayerMove { row: r, col: c }));
            }
        }
    }

    if game_state.to_play == Player::O {
        moves
            .iter()
            .min_by_key(|(score, _)| score)
            .map(|(s, m)| (*s, Some(*m)))
            .unwrap()
    } else {
        moves
            .iter()
            .max_by_key(|(score, _)| score)
            .map(|(s, m)| (*s, Some(*m)))
            .unwrap()
    }
}

fn do_optimal_move(game_state: &mut GameState) -> Result<(), Error> {
    if game_state.status != GameStatus::InProgress {
        return Ok(()); // Not an error, just nothing to do.
    }

    let (_, optimal_move) = minimax(game_state);
    if let Some(player_move) = optimal_move {
        try_move(game_state, Player::O, player_move)
    } else {
        Err(Error::InvalidMove("AI could not find a valid move"))
    }
}

async fn update_game_state(
    State(state): State<AppState>,
    Json(player_move): Json<PlayerMove>,
) -> Result<Json<GameState>, Error> {
    let mut game_state = state.write().await;

    try_move(&mut game_state, Player::X, player_move)?;

    if game_state.status == GameStatus::InProgress {
        do_optimal_move(&mut game_state)?;
    }

    println!("Updated game state:\n{}", game_state);
    Ok(Json(*game_state))
}

#[tokio::main]
async fn main() {
    let app_state = Arc::new(RwLock::new(GameState {
        board: [[Cell::Empty; 3]; 3],
        status: GameStatus::InProgress,
        to_play: Player::X,
    }));
    let app = Router::new()
        .route("/api/move", post(update_game_state))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
