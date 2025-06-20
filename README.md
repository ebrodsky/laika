Unbeatable Tic-Tac-Toe
A web-based, interactive Tic-Tac-Toe game where a human player competes against an unbeatable AI. This project features a Rust (Axum) backend for game logic and a React frontend for the user interface.

It is designed to handle multiple concurrent game sessions, with each browser tab representing a unique game.

Features
Unbeatable AI: The backend uses a minimax algorithm, ensuring the AI will never lose.

Multiple Concurrent Games: The server manages a registry of active games, allowing for any number of simultaneous sessions.

Automatic Cleanup: Finished games are automatically removed from server memory.

Decoupled Architecture: The React SPA is hosted separately from the Rust backend server, communicating via a REST API.

Tech Stack
Backend: Rust, Axum, Tokio

Frontend: React, JavaScript (ES6+)

Tooling: Cargo, Node.js/npm, create-react-app

How to Run
You must run both the backend and frontend in separate terminal windows.

Prerequisites
Rust & Cargo

Node.js & npm

1. Run the Backend Server
# Navigate to the backend directory
cd backend

# Run the server
cargo run

The server will start on http://localhost:3000.

2. Run the Frontend Application
# Navigate to the frontend directory
cd frontend

# Install dependencies (first time only)
npm install

# Start the development server
npm start

The application will open in your browser at http://localhost:3001.

API Endpoints
The frontend communicates with the backend via two simple endpoints:

POST /api/newgame: Creates a new game instance and returns its session ID.

POST /api/games/{game_id}/move: Submits a player's move for a specific game session.
