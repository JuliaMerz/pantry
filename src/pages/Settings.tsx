// src/pages/Settings.tsx
//
import {useState, useEffect} from 'react';
import {TextField, FormControlLabel, IconButton, Switch, CircularProgress, InputAdornment, Box, Typography, Stack} from '@mui/material';
import {invoke} from '@tauri-apps/api/tauri';
import SaveIcon from '@mui/icons-material/Save';


type UserSettings = {
  openai_key: string;
  use_gpu: boolean;
  n_thread: number;
  n_batch: number;
};

type LoadingState = {
  openai_key: boolean;
  use_gpu: boolean;
  n_thread: boolean;
  n_batch: boolean;
};

function Settings() {
  const [openaiKey, setOpenaiKey] = useState('');
  const [useGpu, setUseGpu] = useState(false);
  const [nThread, setNThread] = useState(1);
  const [nBatch, setNBatch] = useState(1);
  const [loading, setLoading] = useState(false);
  const [preferredActive, setPreferredActive] = useState(3);
  const [dedupDownloads, setDedupDownloads] = useState(true);

  useEffect(() => {
    invoke('get_user_settings').then((settings: any) => {
      setUseGpu(settings.use_gpu);
      setNThread(settings.n_thread);
      setNBatch(settings.n_batch);
      setPreferredActive(settings.preferred_active_sessions);
      setDedupDownloads(settings.dedup_downloads);
    });
  }, []);

  const handleSave = () => {
    setLoading(true);
    Promise.all([
      openaiKey ? invoke('set_user_setting', {key: 'openai_key', value: openaiKey}) : Promise.resolve(),
      invoke('set_user_setting', {key: 'use_gpu', value: useGpu}),
      invoke('set_user_setting', {key: 'n_thread', value: nThread}),
      invoke('set_user_setting', {key: 'n_batch', value: nBatch}),
      invoke('set_user_setting', {key: 'preferred_active_sessions', value: preferredActive}),
      invoke('set_user_setting', {key: 'dedup_downloads', value: dedupDownloads}),
    ])
      .then(() => invoke('get_user_settings'))
      .then((settings: any) => {
        setOpenaiKey('');
        setUseGpu(settings.use_gpu);
        setNThread(settings.n_thread);
        setNBatch(settings.n_batch);
        setPreferredActive(settings.preferred_active_sessions);
        setDedupDownloads(settings.dedup_downloads);
        setLoading(false);
      })
      .catch((err) => {
        console.error(err);
        setLoading(false);
      });
  };

  return (
    <Box>
      <Typography variant="h2">Settings</Typography>
      <Typography variant="body1">Note: LLMs need to be reloaded to apply new settings.</Typography>

      <Stack>
        <TextField
          label="OpenAI API Key"
          type="password"
          value={openaiKey}
          onChange={(e) => setOpenaiKey(e.target.value)}
        />
        <FormControlLabel
          control={<Switch checked={useGpu} onChange={(e) => setUseGpu(e.target.checked)} />}
          label="Use GPU"
        />
        <TextField
          label="Number of Threads"
          type="number"
          value={nThread}
          onChange={(e) => setNThread(parseInt(e.target.value))}
        />
        <TextField
          label="Batch Size"
          type="number"
          value={nBatch}
          onChange={(e) => setNBatch(parseInt(e.target.value))}
        />
        <TextField
          label="Preferred Active Sessions (extras will be saved to disk)."
          type="number"
          value={preferredActive}
          onChange={(e) => setPreferredActive(parseInt(e.target.value))}
        />
        <FormControlLabel
          control={<Switch checked={dedupDownloads} onChange={(e) => setDedupDownloads(e.target.checked)} />}
          label="Dedup Downloads (if a new LLM downlaods from the same URL as an existing LLM, will skip download and use the same model file)"
        />
        <IconButton onClick={handleSave} disabled={loading}>
          {loading ? <CircularProgress size={24} /> : <SaveIcon />}
        </IconButton>
      </Stack>
    </Box>
  );
}

export default Settings;


