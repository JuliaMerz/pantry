// src/components/LLMDownloadableInfo.tsx

import {listen} from '@tauri-apps/api/event'
import {invoke} from '@tauri-apps/api/tauri';
import LLMInfo from './LLMInfo';
import {
  LinearProgress,
  Modal,
  Button,
  Card,
  CardContent,
  Typography,
  Box,
} from '@mui/material';
import {ModalBox} from '../theme';
import {LLMRegistry, LLMRegistryEntry, LLMDownloadState, fromLLMRegistryEntry} from '../interfaces';
import {deleteRegistryEntry} from '../registryHelpers';
import {Store} from "tauri-plugin-store-api";
import React, {useEffect, useState, useRef} from 'react';

interface LLMDownloadableInfoProps {
  llm: LLMRegistryEntry,
  registry: LLMRegistry,
  beginDownload: () => void;
  completeDownload: () => void;
}


const LLMDownloadableInfo: React.FC<LLMDownloadableInfoProps> = ({llm, registry, beginDownload, completeDownload}) => {
  const store = new Store(".local.dat");

  const [downloadProgress, setDownloadProgress] = useState('');
  const [deleted, setDeleted] = useState(false);
  const downloadRef = useRef(downloadProgress);
  const [downloadError, setDownloadError] = useState(false);
  const [openDelete, setOpenDelete] = useState(false);
  const downloadClick = async () => {
    console.log("sending off the llm reg", llm);


    setDownloadError(false);
    setDownloadProgress('0');

    // const result = await invoke('download_llm', {llmReg: fromLLMRegistryEntry(llm)});
    // const backendUuid = (result as any).data.uuid;
    beginDownload();

  }

  useEffect(() => {
    downloadRef.current = downloadProgress;
  }, [downloadProgress]);

  const handleOpenDelete = () => {
    setOpenDelete(true);
  };

  const handleCloseModal = () => {
    setOpenDelete(false);
  };
  const handleConfirmDelete = async () => {
    setOpenDelete(false);

    let result = deleteRegistryEntry(llm, registry);
    console.log("deleted", llm.id, result);
    setDeleted(true);
  };

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const errorCheck = (time: string) => {
      setTimeout(() => {
        if (downloadRef.current === time) {
          setDownloadError(true)
        }
      }, 5000);
    }
    errorCheck(downloadProgress);

    (async () => {
      console.log("registering listener", llm.backendUuid);
      unlisten = await listen('downloads', (event: any) => {
        if (event.payload.stream_id !== llm.id + '-' + llm.backendUuid)
          return
        if (event.payload.event.type == "DownloadError") {
          setDownloadError(true)
          return
        }
        if (event.payload.event.type == "DownloadCompletion") {
          setDownloadProgress('100');
          setDownloadError(false);
          completeDownload();
          return
        }

        if (event.payload.event.type == "ChannelClose") {
          unlisten ? unlisten() : null
          return
        }


        setDownloadProgress(event.payload.event.progress);
        setDownloadError(false);

        // Set a timer to set error to true if no updates after 5 seconds
        errorCheck(event.payload.event.progress);
      });
    })();

    return () => {
      console.log("UNREGISTERING LISTENER OR TRYING TO", unlisten)
      unlisten && unlisten();
    }
  }, [llm.backendUuid]);
  if (deleted)
    return (
      <Card className="available-llm">
        <CardContent>
          <Typography variant="h6">Deleted</Typography>
        </CardContent>
      </Card>
    )
  return (
    <Card className="available-llm">
      <CardContent>
        <LLMInfo llm={llm} rightButton={
          llm.downloadState === LLMDownloadState.Downloading ?
            (downloadError ?
              (<Box>
                <Typography className="error download-error" color="error">Error: No update in 5 seconds. Please restart.</Typography>
                <Button variant="contained" onClick={downloadClick} >Retry</Button>
              </Box>)
              :
              (downloadProgress ?
                <LinearProgress sx={{width: '100%'}} variant="determinate" value={parseInt(downloadProgress)} />
                : <LinearProgress sx={{width: '100%'}} variant="indeterminate" />))
            : <Button variant="contained" onClick={downloadClick} >Download</Button>
        } />
        <Typography variant="body1"><b>Requirements:</b> {llm.requirements}</Typography>
        <Typography variant="body1"><b>User Parameters:</b> {llm.userParameters.join(", ")}</Typography>
        <Typography variant="body1"><b>Capabilities:</b> {JSON.stringify(llm.capabilities)}</Typography>
        <Typography variant="body1"><b>Parameters:</b> {JSON.stringify(llm.parameters)}</Typography>
        <Typography variant="body1"><b>Session Parameters:</b> {JSON.stringify(llm.sessionParameters)}</Typography>
        <Typography variant="body1"><b>Config:</b> {JSON.stringify(llm.config)}</Typography>
        <Button variant="contained" onClick={handleOpenDelete} color="error">Delete</Button>

        <Modal
          open={openDelete}
          onClose={handleCloseModal}
          aria-labelledby="delete-confirmation-modal"
          aria-describedby="delete-confirmation-modal-description"
        >
          <ModalBox>
            <Card className="delete-llm">
              <CardContent>
                <Typography variant="h6" id="delete-confirmation-modal">Confirm Delete</Typography>
                <Typography variant="body1" id="delete-confirmation-modal-description">Are you sure you want to delete this item?</Typography>
                <Button variant="contained" onClick={handleConfirmDelete}>Yes</Button>
                <Button variant="outlined" onClick={handleCloseModal}>No</Button>
              </CardContent>
            </Card>
          </ModalBox>
        </Modal>
      </CardContent>
    </Card>
  );
};

export default LLMDownloadableInfo;

