// src/pages/Settings.tsx

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

function Settings() {
  const [apiKey, setApiKey] = useState('');

  useEffect(() => {
    // Load the API key from the backend on component mount
    invoke('get_settings').then(settings => {
      setApiKey(settings.OPENAI_API_KEY);
    });
  }, []);

  const handleApiKeyChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setApiKey(event.target.value);
    // Update the API key in the backend whenever it changes
    invoke('update_settings', { OPENAI_API_KEY:event.target.value });
  };

  return (
    <div>
      <h1>Settings</h1>
      <div>
        <label>OpenAI API Key: </label>
        <input type="password" value={apiKey} onChange={handleApiKeyChange} />
      </div>
    </div>
  );
}

export default Settings;

