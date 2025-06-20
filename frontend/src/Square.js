import React from 'react';

// This helper function safely extracts the 'X' or 'O' from the backend's Cell enum format.
// The backend serializes `Cell::Occupied(Player::X)` to `{"Occupied":"X"}`.
const getDisplayValue = (cell) => {
  if (typeof cell === 'object' && cell.Occupied) {
    return cell.Occupied;
  }
  return ''; // For "Empty"
};

function Square({ cell, onClick, disabled }) {
  const value = getDisplayValue(cell);
  const className = `square ${value.toLowerCase()}`;
  
  return (
    <button className={className} onClick={onClick} disabled={disabled}>
      {value}
    </button>
  );
}

export default Square;
