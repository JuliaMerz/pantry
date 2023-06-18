// src/components/LLMDownloadableInfo.tsx

import React from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import LLMInfo from './LLMInfo';
import Button from '@mui/material/Button';
import { LLMRegistry, LLMRegistryEntry } from '../interfaces';

interface LLMDownloadableInfoProps {
  llm: LLMRegistryEntry,
  registry: LLMRegistry,
}

const LLMDownloadableInfo: React.FC<LLMDownloadableInfoProps> = ({ llm }) => {
  const downloadClick = async () => {
    console.log("sending off the llm reg", llm);
    const result = await invoke('download_llm', {llmReg: llm});

  }
  return (

    <div className="card available-llm">
      <LLMInfo llm={llm} rightButton={<Button variant="contained" onClick={downloadClick} >Download</Button>} />
      <div><b>Requirements:</b> {llm.requirements}</div>
      <div><b>User Parameters:</b> {llm.user_parameters.join(", ")}</div>
      <div><b>Capabilities:</b> {JSON.stringify(llm.capabilities)}</div>
      <div><b>Parameters:</b> {JSON.stringify(llm.parameters)}</div>
      <div><b>Config:</b> {JSON.stringify(llm.config)}</div>
    </div>
  );
};

export default LLMDownloadableInfo;

