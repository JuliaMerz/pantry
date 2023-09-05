// src/pages/Home.tsx
import {
  Box,
  Modal,
  CardContent,
  Grid,
  Card,
  Paper,
  Typography,
  Button,
} from '@mui/material';
import {ModalBox} from '../theme';

import React, {useState, useEffect} from 'react';
import LLMRunningInfo from '../components/LLMRunningInfo';
import {invoke} from '@tauri-apps/api/tauri';
import {LLMRunning, keysToCamelUnsafe, toLLMRunning} from '../interfaces';

function Home() {
  const [activeLlms, setActiveLlms] = useState<LLMRunning[]>([]);
  const [execPath, setExecPath] = useState("");
  const [cliId, setCliId] = useState("");
  const [cliKey, setCliKey] = useState("");
  const [cliOpen, setCliOpen] = useState(false);

  const rustGetLLMs = async (): Promise<{data: LLMRunning[]}> => {
    const activeLLMs: {data: LLMRunning[]} = await invoke('active_llms');
    return activeLLMs;
  };
  const fetchLLMs = async () => {
    const ret: {data: LLMRunning[]} = await rustGetLLMs();
    console.log("running llms", ret.data);
    setActiveLlms(ret.data.map(toLLMRunning));
  };

  async function getExecPath() {
    const ret: {data: string} = await invoke('exec_path');
    setExecPath(ret.data);
  }

  async function getCliUser() {
    const ret: {data: any} = await invoke("new_cli_user");
    setCliId(ret.data.id);
    setCliKey(ret.data.api_key);
    setCliOpen(true);
  }


  useEffect(() => {
    fetchLLMs();
    getExecPath();
  }, []);

  function central() {
    if (activeLlms.length > 0) {
      return (
        <Box>
          {
            activeLlms.map((llm) => (
              <LLMRunningInfo key={llm.id} llm={llm} refreshFn={fetchLLMs} />
            ))
          }
        </Box>
      )
    } else {

      return (
        <Box>
          <Paper sx={{
            m: 4,
            p: 2
          }}>
            <Typography sx={{
              mb: 1
            }}>
              Begin by downloading an llm. You can either click "Download LLMs" or use the CLI. Once you've downloaded an LLM, you'll need to enable it, using the "Available LLMs" tab, or use the CLI to get the model path using `pantry path [model]`, then you can use it in whatever existing software you're using for your LLMs.

            </Typography>
            <Typography sx={{
              mb: 1
            }}>
              To use the CLI, add the following to your bashrc file:
            </Typography>
            <Typography sx={{
              fontFamily: "monospace",
              p: 1,
              border: "solid 1px gray",
              borderRadius: 0.5,
              overflow: "hidden",
            }}>
              alias pantry="{execPath}"
            </Typography>
            <Typography>
              The CLI is still new, and doesn't cover the full featureset the UI currently does. If this bothers you, pull requests are appreciated!
            </Typography>

            <Box sx={{
              mt: 4,
              alignItems: "center",
              justifyContent: "center",
              display: "flex",
              flexDirection: "column",

            }}>
              <Typography>To get rid of the keychain notification, you can manually create a superuser API key and add it to your bash RC.</Typography>
              <Button onClick={getCliUser} variant="contained">Generate New API User</Button>
            </Box>
          </Paper>
        </Box>
      );

    }
  }

  return (
    <Box>
      <Typography variant="h2">Currently Running LLMs</Typography>
      {central()}
      <Modal open={cliOpen} onClose={() => setCliOpen(false)}>
        <ModalBox>
          <Card className="cli-form">
            <CardContent>
              <Typography variant="h5">Created CLI User</Typography>
              <Typography>This will only be displayed once, if you lose the key you'll need to generate a new one.</Typography>
              <Typography sx={{
                fontFamily: "monospace",
                p: 1,
                border: "solid 1px gray",
                borderRadius: 0.5,
                overflow: "hidden",
              }}>
                export PANTRY_CLI_USER={cliId};<br />
                export PANTRY_CLI_KEY={cliKey};
              </Typography>
            </CardContent>
          </Card>
        </ModalBox>
      </Modal>
    </Box>
  );
}

export default Home;

