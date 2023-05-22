// src/pages/Home.tsx

import React, { useState, useEffect } from 'react';
import LLMLiveInfo from '../components/LLMLiveInfo';
import { invoke } from '@tauri-apps/api/tauri';
import { LLMActive } from '../interfaces';

function Home() {
  const [activeLlms, setActiveLlms] = useState<LLMActive[]>([]);

  const rustGetLLMs = async (): Promise<LLMActive[]> => {
    const activeLLMs: LLMActive[] = await invoke('active_llms');
    return activeLLMs;
  };

  useEffect(() => {
    const fetchLLMs = async () => {
      const data: LLMActive[] = await rustGetLLMs();
      setActiveLlms(data);
    };

    fetchLLMs();
  }, []);

  return (
    <div>
      <h1>Home</h1>
      {activeLlms.map((llm) => (
        <LLMLiveInfo key={llm.id} {...llm} />
      ))}
    </div>
  );
}

export default Home;

