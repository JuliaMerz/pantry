// src/components/LLMInfo.tsx
import React from 'react';
import Link from '@mui/material/Link';
import { useCollapse } from 'react-collapsed';


import { LLM } from '../interfaces';

import Switch from '@mui/material/Switch';
import { invoke } from '@tauri-apps/api/tauri';


type LLMInfoProps = {
  llm: LLM
};

//         <Route path="/history/:id" component={History} />

const LLMInfo: React.FC<LLMInfoProps> = ({
  llm
}) => {
  const [checked, setChecked] = React.useState(false);
  const { getCollapseProps, getToggleProps, isExpanded } = useCollapse();


  const handleToggle = async () => {
    // call function to disable the LLM
    console.log("Enable the LLM");
    if (!checked) {
      const result = await invoke('load_llm', {id: llm.id});
      console.log(result);
    } else {
      const result = await invoke('unload_llm', {id: llm.id});
      console.log(result);
    }
    setChecked(!checked);
  };



  return (
    <div className="llm-info" >
      <div className="split">
        <div className="left">
          <h2>
            {llm.name} <small>{llm.id}</small>
          </h2>
          <div>{llm.description}</div>
        </div>
        <div className="right">
          <Switch defaultChecked checked={checked} onClick={handleToggle}/>
        </div>
      </div>
      <div className="collapse-wrapper" >
      <div className="collapser" {...getToggleProps()}>{isExpanded ? '▼ Details' : '▶ Details'}</div>
      <div {...getCollapseProps()}>
          {isExpanded && (
            <div>
              <div>
                <h5>Parameters</h5>
                {Object.entries(llm.parameters).map(([name, value], i) => (
                  <div key={i}>{name}: {value}</div>
                ))}
              </div>

              <div><b>Connector Type: </b>{llm.connector_type}</div>

              <div>
                <h5>Connector Config</h5>
                {Object.entries(llm.config).map(([name, value], i) => (
                  <div key={i}>{name}: {value}</div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>

  );
};

export default LLMInfo;

