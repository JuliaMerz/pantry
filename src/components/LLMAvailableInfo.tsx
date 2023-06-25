import LLMInfo from '../components/LLMInfo';
import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import Switch from '@mui/material/Switch';
import Link from '@mui/material/Link';
import {LLMAvailable} from '../interfaces';

type LLMAvailableInfoProps = {
  llm: LLMAvailable
}

const LLMAvailableInfo: React.FC<LLMAvailableInfoProps> = ({
  llm
}) => {

  // Use this for enabling the LLM
  const [checked, setChecked] = React.useState(false);
  const handleToggle = async () => {
    // call function to disable the LLM
    console.log("Enable the LLM");
    if (!checked) {
      const result = await invoke('load_llm', {uuid: llm.uuid});
      console.log(result);
    } else {
      const result = await invoke('unload_llm', {uuid: llm.uuid});
      console.log(result);
    }
    setChecked(!checked);
  };
  return (
    <div className="card available-llm">
      <LLMInfo key={llm.id} llm={llm} rightButton={<Switch defaultChecked checked={checked} onClick={handleToggle}/> }/>

      <Link href={"/history/"+llm.id}>Last Called: {llm.lastCalled ? llm.lastCalled.toString() : "Never"}</Link>
    <div><small>Downloaded: {llm.downloaded}</small></div>
    </div>
  )
}

export default LLMAvailableInfo;
