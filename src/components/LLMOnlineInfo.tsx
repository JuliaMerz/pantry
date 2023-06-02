// src/components/LLMOnlineInfo.tsx

import React from 'react';
import { LLMDownloadable, LLMSource } from '../interfaces';

interface LLMOnlineInfoProps extends LLMDownloadable {}

function LLMOnlineInfo(props: LLMOnlineInfoProps) {
  const { id, name, description, source, path, type, license } = props;

  return (
    <div className="card split available-llm">
      <div className="left">
      <h2>{name} <small>({id})</small></h2>
      <div>{description}</div>
      <div className="flex-row">
        <div><b>Source:</b> {source === LLMSource.Github ? 'GitHub' : 'External'}</div>
        <div><b>License:</b> {license}</div>
      </div>
      </div>
    <div className="right">
        <button>Download</button>
    </div>
    </div>
  );
}

export default LLMOnlineInfo;

