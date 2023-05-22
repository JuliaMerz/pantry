// src/components/LLMLiveInfo.tsx
import React from 'react';
import Link from '@mui/material/Link';

import Switch from '@mui/material/Switch';


type LLMLiveInfoProps = {
  id: string;
  name: string;
  description: string;
  lastCalled: Date;
  downloaded: string;
  activated: string;
};

//         <Route path="/history/:id" component={History} />

const LLMLiveInfo: React.FC<LLMLiveInfoProps> = ({
  id,
  name,
  description,
  lastCalled,
  downloaded,
  activated
}) => {
  const [checked, setChecked] = React.useState(true);
  const handleToggle = () => {
    // call function to disable the LLM
    console.log("Disable the LLM");
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
        <p><small>Activated: {activated}</small></p>
      </div>
      <div>
        <Switch defaultChecked checked={checked} onClick={handleToggle}/>
      </div>
    </div>
  );
};

export default LLMLiveInfo;

