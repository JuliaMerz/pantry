// src/pages/Home.tsx

import React, { useState, useEffect } from 'react';
import LLMLiveInfo from '../components/LLMLiveInfo';
import { invoke } from '@tauri-apps/api/tauri';
import { LLMActive, keysToCamelUnsafe, toLLMActive } from '../interfaces';

function Home() {
  const [activeLlms, setActiveLlms] = useState<LLMActive[]>([]);

  const rustGetLLMs = async (): Promise<{data: LLMActive[]}> => {
    const activeLLMs: {data: LLMActive[]} = await invoke('active_llms');
    return activeLLMs;
  };

  useEffect(() => {
    const fetchLLMs = async () => {
      const ret: {data: LLMActive[]} = await rustGetLLMs();
      console.log(ret.data);
      setActiveLlms(ret.data.map(toLLMActive));
    };

    fetchLLMs();
  }, []);

  return (
    <div>
      <h1>Home</h1>
      {console.log(activeLlms)}
      {activeLlms.map((llm) => (
        <LLMLiveInfo key={llm.id} {...llm} />
      ))}
    </div>
  );
}

export default Home;

