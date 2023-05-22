// src/pages/AvailableLLMs.tsx

import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { LLMAvailable } from '../interfaces';
import LLMInfo from '../components/LLMInfo';

function AvailableLLMs() {
  const [availableLLMs, setAvailableLLMs] = useState<LLMAvailable[]>([]);

  useEffect(() => {
    const fetchAvailableLLMs = async () => {
      try {
        const result = await invoke<LLMAvailable[]>('available_llms');
        setAvailableLLMs(result);
      } catch (err) {
        console.error(err);
      }
    };

    fetchAvailableLLMs();
  }, []);

  return (
    <div>
      <h1>Available Large Language Models</h1>
      {availableLLMs.map((llm) => (
        <LLMInfo key={llm.id} {...llm} />
      ))}
    </div>
  );
}

export default AvailableLLMs;

