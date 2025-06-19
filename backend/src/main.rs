use std::net::SocketAddr;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};

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
    // Helper function to get the opposing player.
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

#[derive(Debug, Deserialize, Copy, Clone)]
struct PlayerMove {
    row: usize,
    col: usize,
}

impl GameState {
    fn check_status(&self) -> GameStatus {
        // Check for a win
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

        // Check for a draw
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

/// Attempts to make the move, checking for validity, whether the move finishes the game, and
/// updating the game state.
fn try_move(game_state: &mut GameState, player_move: PlayerMove) -> Result<(), Error> {
    if game_state.status != GameStatus::InProgress {
        return Err(Error::InvalidMove("Game is not in progress"));
    }

    let target_cell = &mut game_state.board[player_move.row][player_move.col];
    if *target_cell != Cell::Empty {
        return Err(Error::InvalidMove("Cell already occupied"));
    }

    *target_cell = Cell::Occupied(game_state.to_play);
    game_state.to_play = match game_state.to_play {
        Player::X => Player::O,
        Player::O => Player::X,
    };

    // Check for win or draw
    let game_status = game_state.check_status();

    game_state.status = game_status;

    Ok(())
}

fn minimax(game_state: &GameState) -> (i32, Option<PlayerMove>) {
    // Check for terminal states (win, lose, draw)
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

    // Iterate over all possible moves
    for r in 0..3 {
        for c in 0..3 {
            if game_state.board[r][c] == Cell::Empty {
                let mut new_state = *game_state;
                // We don't need `try_move` here because we've already checked the cell is empty.
                new_state.board[r][c] = Cell::Occupied(new_state.to_play);
                new_state.to_play = new_state.to_play.opponent();
                let (score, _) = minimax(&new_state);
                moves.push((score, PlayerMove { row: r, col: c }));
            }
        }
    }

    // AI is 'O', the minimizing player. It will choose the move with the lowest score.
    // Human is 'X', the maximizing player. It will choose the move with the highest score.
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
        return Err(Error::InvalidMove("Game is not in progress"));
    }

    let (_, optimal_move) = minimax(game_state);
    if let Some(player_move) = optimal_move {
        try_move(game_state, player_move)?;
        Ok(())
    } else {
        Err(Error::InvalidMove("No valid moves available"))
    }
}

/// Updates the state of the board, current player, and game status.
async fn update_game_state(
    State(mut game_state): State<GameState>,
    Json(player_move): Json<PlayerMove>,
) -> Result<Json<GameState>, crate::Error> {
    // process player move, validating it too
    // updates the game state accordingly
    try_move(&mut game_state, player_move)?;
    do_optimal_move(&mut game_state)?;

    Ok(Json(game_state))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/move", post(update_game_state))
        .with_state(GameState {
            board: [[Cell::Empty; 3]; 3],
            status: GameStatus::InProgress,
            to_play: Player::X, // X (human) starts the game
        });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
