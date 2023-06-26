// src/pages/Home.tsx
import {
  Box,
  Typography,
} from '@mui/material';

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
  const fetchLLMs = async () => {
    const ret: {data: LLMRunning[]} = await rustGetLLMs();
    console.log(ret.data);
    setActiveLlms(ret.data.map(toLLMRunning));
  };


  useEffect(() => {
    fetchLLMs();
  }, []);

  return (
    <Box>
      <Typography variant="h2">Currently Running LLMs</Typography>
      {activeLlms.map((llm) => (
        <LLMRunningInfo key={llm.id} llm={llm} refreshFn={fetchLLMs}/>
      ))}
    </Box>
  );
}

export default Home;

