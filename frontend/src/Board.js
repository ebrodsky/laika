import React from 'react';
import Square from './Square';

function Board({ board, onSquareClick, disabled }) {
  return (
    <div className="board">
      {board.flat().map((cell, index) => {
        const row = Math.floor(index / 3);
        const col = index % 3;
        
        // A cell is also disabled if it's already occupied
        const isOccupied = typeof cell === 'object' && cell.Occupied;

        return (
          <Square
            key={index}
            cell={cell}
            onClick={() => onSquareClick(row, col)}
            disabled={disabled || isOccupied}
          />
        );
      })}
    </div>
  );
}

export default Board;
