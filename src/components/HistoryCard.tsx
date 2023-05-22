// src/components/HistoryCard.tsx

import React, { useState } from 'react';

interface HistoryCardProps {
  input: string;
  output: string;
  caller: string;
  datetime: string;
}

function HistoryCard(props: HistoryCardProps) {
  const { input, output, caller, datetime } = props;
  const [showFull, setShowFull] = useState(false);

  return (
    <div onClick={() => setShowFull(!showFull)}>
      <p>Caller: {caller}</p>
      <p>Date/Time: {datetime}</p>
      <p>Input: {showFull ? input : input.substring(0, 100)}</p>
      <p>Output: {showFull ? output : output.substring(0, 100)}</p>
    </div>
  );
}

export default HistoryCard;

