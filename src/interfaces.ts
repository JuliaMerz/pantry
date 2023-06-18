// src/interfaces.ts

interface LLM  {
  id: string;
  family_id: string;
  organization: string;

  name: string;
  description: string;
  requirements: string;
  licence: string;
  user_parameters: string[];

  capabilities: {[id: string]: number};

  parameters: {[id: string]: string};
  config: {[id: string]: string};
  connector_type: string;

  url: string;

  //backend info we maybe don't need
  create_thread: boolean;
};

interface LLMAvailable extends LLM {
  downloaded: string;
  lastCalled: Date | null;
}
interface LLMRunning extends LLMAvailable {
  activated: string;
}

interface LLMResponse {
    response: string, //Actual text response
    parameters: {[id: string]: string};
    llm: LLM, //LLM used
}



enum LLMRegistryEntryConnector {
  Ggml = "ggml",
  LLMrs = "llmrs",
  OpenAI = "openai"
}

interface LLMRegistry {
  id: string,
  url: string,
  models: LLMRegistryEntry[],
}

interface LLMRegistryRegistry {
  [url: string]: LLMRegistry
}

enum LLMDownloadState {
  NotDownloaded,
  Downloading,
  Downloaded,
}

interface LLMRegistryEntry {
  id: string;
  family_id: string;
  organization: string;
  name: string;
  homepage: string;
  download_state: LLMDownloadState;
  backend_uuid: string;
  connector_type: LLMRegistryEntryConnector;
  create_thread: boolean;
  description: string;
  licence: string;
  parameters: {[id: string]: string};
  user_parameters: string[];
  capabilities: {[id: string]: number};
  tags: string[],
  url: string;
  config: {[id: string]: string};
  requirements: string;
}

async function toLLMRegistryEntry(remoteData: any): Promise<LLMRegistryEntry> {
  return {
    ...remoteData,  // this will spread all the existing fields from the remoteData
    backend_uuid: "",  // uuid populated later when download starts
    download_state: LLMDownloadState.NotDownloaded  // initially, it's not downloaded
  };
}

enum LLMRequestType {
  Load = "load",
  Download = "download",
  Unload = "unload"
}

interface LLMRequest extends LLM {
  type: LLMRequestType,
  requester: string, // This is a uuid
  [addlInfo: string]: unknown
}

interface LLMDownloadRequest extends LLMRequest {
  type: LLMRequestType.Download,
  entry: LLMRegistryEntry,
}

interface LLMLoadRequest extends LLMRequest {
  type: LLMRequestType.Load,
}

interface LLMUnloadRequest extends LLMRequest {
  type: LLMRequestType.Unload,
}

/*
Frankly horrible that we need this, but we do.

Typing: We're taking in poorly typed things from
tauri, so any actually makes sense to use here.

Credit to https://matthiashager.com/converting-snake-case-to-camel-case-object-keys-with-javascript
*/
const keysToCamelUnsafe = function (o:any) {
  const toCamel = (s:any) => {
    return s.replace(/([-_][a-z])/ig, ($1:any) => {
      return $1.toUpperCase()
        .replace('-', '')
        .replace('_', '');
    });
  };
  const isArray = function (a:any) {
    return Array.isArray(a);
  };
  const isObject = function (o:any) {
    return o === Object(o) && !isArray(o) && typeof o !== 'function';
  };
  if (isObject(o)) {
    const n = {};

    Object.keys(o)
      .forEach((k) => {
        (n as any)[toCamel(k)] = keysToCamelUnsafe(o[k]);
      });

    return n;
  } else if (isArray(o)) {
    return o.map((i:any) => {
      return keysToCamelUnsafe(i);
    });
  }

  return o;
};


export type {
  LLM,
  LLMAvailable,
  LLMRunning,
  LLMResponse,
  LLMRequest,
  LLMDownloadRequest,
  LLMLoadRequest,
  LLMUnloadRequest,
  LLMRegistry,
  LLMRegistryRegistry,
  LLMRegistryEntry,
}
export {
  LLMRequestType,
  keysToCamelUnsafe,
  LLMDownloadState,
  LLMRegistryEntryConnector,
}

function toLLM(rustLLM: any): LLM {
  return {
    id: rustLLM.id,
    family_id: rustLLM.family_id,
    organization: rustLLM.organization,
    name: rustLLM.name,
    description: rustLLM.description,
    parameters: rustLLM.parameters,
    user_parameters: rustLLM.user_parameters,
    capabilities: rustLLM.capabilities,

    config: rustLLM.config,
    connector_type: rustLLM.connector_type,
  }
}

function toLLMAvailable(rustLLMAvailable: any): LLMAvailable {
  return {
    ...toLLM(rustLLMAvailable.llm_info),
    downloaded: rustLLMAvailable.downloaded,
    lastCalled: rustLLMAvailable.lastCalled ? new Date(rustLLMAvailable.last_called.time) : null,
  };
}

function toLLMRunning(rustLLMRunning: any): LLMRunning {
  return {
    ...toLLMAvailable(rustLLMRunning),
    activated: rustLLMRunning.activated,
  };
}

function toLLMResponse(rustLLMResponse: any): LLMResponse {
  console.log(rustLLMResponse.parameters);
  return {
    llm: toLLM(rustLLMResponse.llm_info),
    response: rustLLMResponse.response,
    parameters: rustLLMResponse.parameters
  }

}

function toLLMRequest(rustLLMRequest: any): LLMRequest {
  const type = rustLLMRequest.type.toLowerCase() as LLMRequestType;

  const baseRequest: LLMRequest = {
    ...toLLM(rustLLMRequest.llm_info),
    type: type,
    requester: rustLLMRequest.requester,
  };

  if (type === LLMRequestType.Download) {
    return {
      ...(baseRequest as any),
      source: rustLLMRequest.source.toLowerCase() as LLMSource,
      url: rustLLMRequest.url,
    };
  }

  return baseRequest;
}

export { toLLM, toLLMAvailable, toLLMRunning, toLLMRequest, toLLMResponse, toLLMRegistryEntry };

