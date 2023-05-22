// src/components/LLMInfo.tsx
import React from 'react';
import Link from '@mui/material/Link';

import Switch from '@mui/material/Switch';


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
  const handleToggle = () => {
    // call function to disable the LLM
    console.log("Enable the LLM");
    setChecked(!checked);
  };



  return (
    <div className="card live-llm" >
      <div>
        <h2>
          {name} <small>{id}</small>
        </h2>
        <Link href={"/history/"+id}>Last Called: {lastCalled.toString()}</Link>
        <p>{description}</p>
        <p><small>Downloaded: {downloaded}</small></p>
      </div>
      <div>
        <Switch defaultChecked checked={checked} onClick={handleToggle}/>
      </div>
    </div>
  );
};

export default LLMInfo;

