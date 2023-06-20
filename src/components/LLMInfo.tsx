// src/components/LLMInfo.tsx
import React from 'react';
import { ReactElement } from 'react';
import Link from '@mui/material/Link';
import Table from '@mui/material/Table';
import Paper from '@mui/material/Paper';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableContainer from '@mui/material/TableContainer';
import TableHead from '@mui/material/TableHead';
import TableRow from '@mui/material/TableRow';

import { useCollapse } from 'react-collapsed';


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
  const { getCollapseProps, getToggleProps, isExpanded } = useCollapse();


  return (
    <div className="llm-info" >
      <div className="split">
        <div className="left">
          <h2>
            {llm.name} <small>{llm.id}</small>
          </h2>
          <div>{llm.description}</div>
        </div>
        <div className="right">
          {rightButton}
        </div>
      </div>
      <div className="flex-row">
        <div><b>License</b> {llm.license}</div>
        <div><b>Model Family</b> {llm.family_id}</div>
        <div><b>Organization</b> {llm.organization}</div>
      </div>
      <div className="collapse-wrapper" >
      <div className="collapser" {...getToggleProps()}>{isExpanded ? '▼ Details' : '▶ Details'}</div>
      <div {...getCollapseProps()}>
          {isExpanded && (
            <div>
              <div>
                Additional Configs
                <div>
                  <h5>User Parameters</h5>
                  // Make this a table!
                {Object.keys(llm.user_parameters).length > 0 ? (
                  <TableContainer component={Paper}>
                  <Table size="small" className="llm-details-table" aria-label="llm details">
                    <TableHead><TableCell>Parameter</TableCell></TableHead>
                    <TableBody>
                  {llm.user_parameters.map((paramName, index) => (<TableRow> <TableCell>{paramName}</TableCell>
                              </TableRow>
                             ))
                  }
                   </TableBody></Table>
                  </TableContainer>
                  ) : (null)}
                </div>
              </div>
              <div>
                <h4>System Configs</h4>
                <div>
                  <h5>Parameters</h5>
                {Object.keys(llm.parameters).length > 0 ? (
                  <TableContainer component={Paper}>
                  <Table size="small" className="llm-details-table" aria-label="llm details">
                    <TableHead><TableCell>Parameter</TableCell><TableCell>Value</TableCell></TableHead>
                    <TableBody>
                  {Object.entries(llm.parameters).map(([paramName, paramValue], index) => (<TableRow> <TableCell>{paramName}</TableCell>
                              <TableCell>{paramValue}</TableCell>
                              </TableRow>
                             ))
                  }
                   </TableBody></Table>
                  </TableContainer>
                  ) : (null)}
                </div>

                <div><b>Connector Type: </b>{llm.connector_type}</div>

                <div>
                  <h5>Connector Config</h5>
                {Object.keys(llm.parameters).length > 0 ? (
                  <TableContainer component={Paper}>
                  <Table size="small" className="llm-details-table" aria-label="llm details">
                    <TableHead><TableCell>Parameter</TableCell><TableCell>Value</TableCell></TableHead>
                    <TableBody>
                  {Object.entries(llm.config).map(([paramName, paramValue], index) => (<TableRow> <TableCell>{paramName}</TableCell>
                              <TableCell>{paramValue}</TableCell>
                              </TableRow>
                             ))
                  }
                   </TableBody></Table>
                  </TableContainer>
                  ) : (null)}
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>

  );
};

export default LLMInfo;

