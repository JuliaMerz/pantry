// src/pages/History.tsx

import React, { useEffect, useState } from 'react';
import { LLMAvailable } from '../interfaces';
import LLMInfo from '../components/LLMInfo';
import HistoryCard from '../components/HistoryCard';

interface HistoryProps extends LLMAvailable {}

function History(props: HistoryProps) {
  const [history, setHistory] = useState([]);

  useEffect(() => {
    // Replace with actual Tauri function invocation
    const fakeHistory = [
      {
        input: 'Long input...',
        output: 'Long output...',
        caller: 'Caller 1',
        datetime: '2023-05-19T10:20:30Z',
      },
      // More history items...
    ];

    setHistory(fakeHistory);
  }, []);

  return (
    <div>
      <LLMInfo {...props} />
      {history.map((item, index) => (
        <HistoryCard key={index} {...item} />
      ))}
    </div>
  );
}

export default History;

