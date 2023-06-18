// src/components/LLMRunningInfo.tsx
import Link from '@mui/material/Link';

import Switch from '@mui/material/Switch';
import Table from '@mui/material/Table';
import Paper from '@mui/material/Paper';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableContainer from '@mui/material/TableContainer';
import TableHead from '@mui/material/TableHead';
import TableRow from '@mui/material/TableRow';

import React, { useState, useEffect } from 'react';
import { useCollapse } from 'react-collapsed';
import { LLMRunning, LLMResponse, toLLMResponse } from '../interfaces';
import LLMInfo from './LLMInfo';

import { invoke } from '@tauri-apps/api/tauri';
import { Store } from "tauri-plugin-store-api";
// import { Link } from 'react-router-dom';

// Define new types for history and user parameters

type HistoryItem = {
  timestamp: Date,
  parameters: {[name: string]: string},
  input: string,
  output: string,
};

type LLMRunningInfoProps = {
  llm: LLMRunning
};

const coerceInput = (input: string): any => {
  if (input.trim() === "") return undefined
  if (input.trim() === "true") return true;
  if (input.trim() === "false") return false;

  const num = parseFloat(input);
  if (!isNaN(num)) return num;


  try {
    const parsedJson = JSON.parse(input);
    return parsedJson;
  } catch (error) {
    return input;
  }
}

const LLMRunningInfo: React.FC<LLMRunningInfoProps> = ({
  llm
}) => {
  console.log(llm);
  const [checked, setChecked] = useState(true);
  const [userParametersState, setUserParametersState] = useState<{[id: string]: any}>(Object.fromEntries(llm.user_parameters.map((val)=>[val, undefined])));
  const [message, setMessage] = useState("");
  const { getCollapseProps, getToggleProps, isExpanded } = useCollapse();
  const [history, setHistory] = useState<HistoryItem[]>([]);
  const [error, setError] = useState("");
  const store = new Store('.local.dat');

  useEffect(() => {
    if (isExpanded) {
      fetchHistory();
    }
  }, [isExpanded]);

  const fetchHistory = async () => {
    store.get(`${llm.id}-history`).then((hist) => {
      const storedHistory: HistoryItem[] = hist ? hist as HistoryItem[] : [];
      setHistory(storedHistory || []);
    }).catch((err:any) => {
      setHistory([]);
    })


  };

  const handleToggle = async () => {
    console.log("Disable the LLM");
    setChecked(!checked);
    const result = await invoke('unload_llm', {id: llm.id});
  };

  const handleParameterChange = (name: string, value: string) => {
    const newUserParametersState = {...userParametersState};
    newUserParametersState[name] = coerceInput(value);
    setUserParametersState(newUserParametersState);
  };

  const handleMessageChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setMessage(e.target.value);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.prevenTableCellefault();

    invoke('call_llm', { id: llm.id, message: message, userParameters: userParametersState}).then((response) => {
      const resp = toLLMResponse((response as any).data)
      console.log(resp);

      // Assuming the history update logic is still the same,
      // add the new call to the stored history.
      const newHistory: HistoryItem = {
        timestamp: new Date(),
        parameters: resp.parameters,
        input: message,
        output: resp.response // I'm not
      };
      const updatedHistory = [...history, newHistory];
      store.set(`${llm.id}-history`, updatedHistory).then(() => {
        setHistory(updatedHistory);
      }).catch((err:any) => {
        setError("failure to save history, are you sure the app is set up correctly?");
      })

    }).catch((err) => {
      console.log(err);
      setError("Failed to retrieve call.");
    });

  };


  return (
    <div className="card live-llm" >
      <LLMInfo llm={llm} rightButton={<Switch defaultChecked checked={checked} onClick={handleToggle}/> }/>
      <Link href={"/history/"+llm.id}>Last Called: {llm.lastCalled? llm.lastCalled.toString() : "Never"}</Link>
    <div className="collapse-wrapper" >
    <div className="collapser" {...getToggleProps()}>{isExpanded ? '▼ Collapse Interface' : '▶ Expand Interface'}</div>
      <div {...getCollapseProps()}>
        <div className="history" style={{overflow: 'auto'}}>
          {history.map((item, index) => (
            <div className="llm-history-item" key={index}>
            {Object.keys(item.parameters).length > 0 ? (
              <TableContainer component={Paper}><Table size="small" className="llm-details-table" aria-label="llm details">
                <TableHead><TableCell>Parameter</TableCell><TableCell>Value</TableCell></TableHead>
                <TableBody>
              {Object.entries(item.parameters).map(([paramName, paramValue], index) => {console.log(item.parameters); return (<TableRow>
                        <TableCell>{paramName}</TableCell>
                        <TableCell>{paramValue}</TableCell>
                        </TableRow>
                       );})
              }
               </TableBody></Table></TableContainer>
              ) : (null)
            }
               <div className="input">{item.input}</div>
               <div className="output">{item.output}</div>
              {/* Add more details as needed */}
            </div>
          ))}
        </div>

        <form onSubmit={handleSubmit}>

        <label><b>Parameters:</b></label>
          {Object.entries(userParametersState).map(([paramName, paramValue], index) => (
            <div key={index}>
              <label>{paramName}
              <input
                type="text"
                value={paramValue}
                onChange={(e) => handleParameterChange(paramName, e.target.value)}
              /></label>
            </div>
          ))}
          <div>
          <label><b>Message</b>
            <textarea placeholder="Enter your LLM prompt here..." value={message} onChange={handleMessageChange} /></label>
          </div>
          <button type="submit">Submit</button>
        </form>
      </div>
      </div>
      <div>
          <div><small>Downloaded: {llm.downloaded}</small></div>
          <div><small>Activated: {llm.activated}</small></div>
      </div>

    </div>
  );
};

export default LLMRunningInfo;

