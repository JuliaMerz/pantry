// src/components/LLMRunningInfo.tsx

import {listen} from '@tauri-apps/api/event'
import {CancelOutlined, CheckCircleOutlined} from '@mui/icons-material';

import {
  Box,
  Accordion,
  Divider,
  AccordionSummary,
  AccordionDetails,
  IconButton,
  Typography,
  Card,
  CardContent,
  CircularProgress,
  TableContainer,
  Table,
  TableHead,
  TableRow,
  TableCell,
  TableBody,
  TextField,
  TextareaAutosize,
  Button,
  Link,
  Switch,
  Paper,
  Select,
  MenuItem
} from '@mui/material';

import React, {useState, useEffect} from 'react';
import {LLMRunning, LLMResponse, toLLMResponse, LLMHistoryItem, LLMEventType, LLMEventPayload, LLMSession, toLLMEventPayload, toLLMHistoryItem, toLLMSession} from '../interfaces';
import LLMInfo from './LLMInfo';

import {invoke} from '@tauri-apps/api/tauri';
import {Store} from "tauri-plugin-store-api";
// import { Link } from 'react-router-dom';

// Define new types for history and user parameters


type LLMRunningInfoProps = {
  llm: LLMRunning;
  refreshFn: () => void;
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
  llm,
  refreshFn,
}) => {
  const [checked, setChecked] = useState(true);
  const [userSessionParametersState, setUserSessionParametersState] = useState<{[id: string]: any}>(Object.fromEntries(llm.userSessionParameters.map((val) => [val, undefined])));
  const [userParametersState, setUserParametersState] = useState<{[id: string]: any}>(Object.fromEntries(llm.userParameters.map((val) => [val, undefined])));
  const [message, setMessage] = useState("");
  const [activeSessions, setActiveSessions] = useState<LLMSession[]>([]);
  const [selectedSessionId, setSelectedSessionId] = useState<string>('New Session');
  const [error, setError] = useState("");
  const store = new Store('.local.dat');
  const [sessionMessage, setSessionMessage] = useState("");
  const [cancellationStatus, setCancellationStatus] = useState<{[key: string]: boolean}>({});
  const [cancellationSuccessful, setCancellationSuccessful] = useState<{[key: string]: boolean}>({});


  useEffect(() => {
    fetchSessions();
    return listenForNewSessions();
  }, []);

  const fetchSessions = async () => {
    console.log("llm.uuid", llm);
    const {data: sessions} = (await invoke('get_sessions', {llmUuid: llm.uuid}) as {data: LLMSession[]});
    console.log("fetche sessions", sessions.map(toLLMSession));
    sessions.map((val) => setCancellationStatus(prevStatus => ({...prevStatus, [val.id]: false})));
    sessions.map((val) => setCancellationSuccessful(prevStatus => ({...prevStatus, [val.id]: false})));

    setActiveSessions(sessions.map(toLLMSession));
  };

  const listenForNewSessions = () => {
    const unlisten_promise = listen<LLMEventPayload>("llm_response", (event) => {
      console.log("heard event: ", event);

      //In doing this we skip channel close messages, but we don't subscribe to a singular channel so it's chilld
      if (!event.payload.event.type || event.payload.event.type !== "LLMResponse")
        return
      let payload: LLMEventPayload = toLLMEventPayload(event.payload.event);
      console.log("processed event: ", payload);
      if (payload.llmUuid !== llm.uuid)
        return;


      // IF a session does not exist yet, we need to create the cancellation trackers.
      setCancellationStatus((prevStatus) => {
        let id = payload.session?.id;
        if (id == undefined) {
          return prevStatus;
        }
        if (id in prevStatus) {
          return prevStatus;
        }
        return ({...prevStatus, [id]: false})
      });

      setCancellationSuccessful((prevStatus) => {
        let id = payload.session?.id;
        if (id == undefined) {
          return prevStatus;
        }
        if (id in prevStatus) {
          return prevStatus;
        }
        return ({...prevStatus, [id]: false})
      });

      setActiveSessions((currentSessions: LLMSession[]) => {
        let sessionIndex = currentSessions.findIndex((session) => session.id === payload.session?.id);
        let session: LLMSession;
        let isNewSession = false;


        // If the session does not exist, create a new one.
        if (sessionIndex === -1) {
          isNewSession = true;
          session = {
            id: payload.session?.id || '',
            started: new Date(),
            name: '', // You mentioned that we don't get the name from the server.
            lastCalled: payload.session?.lastCalled || new Date(),
            llmUuid: payload.llmUuid,
            session_parameters: payload.session?.session_parameters || {},
            items: [],
          };
        } else {
          session = {...currentSessions[sessionIndex]};
        }

        // Check if the history item already exists within the session.
        let historyItemIndex = session.items.findIndex((item) => item.id === payload.streamId);
        let historyItem: LLMHistoryItem;

        // If the history item does not exist, create a new one.
        if (historyItemIndex === -1) {
          historyItem = {
            id: payload.streamId,
            callTimestamp: payload.callTimestamp,
            complete: false,
            updateTimestamp: new Date(),
            parameters: payload.parameters,
            input: payload.input,
            output: '', // As per your model, the output field is empty initially
          };
          session.items.push(historyItem);
        } else {
          // TODO: FIGURE OUT WHY INPUT/OUTPUT IS NOT UPDATING.
          // If the history item exists, update it.
          historyItem = session.items[historyItemIndex];
          if (payload.timestamp > historyItem.updateTimestamp) {
            historyItem.updateTimestamp = payload.timestamp
            if (payload.event.type === "PromptProgress") {
              console.log("adding next");
              historyItem.output = payload.event.previous + payload.event.next; // Assuming the output is in the previous field of the event
            }
            if (payload.event.type === "PromptCompletion") {
              historyItem.output = payload.event.previous; // Assuming the output is in the previous field of the event
              console.log("completing");
              historyItem.complete = true;
            }
            session.items[historyItemIndex] = historyItem;
          } else {
            console.log("timestamp skip");
          }
        }

        console.log("setting out to", session.items[session.items.length - 1].output, payload, session);
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
    return () => {
      // https://github.com/tauri-apps/tauri/discussions/5194#discussioncomment-3651818
      unlisten_promise.then(f => f());
    };
  };




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
    const result = await invoke('unload_llm', {uuid: llm.uuid});
    refreshFn();
  };
  const handleSessionParameterChange = (name: string, value: string) => {
    const newUserSessionParametersState = {...userSessionParametersState};
    newUserSessionParametersState[name] = coerceInput(value);
    setUserSessionParametersState(newUserSessionParametersState);
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
    setCancellationStatus(prevStatus => ({...prevStatus, [selectedSessionId]: false}));
    setCancellationSuccessful(prevSuccess => ({...prevSuccess, [selectedSessionId]: false}));


    if (selectedSessionId) {
      await invoke('prompt_session', {llmUuid: llm.uuid, sessionId: selectedSessionId, prompt: sessionMessage, parameters: userParametersState})
        .catch((err) => {
          console.error(err);
          setError("Failed to call the session.");
        });
    } else {
      console.error("No session selected.");
      setError("No session selected.");
    }
  };

  const cancelSession = async (id: string) => {
    console.log("attempting to interrupt");

    setCancellationStatus(prevStatus => ({...prevStatus, [id]: true}));

    invoke('interrupt_session', {llmUuid: llm.uuid, sessionId: id})
      .then((cancelled) => {
        console.log("canclled");
        setCancellationSuccessful(prevSuccess => ({...prevSuccess, [id]: true}));
      })
      .catch((err) => {
        console.error(err);
        setError("Failed to interrupt session.");
      });
  };


  const handleNewSessionSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    console.log("attempting to create new session", llm.uuid);

    //call_llm is a shorthand for create_session+prompt_session
    invoke('call_llm', {llmUuid: llm.uuid, prompt: message, userSessionParameters: userSessionParametersState, userParameters: userParametersState})
      .then((response) => {
        console.log("call_llm response: ", response);
        setSelectedSessionId((response as any).data.session_id); //raw so underscore case
        //create a new session here
        return toLLMResponse((response as any).data);
      }).catch((err) => {
        console.log(err);
        setError("Failed to create a new session.");
      });
  };

  const [expanded, setExpanded] = React.useState<string | false>(false);
  const handleAccordion =
    (panel: string) => (event: React.SyntheticEvent, newExpanded: boolean) => {
      setExpanded(newExpanded ? panel : false);
    };


  return (
    <Card variant="outlined" sx={{boxShadow: 1, p: 2, paddingTop: 0, marginBottom: 2}}>
      <CardContent>
        <LLMInfo llm={llm} rightButton={<Switch checked={checked} onClick={handleToggle} />} />
        <Link href={"/history/" + llm.id}>Last Called: {llm.lastCalled ? llm.lastCalled.toString() : "Never"}</Link>
        <Box>
          <Accordion variant="innerCard" expanded={expanded === 'interface'} onChange={handleAccordion('interface')}>
            <AccordionSummary variant="innerCard" aria-controls="panel1d-content" id="panel1d-header">
              <Typography>Interface</Typography>
            </AccordionSummary>
            <AccordionDetails variant="innerCard">
              <Box sx={{borderBottom: "2 solid black"}}>
                <Select value={selectedSessionId} onChange={(e) => setSelectedSessionId(e.target.value)}>
                  <MenuItem key="new" value='New Session'>New Session</MenuItem>
                  {activeSessions.sort((a, b) => a.lastCalled.getTime() - b.lastCalled.getTime()).map((session) => (
                    <MenuItem key={session.id} value={session.id}>{session.name ? `${session.name}` : `${session.id}`}</MenuItem>
                  ))}
                </Select>
              </Box>

              {selectedSessionId !== 'New Session' ? (activeSessions.findIndex((sess) => {return sess.id == selectedSessionId}) == -1 ? (
                <Box key={setSelectedSessionId}>
                  <CircularProgress />
                </Box>
              ) :
                activeSessions.map((session) => (
                  session.id === selectedSessionId && (
                    <Box key={session.id}>
                      <Typography variant="h4">{session.name ? `Session: ${session.name}` : `Session ID: ${session.id}`}</Typography>
                      <Typography variant="subtitle2">Started At: {session.started.toString()}</Typography>
                      <Typography variant="h5">Session Parameters:</Typography>
                      {Object.keys(session.session_parameters).length > 0 ? (
                        <TableContainer component={Paper}>
                          <Table size="small" aria-label="llm details">
                            <TableHead>
                              <TableRow>
                                <TableCell>Parameter</TableCell>
                                <TableCell>Value</TableCell>
                              </TableRow>
                            </TableHead>
                            <TableBody>
                              {Object.entries(session.session_parameters).map(([paramName, paramValue], index) => (
                                <TableRow key={index}>
                                  <TableCell>{paramName}</TableCell>
                                  <TableCell>{paramValue}</TableCell>
                                </TableRow>
                              ))}
                            </TableBody>
                          </Table>
                        </TableContainer>
                      ) : null
                      }
                      <Typography variant="h5">History:</Typography>
                      {session.items.map((item, index) => (
                        <Box key={index}>
                          <Typography variant="subtitle2">History Item ID: {item.id}</Typography>
                          <Typography variant="subtitle2">Timestamp: {item.callTimestamp.toString()}</Typography>
                          {Object.keys(item.parameters).length > 0 ? (
                            <TableContainer component={Paper}>
                              <Table size="small" aria-label="llm details">
                                <TableHead>
                                  <TableRow>
                                    <TableCell>Parameter</TableCell>
                                    <TableCell>Value</TableCell>
                                  </TableRow>
                                </TableHead>
                                <TableBody>
                                  {Object.entries(item.parameters).map(([paramName, paramValue], index) => (
                                    <TableRow key={index}>
                                      <TableCell>{paramName}</TableCell>
                                      <TableCell>{paramValue}</TableCell>
                                    </TableRow>
                                  ))}
                                </TableBody>
                              </Table>
                            </TableContainer>
                          ) : null
                          }
                          <Typography variant="subtitle2">Input</Typography>
                          <Paper sx={{

                            p: 1,
                          }}>
                            <Typography variant="body1">{item.input}</Typography>
                          </Paper>
                          <Typography variant="subtitle2">Output</Typography>
                          <Paper sx={{

                            p: 1,
                          }}>
                            <Typography>{item.output}</Typography>
                          </Paper>
                          <Divider sx={{

                            margin: 4
                          }} />
                        </Box>
                      ))}
                      {session.items.length == 0 || session.items[session.items.length - 1].complete ? (null) : (
                        <Box sx={{display: 'flex', alignItems: 'center'}}>
                          <Button variant="contained" color="error" onClick={() => cancelSession(session.id)}>
                            {cancellationStatus[session.id] ? (cancellationSuccessful[session.id] ? <CheckCircleOutlined /> : <CircularProgress />) : "Cancel"}
                          </Button>
                          {cancellationSuccessful[session.id] ? (
                            <Typography variant="subtitle2" sx={{color: 'green', marginLeft: 1}}>
                              Cancellation Successful
                            </Typography>
                          ) : null}
                        </Box>)}
                      <Box>
                        <form onSubmit={handleSessionSubmit}>
                          <Typography component="label">Parameters:</Typography>
                          {Object.entries(userParametersState).map(([paramName, paramValue], index) => (
                            <Box key={index}>
                              <TextField
                                label={paramName}
                                onChange={(e) => handleParameterChange(paramName, e.target.value)}
                                variant="outlined"
                              />
                            </Box>
                          ))}
                          <Box>
                            <TextField
                              label="Session Message"
                              multiline
                              value={sessionMessage}
                              onChange={handleSessionMessageChange}
                              variant="outlined"
                            />
                          </Box>
                          <Button type="submit">Submit</Button>
                        </form>
                      </Box>
                    </Box>
                  )))) :
                (
                  <Box>
                    <Typography variant="h5">Create a New Session</Typography>
                    <form onSubmit={handleNewSessionSubmit}>
                      <Typography component="label">Session Parameters:</Typography>
                      {Object.entries(userSessionParametersState).map(([paramName, paramValue], index) => (
                        <Box key={index}>
                          <TextField
                            label={paramName}
                            onChange={(e) => handleSessionParameterChange(paramName, e.target.value)}
                            variant="outlined"
                          />
                        </Box>
                      ))}
                      <Typography component="label">User Parameters:</Typography>
                      {Object.entries(userParametersState).map(([paramName, paramValue], index) => (
                        <Box key={index}>
                          <TextField
                            label={paramName}
                            onChange={(e) => handleParameterChange(paramName, e.target.value)}
                            variant="outlined"
                          />
                        </Box>
                      ))}
                      <Box>
                        <TextField
                          label="Message"
                          multiline
                          minRows={4}
                          minWidth={20}
                          value={message}
                          onChange={handleMessageChange}
                          variant="outlined"
                        />
                      </Box>
                      <Button type="submit">Submit</Button>
                    </form>
                  </Box>
                )}
            </AccordionDetails>
          </Accordion>
        </Box>
        <Box>
          <Typography variant="body2"><small>Downloaded: {llm.downloaded}</small></Typography>
          <Typography variant="body2"><small>Activated: {llm.activated}</small></Typography>
        </Box>
      </CardContent>
    </Card>

  );

}

export default LLMRunningInfo;

