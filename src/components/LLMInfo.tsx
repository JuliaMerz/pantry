// src/components/LLMInfo.tsx
import React from 'react';
import Link from '@mui/material/Link';

import Switch from '@mui/material/Switch';
import { invoke } from '@tauri-apps/api/tauri';


type LLMInfoProps = {
  id: string;
  name: string;
  description: string;
  lastCalled: Date;
  downloaded: string;
};

//         <Route path="/history/:id" component={History} />

const LLMInfo: React.FC<LLMInfoProps> = ({
  id,
  name,
  description,
  lastCalled,
  downloaded,
}) => {
  const [checked, setChecked] = React.useState(false);
  const handleToggle = async () => {
    // call function to disable the LLM
    console.log("Enable the LLM");
    const result = await invoke('load_llm', {id: id});
    console.log(result);
    setChecked(!checked);
  };



  return (
    <div className="card split live-llm" >
      <div className="left">
        <h2>
          {name} <small>{id}</small>
        </h2>
        <Link href={"/history/"+id}>Last Called: {lastCalled.toString()}</Link>
        <div>{description}</div>
        <div><small>Downloaded: {downloaded}</small></div>
      </div>
      <div className="right">
        <Switch defaultChecked checked={checked} onClick={handleToggle}/>
      </div>
    </div>
  );
};

export default LLMInfo;

