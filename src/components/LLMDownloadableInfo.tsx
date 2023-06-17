// src/components/LLMDownloadableInfo.tsx

import React from 'react';
import { LLMRegistry, LLMRegistryEntry } from '../interfaces';

interface LLMDownloadableInfoProps {
  llm: LLMRegistryEntry,
  registry: LLMRegistry,
}

const LLMDownloadableInfo: React.FC<LLMDownloadableInfoProps> = ({ llm }) => {
  return (
    <div className="card split available-llm">
      <div className="left">
        <h2>{llm.name} <small>({llm.id})</small></h2>
        <div>{llm.description}</div>
        <div><b>URL:</b> {llm.url}</div>
        <div><b>Type:</b> {llm.type}</div>
        <div><b>Connector:</b> {llm.connector}</div>
        <div><b>Create Thread:</b> {llm.create_thread}</div>
        <div><b>Requirements:</b> {llm.requirements}</div>
        <div className="flex-row">
          <div><b>License:</b> {llm.licence}</div>
          <div><b>Capabilities:</b> {JSON.stringify(llm.capabilities)}</div>
          <div><b>Parameters:</b> {JSON.stringify(llm.parameters)}</div>
          <div><b>User Parameters:</b> {llm.user_parameters.join(", ")}</div>
          <div><b>Config:</b> {JSON.stringify(llm.config)}</div>
        </div>
      </div>
      <div className="right">
        <button>Download</button>
      </div>
    </div>
  );
};

export default LLMDownloadableInfo;

