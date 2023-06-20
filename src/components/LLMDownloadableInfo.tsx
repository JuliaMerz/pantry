// src/components/LLMDownloadableInfo.tsx

import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/tauri';
import LLMInfo from './LLMInfo';
import { LinearProgress } from '@mui/material';
import Button from '@mui/material/Button';
import { LLMRegistry, LLMRegistryEntry, LLMDownloadState } from '../interfaces';
import { Store } from "tauri-plugin-store-api";
import React, { useEffect, useState } from 'react';

interface LLMDownloadableInfoProps {
  llm: LLMRegistryEntry,
  registry: LLMRegistry,
  beginDownload: (uuid: string) => void;
}


const LLMDownloadableInfo: React.FC<LLMDownloadableInfoProps> = ({ llm, registry, beginDownload }) => {
  const store = new Store(".local.dat");

  const [downloadProgress, setDownloadProgress] = useState('');
  const [downloadError, setDownloadError] = useState(false);
  const downloadClick = async () => {
    console.log("sending off the llm reg", llm);


    const result = await invoke('download_llm', {llmReg: llm});
    const backend_uuid = (result as any).data.uuid;
    beginDownload(backend_uuid);

  }
  console.log("llm", llm);


  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const errorCheck = (time: string) => {
      setTimeout(() => {
        if (downloadProgress === time) {
          setDownloadError(true)
        }
      });
    }
    errorCheck(downloadProgress);

    (async () => {
      console.log("registering listener", llm.backend_uuid);
      const unlisten = await listen('downloads', (event) => {
        if (event.payload.stream_id !== llm.id+'-'+llm.backend_uuid)
          return
        if (event.payload.event.type == "download_error") {
          setDownloadError(true)
          return
        }
        setDownloadProgress(event.payload.event.progress);
        setDownloadError(false);

        // Set a timer to set error to true if no updates after 5 seconds
        errorCheck(event.payload.event.progress);
      });
    })();

    return () => {
      unlisten && unlisten();
    }
  }, [llm.backend_uuid]);
  return (

    <div className="card available-llm">
      <LLMInfo llm={llm} rightButton={
          llm.download_state === LLMDownloadState.Downloading ? (
            downloadProgress ?
              (downloadError ?
                <div>Error: No update in 5 seconds. Please restart.</div> :
                <LinearProgress variant="determinate" value={parseInt(downloadProgress)} />)
              : <LinearProgress variant="indeterminate" />)
              : <Button variant="contained" onClick={downloadClick} >Download</Button>
      } />
      <div><b>Requirements:</b> {llm.requirements}</div>
      <div><b>User Parameters:</b> {llm.user_parameters.join(", ")}</div>
      <div><b>Capabilities:</b> {JSON.stringify(llm.capabilities)}</div>
      <div><b>Parameters:</b> {JSON.stringify(llm.parameters)}</div>
      <div><b>Config:</b> {JSON.stringify(llm.config)}</div>
    </div>
  );
};

export default LLMDownloadableInfo;

