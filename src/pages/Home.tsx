// src/pages/Home.tsx

import React, { useState, useEffect } from 'react';
import LLMRunningInfo from '../components/LLMRunningInfo';
import { invoke } from '@tauri-apps/api/tauri';
import { LLMRunning, keysToCamelUnsafe, toLLMRunning } from '../interfaces';

function Home() {
  const [activeLlms, setActiveLlms] = useState<LLMRunning[]>([]);

  const rustGetLLMs = async (): Promise<{data: LLMRunning[]}> => {
    const activeLLMs: {data: LLMRunning[]} = await invoke('active_llms');
    return activeLLMs;
  };

  useEffect(() => {
    const fetchLLMs = async () => {
      const ret: {data: LLMRunning[]} = await rustGetLLMs();
      console.log(ret.data);
      setActiveLlms(ret.data.map(toLLMRunning));
    };

    fetchLLMs();
  }, []);

  return (
    <div>
      <h1>Home</h1>
      {activeLlms.map((llm) => (
        <LLMRunningInfo key={llm.id} llm={llm} />
      ))}
    </div>
  );
}

export default Home;

