// src/components/LLMOnlineInfo.tsx

import React from 'react';
import { LLMDownloadable, LLMSource } from '../interfaces';

interface LLMOnlineInfoProps extends LLMDownloadable {}

function LLMOnlineInfo(props: LLMOnlineInfoProps) {
  const { id, name, description, source } = props;

  return (
    <div>
      <h2>{name} <small>({id})</small></h2>
      <p>{description}</p>
      <p>Source: {source === LLMSource.Github ? 'Github' : 'URL'}</p>
    </div>
  );
}

export default LLMOnlineInfo;

