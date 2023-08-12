// src/components/LLMInfo.tsx
import {forwardRef, useState, useMemo, useContext, useEffect} from "react";
import ContentCopyIcon from '@mui/icons-material/ContentCopy';
import {Buffer} from 'buffer';
import React from 'react';
import ReactJsonView from 'react-json-view';
import {ReactElement} from 'react';
import {InnerCard} from './InnerCard';
import {writeText, readText} from '@tauri-apps/api/clipboard';
import {ModalBox} from '../theme';
import {
  Accordion,
  AccordionSummary,
  AccordionDetails,
  Button,
  Card,
  CardContent,
  Link,
  Modal,
  Table,
  Paper,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Box,
  Typography,
} from '@mui/material';




import {LLM, LLMRegistryEntry, } from '../interfaces';

import {invoke} from '@tauri-apps/api/tauri';
import {ColorContext} from "../theme";


function jsonToBase64(object: any) {
  const json = JSON.stringify(object);
  return Buffer.from(json).toString("base64");
}

function base64ToJson(base64String: string) {
  const json = Buffer.from(base64String, "base64").toString();
  return JSON.parse(json);
}


type LLMInfoProps = {
  llm: LLM | LLMRegistryEntry,
  rightButton: ReactElement<any> | null;
  //MAKE THE BUTTON APPEAR
  //THEN DO THE DOWNLOAD ITSELF
};

//         <Route path="/history/:id" component={History} />

const LLMInfo: React.FC<LLMInfoProps> = ({
  llm,
  rightButton
}) => {
  const [shareModalOpen, setShareModalOpen] = useState(false);
  const [jsonEdition, setJSONEdition] = useState(llm); //Deep copy hack because it wants json.
  const [success, setSuccess] = useState(false);
  const colorMode = useContext(ColorContext);

  const shareToClipboard = async () => {
    await writeText(`pantry://download/${jsonToBase64(jsonEdition)}`);
    setSuccess(true)
    setTimeout(() => setSuccess(false), 200);
  }



  return (
    <Box>
      <Box sx={{display: 'flex', justifyContent: 'space-between', mb: 2}}>
        <Box sx={{flexGrow: 3}}>
          <Typography variant="h3">
            {llm.name} </Typography>
          <Typography variant="subtitle2">{llm.id}
          </Typography>
          <Typography variant="body1">{llm.description}</Typography>
        </Box>
        <Box sx={{display: 'flex', justifyContent: 'flex-end', alignItems: "center", minWidth: '100px', flexGrow: 1}}>
          {rightButton}
        </Box>
      </Box>
      <Box sx={{display: 'flex', gap: 2, mb: 2, alignItems: "center", justifyContent: 'space-between'}}>
        <Typography><b>License:</b> {llm.license}</Typography>
        <Typography><b>Model Family:</b> {llm.familyId}</Typography>
        <Typography><b>Organization:</b> {llm.organization}</Typography>
        <Button sx={{
          margin: 0,
          justify: 'right',
        }} onClick={() => setShareModalOpen(true)} variant="outlined">Share</Button>
      </Box>
      <Box>
        <InnerCard title={"Details"}>
          <Box>
            <Typography variant="h4">Additional Configs</Typography>
            {Object.keys(llm.userParameters).length > 0 || Object.keys(llm.userSessionParameters).length > 0 ? (
              <Typography variant="h5">User Configurable Parameters</Typography>
            ) : null}
            {Object.keys(llm.userParameters).length > 0 ? (
              <TableContainer component={Paper}>
                <Table size="small" aria-label="llm details">
                  <TableHead>
                    <TableRow>
                      <TableCell>Parameter</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {llm.userParameters.map((paramName, index) => (
                      <TableRow key={index}>
                        <TableCell>{paramName}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
            ) : null}
            {Object.keys(llm.userSessionParameters).length > 0 ? (
              <TableContainer component={Paper}>
                <Table size="small" aria-label="llm details">
                  <TableHead>
                    <TableRow>
                      <TableCell>Parameter</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {llm.userSessionParameters.map((paramName, index) => (
                      <TableRow key={index}>
                        <TableCell>{paramName}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
            ) : null}
            <Typography variant="h5">System Configs</Typography>
            {Object.keys(llm.sessionParameters).length > 0 ? (
              <>
                <Typography variant="h6">Default Session Parameters</Typography>
                <TableContainer component={Paper}>
                  <Table size="small" aria-label="llm details">
                    <TableHead>
                      <TableRow>
                        <TableCell>Parameter</TableCell>
                        <TableCell>Value</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {Object.entries(llm.sessionParameters).map(([paramName, paramValue], index) => (
                        <TableRow key={index}>
                          <TableCell>{paramName}</TableCell>
                          <TableCell>{paramValue}</TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </TableContainer></>
            ) : null}
            {Object.keys(llm.parameters).length > 0 ? (
              <>
                <Typography variant="h6">Default Inference Parameters</Typography>
                <TableContainer component={Paper}>
                  <Table size="small" aria-label="llm details">
                    <TableHead>
                      <TableRow>
                        <TableCell>Parameter</TableCell>
                        <TableCell>Value</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {Object.entries(llm.parameters).map(([paramName, paramValue], index) => (
                        <TableRow key={index}>
                          <TableCell>{paramName}</TableCell>
                          <TableCell>{paramValue}</TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </TableContainer></>
            ) : null}
            <Typography variant="body1">
              <strong>Connector Type: </strong>{llm.connectorType}
            </Typography>
            {Object.keys(llm.config).length > 0 ? (
              <>
                <Typography variant="h6">Connector Config</Typography>
                <TableContainer component={Paper}>
                  <Table size="small" aria-label="llm details">
                    <TableHead>
                      <TableRow>
                        <TableCell>Parameter</TableCell>
                        <TableCell>Value</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {Object.entries(llm.config).map(([paramName, paramValue], index) => (
                        <TableRow key={index}>
                          <TableCell>{paramName}</TableCell>
                          <TableCell>{paramValue}</TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </TableContainer></>
            ) : null}
          </Box>
        </InnerCard>

      </Box>
      <Modal open={shareModalOpen} onClose={() => setShareModalOpen(false)}>
        <ModalBox>
          <Card className="available-llm">
            <CardContent>

              <Typography variant="subtitle1">Click here to copy a deep link: </Typography>
              <Box sx={{
                display: 'flex', flexDirection: 'row',
                border: 1,
                padding: 1,
                marginBottom: 2,
                transition: success ? 'background-color 0.05s ease' : 'background-color 1.5s ease',
                bgcolor: success ? 'success.light' : 'info',
                color: success ? 'success.contrastText' : 'gray',
                overflow: 'hidden',
                cursor: 'pointer',
              }}>
                <ContentCopyIcon sx={{paddingRight: 1}} />
                <Typography sx={{
                  overflow: 'hidden',

                }}
                  onClick={shareToClipboard}>{`pantry://download/${jsonToBase64(jsonEdition)}`}</Typography></Box>
              <Typography variant="subtitle1">JSON View:</Typography>
              <ReactJsonView src={jsonEdition} theme={colorMode.color === 'light' ? 'apathy:inverted' : 'apathy'} />
            </CardContent>
          </Card>
        </ModalBox>
      </Modal>
    </Box>

  );
};

export default LLMInfo;

