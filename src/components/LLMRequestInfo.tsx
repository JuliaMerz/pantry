// src/components/LLMRequestInfo.tsx

import React from 'react';
import {LLMRequest, LLMDownloadRequest, LLMLoadRequest, LLMUnloadRequest} from '../interfaces';

interface LLMRequestInfoProps extends LLMRequest {}
interface LLMDownloadRequestInfoProps extends LLMDownloadRequest {}
interface LLMLoadRequestInfoProps extends LLMLoadRequest {}
interface LLMUnloadRequestInfoProps extends LLMUnloadRequest {}

function LLMRequestInfo(props: LLMRequestInfoProps) {
  const {id, name, description, type, requester} = props;

  return (
    <div className="card request request-generic">
      <h2>{name} <small>({id})</small></h2>
      <p>{description}</p>
      <p>Requested by: {requester}</p>
      <p>Type: {type}</p>
    </div>
  );
}
function LLMDownloadRequestInfo(props: LLMDownloadRequestInfoProps) {
  const {id, name, description, type, requester, source, url} = props;

  return (
    <div className="card request request-download">
      <h2>{name} <small>({id})</small></h2>
      <p>{description}</p>
      <p>{`Downloaded via ${source} at ${url}`}</p>
      <p>Requested by: {requester}</p>
      <p>Type: {type}</p>
    </div>
  );
}

function LLMLoadRequestInfo(props: LLMLoadRequestInfoProps) {
  const {id, name, description, type, requester} = props;

  return (
    <div className="card request request-download">
      <h2>{name} <small>({id})</small></h2>
      <p>{description}</p>
      <p>Requested by: {requester}</p>
      <p>Type: {type}</p>
    </div>
  );
}

function LLMUnloadRequestInfo(props: LLMUnloadRequestInfoProps) {
  const {id, name, description, type, requester} = props;

  return (
    <div className="card request request-unload">
      <h2>{name} <small>({id})</small></h2>
      <p>{description}</p>
      <p>Requested by: {requester}</p>
      <p>Type: {type}</p>
    </div>
  );
}

{/* <p>Source: {source === LLMSource.Github ? 'Github' : 'URL'}</p> */}
export {LLMRequestInfo, LLMDownloadRequestInfo, LLMLoadRequestInfo, LLMUnloadRequestInfo};

