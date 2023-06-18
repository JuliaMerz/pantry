// src/pages/Requests.tsx

import React, { useEffect, useState } from 'react';
import { LLMRequestInfo, LLMDownloadRequestInfo, LLMUnloadRequestInfo, LLMLoadRequestInfo } from '../components/LLMRequestInfo';
import { LLMRequestType, LLMRequest, LLMDownloadRequest, LLMLoadRequest, LLMUnloadRequest } from '../interfaces';

function Requests() {
  const [requestedLLMs, setRequestedLLMs] = useState<LLMRequest[]>([]);

  useEffect(() => {
    // Replace with actual data fetching
    // const fakeData: LLMRequest[] = [
    //   {
    //     id: '1',
    //     name: 'LLM 1',
    //     description: 'Description 1',
    //     source: LLMSource.Github,
    //     type: LLMRequestType.Download,
    //     requester: 'Requester 1',
    //   },
    //   {
    //     id: '2',
    //     name: 'LLM 2',
    //     description: 'Description 2',
    //     source: LLMSource.URL,
    //     type: LLMRequestType.Download,
    //     requester: 'Requester 2',
    //   },
    //   {
    //     id: 'gpt_4',
    //     name: 'LLM 2',
    //     description: 'Description 2',
    //     type: LLMRequestType.Load,
    //     requester: 'Requester 2',
    //   },
    //   // More LLMs...
    // ];

    // setRequestedLLMs(fakeData);
  }, []);

  return (
    <div>
      <h1>Requested Large Language Models</h1>
      {requestedLLMs.map((llm) => {

        switch (llm.type) {
          case LLMRequestType.Download:
            return <LLMDownloadRequestInfo key={llm.id} {...llm as LLMDownloadRequest} />
          case LLMRequestType.Unload:
            return <LLMUnloadRequestInfo key={llm.id} {...llm as LLMUnloadRequest} />
          case LLMRequestType.Load:
            return <LLMLoadRequestInfo key={llm.id} {...llm as LLMLoadRequest} />
          default:
            return <LLMRequestInfo key={llm.id} {...llm} />
        }

      }
      )}
    </div>
  );
}

export default Requests;

