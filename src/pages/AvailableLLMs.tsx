// src/pages/AvailableLLMs.tsx

import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { LLMAvailable, toLLMAvailable } from '../interfaces';
import LLMInfo from '../components/LLMInfo';

function AvailableLLMs() {
  const [availableLLMs, setAvailableLLMs] = useState<LLMAvailable[]>([]);

  useEffect(() => {
    const fetchAvailableLLMs = async () => {
      try {
        console.log("sending");
        const result: {data: LLMAvailable[]} = await invoke<{data: LLMAvailable[]}>('available_llms');
        const res2: {[key:string]: String} = await invoke<{[key:string]: String}>('ping');
        console.log(res2);
        setAvailableLLMs(result.data.map(toLLMAvailable));
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

