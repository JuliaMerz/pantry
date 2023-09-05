// src/pages/Requests.tsx

import React, {useEffect, useState} from 'react';
import {invoke} from '@tauri-apps/api/tauri';
import {TextField, FormControlLabel, IconButton, Switch, CircularProgress, InputAdornment, Box, Typography, Stack} from '@mui/material';
import {UserRequestInfo} from '../components/LLMRequestInfo';
import {UserRequestType, UserRequest, toUserRequest, UserDownloadRequest, UserLoadRequest, UserUnloadRequest} from '../interfaces';

function Requests() {
  const [requests, setRequests] = useState<UserRequest[]>([]);

  useEffect(() => {
    const fetchRequests = async () => {

      const result: {data: UserRequest[]} = await invoke<{data: UserRequest[]}>('get_requests');
      console.log("requests", result);
      setRequests(result.data.map(toUserRequest).filter((val) => !val.complete));
    };

    fetchRequests();
  }, []);

  return (

    <Box>
      <Typography variant="h2">Program Requests</Typography>
      <Typography>Programs interfacing with the pantry API will show up here when they first request permission.
        They can also make requests here to download, activate, or deactivate models.</Typography>
      <Box>
        {requests.map((req) => {
          return (<UserRequestInfo key={req.id} request={req} />)
        }
        )}</Box>
    </Box>

  );
}

export default Requests;

