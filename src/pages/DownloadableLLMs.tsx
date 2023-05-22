// src/pages/DownloadableLLMs.tsx

import React, { useEffect, useState } from 'react';
import { fetch } from '@tauri-apps/api/http';
import { LLMDownloadable } from '../interfaces';
import LLMOnlineInfo from '../components/LLMOnlineInfo';

const LLM_INFO_SOURCE = "http://example.com";

function DownloadableLLMs() {
  const [downloadableLLMs, setDownloadableLLMs] = useState<LLMDownloadable[]>([]);

  useEffect(() => {
    const fetchDownloadableLLMs = async () => {
      try {
        const response = await fetch(LLM_INFO_SOURCE);
        const data = await response.json();
        setDownloadableLLMs(data);
      } catch (err) {
        console.error(err);
      }
    };

    fetchDownloadableLLMs();
  }, []);

  return (
    <div>
      <h1>Downloadable Large Language Models</h1>
      {downloadableLLMs.map((llm) => (
        <LLMOnlineInfo key={llm.id} {...llm} />
      ))}
    </div>
  );
}

export default DownloadableLLMs;

