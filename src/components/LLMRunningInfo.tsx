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
  const [userParametersState, setUserParametersState] = useState<{[id: string]: any}>(Object.fromEntries(llm.userParameters.map((val)=>[val, undefined])));
  const [message, setMessage] = useState("");
  const { getCollapseProps, getToggleProps, isExpanded } = useCollapse();
  const [activeSessions, setActiveSessions] = useState<LLMSession[]>([]);
  const [selectedSessionId, setSelectedSessionId] = useState<string>('');
  const [error, setError] = useState("");
  const store = new Store('.local.dat');
  const [sessionMessage, setSessionMessage] = useState("");

  useEffect(() => {
    fetchSessions();
    listenForNewSessions();
  }, []);

  const fetchSessions = async () => {
    console.log("llm.uuid", llm);
    const {data: sessions} = (await invoke('get_sessions', {llmUuid: llm.uuid}) as {data: LLMSession[]});
    console.log("fetched sessions", sessions);
    setActiveSessions(sessions);
  };

  const listenForNewSessions = async () => {
    listen<LLMEventPayload>("llm_response", (event) => {
      console.log("received event: ", event);
      if (event.payload.llmUuid !== llm.uuid)
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
            lastCalled: event.payload.session?.lastCalled || new Date(),
            llmUuid: event.payload.llmUuid,
            parameters: event.payload.session?.parameters || {},
            items: [],
          };
        } else {
          session = {...currentSessions[sessionIndex]};
        }

        // Check if the history item already exists within the session.
        let historyItemIndex = session.items.findIndex((item) => item.id === event.payload.streamId);
        let historyItem: LLMHistoryItem;

        // If the history item does not exist, create a new one.
        if (historyItemIndex === -1) {
          historyItem = {
            id: event.payload.streamId,
            callTimestamp: event.payload.callTimestamp,
            complete: false,
            updateTimestamp: new Date(),
            parameters: event.payload.parameters,
            input: event.payload.input,
            output: '', // As per your model, the output field is empty initially
          };
          session.items.push(historyItem);
        } else {
          // If the history item exists, update it.
          historyItem = session.items[historyItemIndex];
          if(event.payload.callTimestamp > historyItem.updateTimestamp) {
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

    const handleSessionMessageChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setSessionMessage(e.target.value);
  };

  const handleSessionSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    console.log("attempting to call session");

    if (selectedSessionId) {
      await invoke('prompt_session', { llmUuid: llm.uuid, sessionId: selectedSessionId, prompt: sessionMessage})
      .catch((err) => {
        console.error(err);
        setError("Failed to call the session.");
      });
    } else {
      console.error("No session selected.");
      setError("No session selected.");
    }
  };

  const handleNewSessionSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    console.log("attempting to create new session", llm.uuid);

    //call_llm is a shorthand for create_session+prompt_session
    await invoke('call_llm', { llmUuid: llm.uuid, prompt:message, userParameters: userParametersState})
      .then((response) => {
        console.log("call_llm response: ", response);
        //create a new session here
        return toLLMResponse((response as any).data);
      }).catch((err) => {
        console.log(err);
        setError("Failed to create a new session.");
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
          <div>
          <label>Select a session:
            <select value={selectedSessionId} onChange={(e) => setSelectedSessionId(e.target.value)}>
              <option key="new" value=''>New Session</option>
              {activeSessions.sort((a, b) => a.lastCalled.getTime()-b.lastCalled.getTime()).map((session) => (
                <option key={session.id} value={session.id}>{session.name ? `${session.name}` : `${session.id}`}</option>
              ))}
            </select>
          </label>
        </div>

    {selectedSessionId !== '' ? (activeSessions.map((session) => (
      session.id === selectedSessionId && (
        <div key={session.id} className="session-details">
          <h5>{session.name ? `Session: ${session.name}` : `Session ID: ${session.id}`}</h5>
          <h3>Started At: {session.started.toString()}</h3>
          <h3>LLM UUID: {session.llmUuid}</h3>
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
              <h4>Timestamp: {item.callTimestamp.toString()}</h4>
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
            <div>
            <form onSubmit={handleSessionSubmit}>
            <div>
              <label><b>Session Message</b>
                <textarea placeholder="Enter your message for the session here..." value={sessionMessage} onChange={handleSessionMessageChange} /></label>
            </div>
            <button type="submit">Submit</button>
          </form>
        </div>

        </div>
      )))) :
        (
          <div>
            <h5>Create a New Session</h5>
            <form onSubmit={handleNewSessionSubmit}>

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
    )}

        </div>

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

