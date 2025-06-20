import React from 'react';

// This helper parses the status object from the backend, e.g., `{"Win":"X"}`.
const getStatusMessage = (status) => {
  if (typeof status === 'object' && status.Win) {
    return status.Win === 'X' ? 'You Win! 🎉' : 'The AI Wins!';
  }
  if (status === 'Draw') {
    return "It's a Draw! 😑";
  }
  // This can be expanded to show whose turn it is
  return 'Your Turn (X)';
};


function Status({ status }) {
  const message = getStatusMessage(status);
  
  return (
    <div className="game-status">
      <h2>{message}</h2>
    </div>
  );
}

export default Status;
