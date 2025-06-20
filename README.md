# Unbeatable Tic-Tac-Toe

A web-based, interactive Tic-Tac-Toe game where a human player competes against an unbeatable AI. This project features a Rust (Axum) backend for game logic and a React frontend for the user interface.

It is designed to handle multiple concurrent game sessions and can be run either locally for development or as a set of containerized services using Docker Compose.

## Features

* **Unbeatable AI:** The backend uses a minimax algorithm, ensuring the AI will never lose.

* **Multiple Concurrent Games:** The server manages a registry of active games, allowing for any number of simultaneous sessions.

* **Stateless Operation:** Finished games are automatically removed from server memory, requiring no cleanup tasks.

* **Decoupled Architecture:** The React SPA is hosted separately from the Rust backend server, communicating via a REST API.

## Tech Stack

* **Backend:** Rust, Axum, Tokio

* **Frontend:** React, JavaScript

* **Containerization:** Docker, Docker Compose, Nginx

## How to Run

There are two methods to run this application.

### Method 1: Running Locally (Without Docker)

#### Prerequisites

* [Rust & Cargo](https://www.rust-lang.org/tools/install)

* [Node.js & npm](https://nodejs.org/en/download/)

#### 1. Run the Backend Server

```
# Navigate to the backend directory
cd backend

# Run the server
cargo run

```

The server will start on `http://localhost:3000`.

#### 2. Run the Frontend Application

```
# In a new terminal, navigate to the frontend directory
cd frontend

# Install dependencies (first time only)
npm install

# Start the development server
npm start

```

The application will open in your browser at `http://localhost:3001`.

### Method 2: Running with Docker Compose

#### Prerequisites

* [Docker & Docker Compose](https://www.docker.com/products/docker-desktop/)

#### Instructions

From the root directory of the project, run the following command to start the services in the background:

```
docker-compose up -d

```

* This command will build the Docker images (if not already built) and start the containers in detached mode (`-d`).

* The frontend will be accessible in your browser at **`http://localhost:3001`**.

* To view the logs from both running services, use the command: `docker-compose logs -f`.

* To stop the services, run: `docker-compose down`.

## API Endpoints

The frontend communicates with the backend via two simple endpoints:

* **`POST /api/newgame`**: Creates a new game instance and returns its session ID.

* **`POST /api/games/{game_id}/move`**: Submits a player's move for a specific game session.

