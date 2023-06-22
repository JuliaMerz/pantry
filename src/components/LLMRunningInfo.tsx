// src/components/LLMRunningInfo.tsx
import Link from '@mui/material/Link';

import { listen } from '@tauri-apps/api/event'
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
import { LLMRunning, LLMResponse, toLLMResponse, LLMHistoryItem, LLMEventType, LLMEventPayload, LLMSession } from '../interfaces';
import LLMInfo from './LLMInfo';

import { invoke } from '@tauri-apps/api/tauri';
import { Store } from "tauri-plugin-store-api";
// import { Link } from 'react-router-dom';

// Define new types for history and user parameters


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
  const [checked, setChecked] = useState(true);
  const [userParametersState, setUserParametersState] = useState<{[id: string]: any}>(Object.fromEntries(llm.user_parameters.map((val)=>[val, undefined])));
  const [message, setMessage] = useState("");
  const { getCollapseProps, getToggleProps, isExpanded } = useCollapse();
  const [activeSessions, setActiveSessions] = useState<LLMSession[]>([]);
  const [selectedSessionId, setSelectedSessionId] = useState<string>('');
  const [error, setError] = useState("");
  const store = new Store('.local.dat');

  useEffect(() => {
  fetchSessions();
  listenForNewSessions();
}, []);

  const fetchSessions = async () => {
    const sessions = await invoke('get_sessions', {client_id: "local", llm_id: llm.uuid});
    setActiveSessions(sessions);
  };

  const listenForNewSessions = async () => {
    listen<LLMEventPayload>("llm_response", (event) => {
      if (event.payload.llm_uuid !== llm.uuid)
        return;

      setActiveSessions((currentSessions: LLMSession[]) => {
        let sessionIndex = currentSessions.findIndex((session) => session.id === event.payload.session?.id);
        let session: LLMSession;
        let isNewSession = false;

        // If the session does not exist, create a new one.
        if (sessionIndex === -1) {
          isNewSession = true;
          session = {
            id: event.payload.session?.id || '',
            started: new Date(),
            name: '', // You mentioned that we don't get the name from the server.
            llm_uuid: event.payload.llm_uuid,
            parameters: event.payload.session?.parameters || {},
            items: [],
          };
        } else {
          session = {...currentSessions[sessionIndex]};
        }

        // Check if the history item already exists within the session.
        let historyItemIndex = session.items.findIndex((item) => item.id === event.payload.stream_id);
        let historyItem: LLMHistoryItem;

        // If the history item does not exist, create a new one.
        if (historyItemIndex === -1) {
          historyItem = {
            id: event.payload.stream_id,
            timestamp: event.payload.call_timestamp,
            complete: false,
            last_call_timestamp: new Date(),
            parameters: event.payload.parameters,
            input: event.payload.input,
            output: '', // As per your model, the output field is empty initially
          };
          session.items.push(historyItem);
        } else {
          // If the history item exists, update it.
          historyItem = session.items[historyItemIndex];
          if(event.payload.call_timestamp > historyItem.last_call_timestamp) {
            if (event.payload.event.type === "PromptProgress") {
              historyItem.output = event.payload.event.previous+event.payload.event.next; // Assuming the output is in the previous field of the event
            }
            if (event.payload.event.type === "PromptCompletion") {
              historyItem.output = event.payload.event.previous; // Assuming the output is in the previous field of the event
              historyItem.complete = true;
            }
            session.items[historyItemIndex] = historyItem;
          }
        }

        if (isNewSession) {
          return [...currentSessions, session];
        } else {
          return [
            ...currentSessions.slice(0, sessionIndex),
            session,
            ...currentSessions.slice(sessionIndex + 1)
          ];
        }
      });
    });
  };



  useEffect(() => {
    if (isExpanded) {
      // fetchHistory();
    }
  }, [isExpanded]);

  // const fetchHistory = async () => {
  //   store.get(`${llm.id}-history`).then((hist) => {
  //     const storedHistory: HistoryItem[] = hist ? hist as HistoryItem[] : [];
  //     setHistory(storedHistory || []);
  //   }).catch((err:any) => {
  //     setHistory([]);
  //   })
  // };

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
  e.preventDefault();

  const respPromise = invoke('call_llm', { id: llm.id, message: message, userParameters: userParametersState})
    .then((response) => {
      return toLLMResponse((response as any).data);
    }).catch((err) => {
      console.log(err);
      setError("Failed to retrieve call.");
    });

  let completeOutput = "";
  // const unlistenPromise = listen<LLMEventPayload>("llm_response", (event) => {
  //   console.log("received event llm_response", event);
  //   if (resp === undefined)
  //     return
  //   if (event.payload.stream_id !== resp.session_id)
  //     return;

  //   switch (event.payload.event.type) {
  //     case "PromptProgress":
  //       completeOutput += event.payload.event.next;
  //       break;
  //     case "PromptCompletion":
  //       completeOutput += event.payload.event.previous;
  //       break;
  //     case "ChannelClose":
  //       unlisten();
  //       break;
  //     default:
  //       console.error("Unexpected event type: " + event.payload.event.type);
  //   }
  // });

  // Wait for both the initial response and the listener registration
  // const [resp, unlisten] = await Promise.all([respPromise, unlistenPromise]);

  // if(resp) {
    // TODO: Add this back later, but right now we don't need it?
    //
    // Assuming the history update logic is still the same,
    // add the new call to the stored history.
    // const newHistory: LLMHistoryItem = {
    //   id: resp.session_id,
    //   timestamp: new Date(),
    //   parameters: resp.parameters,
    //   input: message,
    //   output: completeOutput
    // };
    // const updatedHistory = [...history, newHistory];
    // store.set(`${llm.id}-history`, updatedHistory).then(() => {
    //   setHistory(updatedHistory);
    // }).catch((err:any) => {
    //   setError("failure to save history, are you sure the app is set up correctly?");
    // })
  // }

  };


  return (
    <div className="card live-llm" >
      <LLMInfo llm={llm} rightButton={<Switch defaultChecked checked={checked} onClick={handleToggle}/> }/>
      <Link href={"/history/"+llm.id}>Last Called: {llm.lastCalled? llm.lastCalled.toString() : "Never"}</Link>
    <div className="collapse-wrapper" >
    <div className="collapser" {...getToggleProps()}>{isExpanded ? '▼ Collapse Interface' : '▶ Expand Interface'}</div>
      <div {...getCollapseProps()}>
        <div className="history" style={{overflow: 'auto'}}>
          <div>
  <label>Select a session:
    <select value={selectedSessionId} onChange={(e) => setSelectedSessionId(e.target.value)}>
      {activeSessions.map((session) => (
        <option value={session.id}>{session.name}</option>
      ))}
    </select>
  </label>
</div>
{activeSessions.map((session) => (
  session.id === selectedSessionId && (
    <div className="session-details">
      <h2>Session ID: {session.id}</h2>
      <h3>Started At: {session.started.toString()}</h3>
      <h3>LLM UUID: {session.llm_uuid}</h3>
      <h3>Session Parameters:</h3>
      {Object.keys(session.parameters).length > 0 ? (
        <TableContainer component={Paper}><Table size="small" className="llm-details-table" aria-label="llm details">
          <TableHead><TableCell>Parameter</TableCell><TableCell>Value</TableCell></TableHead>
          <TableBody>
        {Object.entries(session.parameters).map(([paramName, paramValue], index) => (
          <TableRow key={index}>
            <TableCell>{paramName}</TableCell>
            <TableCell>{paramValue}</TableCell>
          </TableRow>
        ))}
         </TableBody></Table></TableContainer>
        ) : (null)
      }
      <h3>History Items:</h3>
      {session.items.map((item, index) => (
        <div className="llm-history-item" key={index}>
          <h4>History Item ID: {item.id}</h4>
          <h4>Timestamp: {item.timestamp.toString()}</h4>
          <h3>Parameters:</h3>
          {Object.keys(item.parameters).length > 0 ? (
            <TableContainer component={Paper}><Table size="small" className="llm-details-table" aria-label="llm details">
              <TableHead><TableCell>Parameter</TableCell><TableCell>Value</TableCell></TableHead>
              <TableBody>
            {Object.entries(item.parameters).map(([paramName, paramValue], index) => (
              <TableRow key={index}>
                <TableCell>{paramName}</TableCell>
                <TableCell>{paramValue}</TableCell>
              </TableRow>
            ))}
             </TableBody></Table></TableContainer>
            ) : (null)
          }
          <div className="input">{item.input}</div>
          <div className="output">{item.output}</div>
        </div>
      ))}
    </div>
  )
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

