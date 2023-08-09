// src/components/UserRequestInfo.tsx

import {
  Accordion, AccordionDetails, AccordionSummary, Box, Button, Card,
  CardContent, Grid,
  CircularProgress, Divider, Link, MenuItem, Paper,
  Select, Switch, Table, TableBody, TableCell, TableContainer, TableHead,
  TableRow, TextField, Typography
} from '@mui/material';
import React, {useEffect, useState} from 'react';
import LLMInfo from './LLMInfo';
import {invoke} from '@tauri-apps/api/tauri';
import {UserRequest, UserRequestType, UserPermissionRequest, UserDownloadRequest, UserLoadRequest, UserUnloadRequest} from '../interfaces';

interface UserRequestInfoProps {
  request: UserPermissionRequest | UserDownloadRequest | UserLoadRequest | UserUnloadRequest;
}

export function UserRequestInfo(props: UserRequestInfoProps) {
  const {request} = props;
  const [completed, setCompleted] = useState(false);

  const handleAcceptRequest = async () => {
    const result = await invoke('accept_request', {requestId: request.id});
    console.log("accepted", request.id, result);
    setCompleted(true);

  }

  const handleRejectRequest = async () => {
    const result = await invoke('reject_request', {requestId: request.id});
    console.log("rejected", request.id, result);
    setCompleted(true);

  }

  if (completed) {
    return (
      <Card variant="outlined" sx={{boxShadow: 1, p: 2, paddingTop: 0, marginBottom: 2}}>
        <CardContent>
          <Typography variant="h5">Request Completed</Typography>
        </CardContent>
      </Card >
    )
  }

  let inner = () => {
    let req;
    switch (request.type) {
      case UserRequestType.Permission:
        req = (request as UserPermissionRequest);
        return (

          <Box>
            <Typography variant="h5">Permission Request:</Typography>
            <TableContainer component={Paper}>
              <Table size="small" aria-label="llm details">
                <TableBody>
                  {Object.entries(req.request.requestedPermissions).map(([perm, value], index) => (
                    <TableRow key={index}>
                      <TableCell>{value ? <b>{perm}</b> : perm}</TableCell>
                      <TableCell>{value ? <b>Yes</b> : "No"}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
          </Box>
        )
      case UserRequestType.Download:
        req = (request as UserDownloadRequest);
        return (
          <Box>
            <Typography variant="h5">Download Request:</Typography>
            <LLMInfo llm={req.request.llmRegistryEntry} rightButton={null} />
          </Box>)
      case UserRequestType.Unload:
        req = (request as UserUnloadRequest);
        return (
          <Box>
            <Typography variant="h5">Unload Request:</Typography>
            <Typography>Unload {req.request.llmId}</Typography>
          </Box>

        )
      case UserRequestType.Load:
        req = (request as UserLoadRequest);
        return (
          <Box>
            <Typography variant="h5">Load Request:</Typography>
            <Typography>Load {req.request.llmId}</Typography>
          </Box>)

      default:
        return "ERROR";

    }
  }

  return (
    <Card variant="outlined" sx={{boxShadow: 1, p: 2, paddingTop: 0, marginBottom: 2}}>
      <CardContent>
        <Typography variant="h3"> Program "{request.originator}" </Typography>
        <Typography variant="subtitle2">{request.timestamp.toString()}</Typography>
        <Typography>Requests the following:</Typography>
        {inner()}
        <Button variant="contained" onClick={handleAcceptRequest}>Accept</Button>
        <Button variant="contained" color="error" onClick={handleRejectRequest}>Reject</Button>
      </CardContent>
    </Card >
  );
}


