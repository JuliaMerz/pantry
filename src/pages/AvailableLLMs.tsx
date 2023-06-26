// src/pages/AvailableLLMs.tsx

import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import Link from '@mui/material/Link';
import { LLMAvailable, toLLMAvailable } from '../interfaces';
import LLMAvailableInfo from '../components/LLMAvailableInfo';
import Switch from '@mui/material/Switch';

import {
  Box,
  Typography,
} from '@mui/material';

function AvailableLLMs() {
  const [availableLLMs, setAvailableLLMs] = useState<LLMAvailable[]>([]);

  useEffect(() => {
    const fetchAvailableLLMs = async () => {
      try {
        console.log("sending");
        const result: {data: LLMAvailable[]} = await invoke<{data: LLMAvailable[]}>('available_llms');
        console.log("resultsss", result);
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
    <Box>
      <Typography variant="h2">Available Large Language Models</Typography>
      {availableLLMs.map((llm) => (
        <LLMAvailableInfo llm={llm} key={llm.id}/>
        ))}
    </Box>
  );
}

export default AvailableLLMs;

