import React, { useState, useEffect, useCallback } from 'react';
import Board from './Board';
import Status from './Status';
import './App.css';

const API_BASE_URL = 'http://localhost:3000/api';

const initialGameState = {
  board: [['Empty', 'Empty', 'Empty'], ['Empty', 'Empty', 'Empty'], ['Empty', 'Empty', 'Empty']],
  status: 'InProgress',
  to_play: 'X',
};

function App() {
  const [gameId, setGameId] = useState(null);
  const [gameState, setGameState] = useState(initialGameState);
  const [error, setError] = useState(null);
  const [isLoading, setIsLoading] = useState(true);

  // Create a new game
  const startNewGame = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await fetch(`${API_BASE_URL}/newgame`, {
        method: 'POST',
      });
      if (!response.ok) {
        throw new Error('Failed to fetch a new game from the server.');
      }
      const { game_id, game_state } = await response.json();
      setGameId(game_id); // We use the same game_id for the entire game session
      setGameState(game_state);
    } catch (err) {
      console.error('Failed to create a new game:', err);
      setError('Could not connect to the server. Please ensure it is running and try again.');
    } finally {
      setIsLoading(false);
    }
  }, []);

  // This hook runs once when the component mounts to start the first game.
  useEffect(() => {
    startNewGame();
  }, [startNewGame]);

  // Handles a player's click on a square.
  const handlePlayerMove = async (row, col) => {
    if (!gameId || gameState.status !== 'InProgress' || gameState.to_play !== 'X') {
      return;
    }
    const moveUrl = `${API_BASE_URL}/games/${gameId}/move`; // All our moves correspond to the same game_id
    try {
      const response = await fetch(moveUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ row, col }),
      });

      if (!response.ok) {
        if (response.status === 404) {
             setError("This game has already ended. Please start a new one.");
        } else {
            const errorText = await response.text();
            throw new Error(`Invalid move: ${errorText}`);
        }
        setGameId(null);
      } else {
        const updatedState = await response.json();
        setGameState(updatedState);
        setError(null);
      }
    } catch (err) {
      console.error('Error making move:', err);
      setError(err.message);
    }
  };
  
  // Handles the "Play Again" button click by starting a new game.
  const handlePlayAgain = () => {
    startNewGame(); // Just calls `startNewGame` which sets a new game id and resets the game state
  };
  
  if (isLoading) {
    return <div className="app"><h1>Loading new game...</h1></div>;
  }

  return (
    <div className="app">
      <header>
        <h1>Tic-Tac-No</h1>
        <h3>You're not gonna win</h3>
        {gameId && gameState.status === 'InProgress' && <p style={{color: "#aaa", fontSize: "0.8rem"}}>Game ID: {gameId}</p>}
      </header>
      <main>
        <Board board={gameState.board} onSquareClick={handlePlayerMove} disabled={gameState.status !== 'InProgress' || gameState.to_play !== 'X'} />
        <Status status={gameState.status} />
        {error && <p className="error" style={{ color: 'red' }}>{error}</p>}
        {gameState.status !== 'InProgress' && !isLoading && (
          <button className="reset-button" onClick={handlePlayAgain}>
            Play Again
          </button>
        )}
      </main>
    </div>
  );
}

export default App;
