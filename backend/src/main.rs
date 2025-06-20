use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

// --- Error Handling ---
#[derive(Debug)]
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

// --- Game Logic Constants and Types ---

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

// The state for a single game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct GameState {
    board: GameBoard,
    status: GameStatus,
    to_play: Player,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            board: [[Cell::Empty; 3]; 3],
            status: GameStatus::InProgress,
            to_play: Player::X,
        }
    }
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

// --- AI and Move Logic ---

#[derive(Debug, Deserialize, Copy, Clone)]
struct PlayerMove {
    row: usize,
    col: usize,
}

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
        // AI is minimizing
        moves
            .into_iter()
            .min_by_key(|(score, _)| *score)
            .map(|(s, m)| (s, Some(m)))
            .unwrap()
    } else {
        // Human is maximizing
        moves
            .into_iter()
            .max_by_key(|(score, _)| *score)
            .map(|(s, m)| (s, Some(m)))
            .unwrap()
    }
}

fn do_optimal_move(game_state: &mut GameState) -> Result<(), Error> {
    if game_state.status != GameStatus::InProgress {
        return Ok(());
    }

    let (_, optimal_move) = minimax(game_state);
    if let Some(player_move) = optimal_move {
        try_move(game_state, Player::O, player_move)
    } else {
        Err(Error::InvalidMove("AI could not find a valid move"))
    }
}

// --- Application State ---

// The shared application state: a map from a unique game ID to its state.
type GameRegistry = HashMap<Uuid, GameState>;
type AppState = Arc<RwLock<GameRegistry>>;

// --- API Handlers ---

/// Creates a new game, adds it to the registry, and returns the new game ID and state.
async fn new_game(State(state): State<AppState>) -> impl IntoResponse {
    let mut registry = state.write().await;
    let new_game_id = Uuid::new_v4();
    let new_game = GameState::default();

    registry.insert(new_game_id, new_game);

    log::info!("Created new game with id: {}", new_game_id);
    log::info!("Total number of games: {}", registry.len());

    Json(serde_json::json!({
        "game_id": new_game_id,
        "game_state": new_game
    }))
}

/// Updates a specific game state and removes it if the game is over.
async fn update_game_state(
    State(state): State<AppState>,
    Path(game_id): Path<Uuid>,
    Json(player_move): Json<PlayerMove>,
) -> Result<Json<GameState>, Response> {
    let mut registry = state.write().await;

    // We use `get_mut` to ensure we can modify the state.
    if let Some(mut game_state) = registry.get_mut(&game_id).copied() {
        try_move(&mut game_state, Player::X, player_move).map_err(|e| e.into_response())?;

        if game_state.status == GameStatus::InProgress {
            do_optimal_move(&mut game_state).map_err(|e| e.into_response())?;
        }

        // If the game is over, remove it from the registry.
        // Otherwise, update the state in the registry.
        if game_state.status != GameStatus::InProgress {
            registry.remove(&game_id);
            log::info!("Game {} finished and was removed.", game_id);
            log::info!("Total number of games after removal: {}", registry.len());
        } else {
            // Update the state in the registry
            *registry.get_mut(&game_id).unwrap() = game_state;
        }

        // Return the final or updated state to the client.
        Ok(Json(game_state))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            format!("Game with id {} not found", game_id),
        )
            .into_response())
    }
}

// --- Main Server Function ---

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Stdout)
        .init();
    // Initialize the shared state for the game registry.
    let app_state = Arc::new(RwLock::new(GameRegistry::new()));

    // Configure CORS to allow requests from the frontend server.
    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:3001"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(vec![axum::http::header::CONTENT_TYPE]);

    // Define the application routes.
    let app = Router::new()
        .route("/api/newgame", post(new_game))
        .route("/api/games/{game_id}/move", post(update_game_state))
        .with_state(app_state)
        .layer(cors);

    // Start the server.
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    log::info!("Server starting...");
    log::info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

#[cfg(test)]
mod tests {
    // Import everything from the parent module (your main.rs code)
    use super::*;
    use rand::rng;
    use rand::seq::IndexedRandom;

    /// This test plays 100 games with a random-move-making human player (X)
    /// and asserts that the AI (O) never loses.
    #[test]
    fn test_ai_is_unbeatable_over_100_random_games() {
        for i in 0..100 {
            println!("\n--- Starting Random Game #{} ---", i + 1);
            let mut game_state = GameState::default();
            let mut rng = rng();

            // Loop until the game is no longer in progress.
            while game_state.status == GameStatus::InProgress {
                // It's always the human's turn first.
                assert_eq!(game_state.to_play, Player::X);

                // --- Human's Turn (Player X) ---
                let mut available_moves = Vec::new();
                for r in 0..3 {
                    for c in 0..3 {
                        if game_state.board[r][c] == Cell::Empty {
                            available_moves.push(PlayerMove { row: r, col: c });
                        }
                    }
                }

                // If there are no moves, the game should already be over, but we break just in case.
                if available_moves.is_empty() {
                    break;
                }

                // Choose a random valid move for the human player.
                let human_move = *available_moves.choose(&mut rng).unwrap();
                println!(
                    "Human (X) plays at ({}, {})",
                    human_move.row, human_move.col
                );

                // Apply the human's move.
                try_move(&mut game_state, Player::X, human_move)
                    .expect("Human move should be valid");

                // Check if the human's move ended the game.
                if game_state.status != GameStatus::InProgress {
                    break;
                }

                // --- AI's Turn (Player O) ---
                assert_eq!(game_state.to_play, Player::O);
                println!("AI (O) is thinking...");

                // The AI makes its optimal move.
                do_optimal_move(&mut game_state).expect("AI move should be valid");
                println!("{}", game_state);
            }

            println!("Game Over. Final Status: {:?}", game_state.status);

            // --- THE CORE ASSERTION ---
            // The human player (X) should NEVER win.
            // The game can be a Draw or a Win for O.
            assert_ne!(
                game_state.status,
                GameStatus::Win(Player::X),
                "AI FAILED: The AI lost a game! Final board:\n{}",
                game_state
            );
        }
    }

    #[test]
    fn test_optimal_vs_optimal_is_always_a_draw() {
        println!("\n--- Starting Optimal vs Optimal Game ---");
        let mut game_state = GameState::default();

        while game_state.status == GameStatus::InProgress {
            // --- Player X's Turn (Optimal "Human") ---
            if game_state.to_play == Player::X {
                println!("Optimal Human (X) is thinking...");
                // We manually find and apply the best move for 'X' since
                // do_optimal_move is hardcoded for Player O.
                let (_, optimal_move_for_x) = minimax(&game_state);
                let player_move =
                    optimal_move_for_x.expect("Minimax should always find a move for X");

                try_move(&mut game_state, Player::X, player_move)
                    .expect("Optimal move for X should be valid");

                println!("{}", game_state);
            }

            // Check if Player X's move ended the game
            if game_state.status != GameStatus::InProgress {
                break;
            }

            // --- Player O's Turn (AI) ---
            if game_state.to_play == Player::O {
                println!("AI (O) is thinking...");
                // We can use the existing function here as it's designed for 'O'.
                do_optimal_move(&mut game_state).expect("Optimal move for O should be valid");
                println!("{}", game_state);
            }
        }

        println!("Game Over. Final Status: {:?}", game_state.status);

        // --- THE CORE ASSERTION ---
        // A game between two perfect players must result in a draw.
        assert_eq!(
            game_state.status,
            GameStatus::Draw,
            "MINIMAX FAILED: A game between two optimal players did not result in a draw! Final board:\n{}",
            game_state
        );
    }
}
