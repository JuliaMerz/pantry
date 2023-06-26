// src/components/LLMInfo.tsx
import React from 'react';
import { ReactElement } from 'react';
import {
  Accordion,
  AccordionSummary,
  AccordionDetails,
  Card,
  CardContent,
  Link,
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




import { LLM, LLMRegistryEntry } from '../interfaces';

import { invoke } from '@tauri-apps/api/tauri';


type LLMInfoProps = {
  llm: LLM|LLMRegistryEntry,
  rightButton: ReactElement<any>;
  //MAKE THE BUTTON APPEAR
  //THEN DO THE DOWNLOAD ITSELF
};

//         <Route path="/history/:id" component={History} />

const LLMInfo: React.FC<LLMInfoProps> = ({
  llm,
  rightButton
}) => {

  const [expanded, setExpanded] = React.useState<string | false>(false);
  const handleAccordion =
  (panel: string) => (event: React.SyntheticEvent, newExpanded: boolean) => {
    setExpanded(newExpanded ? panel : false);
  };



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
      <Box sx={{display: 'flex', gap: 2, mb: 2}}>
        <Typography><b>License:</b> {llm.license}</Typography>
        <Typography><b>Model Family:</b> {llm.familyId}</Typography>
        <Typography><b>Organization:</b> {llm.organization}</Typography>
      </Box>
      <Box>
        <Accordion variant="innerCard" expanded={expanded === 'interface'} onChange={handleAccordion('interface')}>
          <AccordionSummary variant="innerCard" aria-controls="panel1d-content" id="panel1d-header">
            <Typography>Details</Typography>
          </AccordionSummary>
          <AccordionDetails variant="innerCard">
            <Box>
              <Typography variant="h4">Additional Configs</Typography>
              <Typography variant="h5">User Parameters</Typography>
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
              <Typography variant="h5">System Configs</Typography>
              <Typography variant="h6">Parameters</Typography>
              {Object.keys(llm.parameters).length > 0 ? (
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
                </TableContainer>
              ) : null}
              <Typography variant="body1">
                <strong>Connector Type: </strong>{llm.connectorType}
              </Typography>
              <Typography variant="h6">Connector Config</Typography>
              {Object.keys(llm.parameters).length > 0 ? (
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
                </TableContainer>
              ) : null}
            </Box>
          </AccordionDetails>
        </Accordion>
    </Box>
  </Box>

  );
};

export default LLMInfo;

