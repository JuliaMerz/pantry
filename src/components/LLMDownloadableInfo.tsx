// src/components/LLMDownloadableInfo.tsx

import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/tauri';
import LLMInfo from './LLMInfo';
import { LinearProgress } from '@mui/material';
import Button from '@mui/material/Button';
import { LLMRegistry, LLMRegistryEntry, LLMDownloadState } from '../interfaces';
import { Store } from "tauri-plugin-store-api";
import React, { useEffect, useState, useRef} from 'react';

interface LLMDownloadableInfoProps {
  llm: LLMRegistryEntry,
  registry: LLMRegistry,
  beginDownload: (uuid: string) => void;
  completeDownload: () => void;
}


const LLMDownloadableInfo: React.FC<LLMDownloadableInfoProps> = ({ llm, registry, beginDownload, completeDownload}) => {
  const store = new Store(".local.dat");

  const [downloadProgress, setDownloadProgress] = useState('');
  const downloadRef = useRef(downloadProgress);
  const [downloadError, setDownloadError] = useState(false);
  const downloadClick = async () => {
    console.log("sending off the llm reg", llm);


    setDownloadError(false);
    setDownloadProgress('0');

    const result = await invoke('download_llm', {llmReg: llm});
    const backendUuid = (result as any).data.uuid;
    beginDownload(backendUuid);

  }

  useEffect(() => {
    downloadRef.current = downloadProgress;
  }, [downloadProgress]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const errorCheck = (time: string) => {
      console.log("arm error");
      setTimeout(() => {
        console.log("error!", downloadRef.current, time);
        if (downloadRef.current === time) {
          console.log("huh")
          setDownloadError(true)
        }
      }, 5000);
    }
    errorCheck(downloadProgress);

    (async () => {
      console.log("registering listener", llm.backendUuid);
      const unlisten = await listen('downloads', (event) => {
        if (event.payload.streamId !== llm.id+'-'+llm.backendUuid)
          return
        if (event.payload.event.type == "DownloadError") {
          setDownloadError(true)
          return
        }
        console.log("setting download progress",  event.payload.event.type, event.payload.event.progress);
        if (event.payload.event.type == "DownloadCompletion") {
          setDownloadProgress('100');
          setDownloadError(false);
          completeDownload();
          return
        }

        if (event.payload.event.type == "ChannelClose") {
          unlisten()
          return
        }

        console.log("Setting...");

        setDownloadProgress(event.payload.event.progress);
        setDownloadError(false);

        // Set a timer to set error to true if no updates after 5 seconds
        errorCheck(event.payload.event.progress);
      });
    })();

    return () => {
      unlisten && unlisten();
    }
  }, [llm.backendUuid]);
  return (

    <div className="card available-llm">
      <LLMInfo llm={llm} rightButton={
        llm.downloadState === LLMDownloadState.Downloading ?
            (downloadError ?
                    (<div>
                      <div className="error download-error">Error: No update in 5 seconds. Please restart.</div>
                      <Button variant="contained" onClick={downloadClick} >Retry</Button>
                      </div>)
               :
              (downloadProgress ?
                <LinearProgress variant="determinate" value={parseInt(downloadProgress)} />
              : <LinearProgress variant="indeterminate" />))
              : <Button variant="contained" onClick={downloadClick} >Download</Button>
      } />
      <div><b>Requirements:</b> {llm.requirements}</div>
      <div><b>User Parameters:</b> {llm.userParameters.join(", ")}</div>
      <div><b>Capabilities:</b> {JSON.stringify(llm.capabilities)}</div>
      <div><b>Parameters:</b> {JSON.stringify(llm.parameters)}</div>
      <div><b>Config:</b> {JSON.stringify(llm.config)}</div>
    </div>
  );
};

export default LLMDownloadableInfo;

