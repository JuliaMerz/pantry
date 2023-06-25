// src/interfaces.ts

interface LLM {
  id: string;
  familyId: string;
  organization: string;

  name: string;
  description: string;
  license: string;
  homepage: string,

  capabilities: {[id: string]: number};
  requirements: string;
  tags: string[];


  url: string;

  config: {[id: string]: string};
  connectorType: string;
  //backend info we maybe don't need
  createThread: boolean;

  parameters: {[id: string]: string};
  userParameters: string[];
};

interface LLMAvailable extends LLM {
  downloaded: string;
  uuid: string;
  lastCalled: Date | null;
}

interface LLMRunning extends LLMAvailable {
  activated: string;
}

interface LLMResponse {
    sessionId: string,
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
  familyId: string;
  organization: string;

  name: string;
  homepage: string;
  description: string;
  license: string;

  capabilities: {[id: string]: number};
  tags: string[],
  requirements: string;

  url: string;
  backendUuid: string;

  connectorType: LLMRegistryEntryConnector;
  createThread: boolean;
  config: {[id: string]: string};

  parameters: {[id: string]: string};
  userParameters: string[];

  downloadState: LLMDownloadState;
}

async function toLLMRegistryEntry(remoteData: any): Promise<LLMRegistryEntry> {
  return {
    ...remoteData,  // this will spread all the existing fields from the remoteData
    backendUuid: "",  // uuid populated later when download starts
    downloadState: LLMDownloadState.NotDownloaded  // initially, it's not downloaded
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


type DownloadEventType =
  | { type: "DownloadProgress"; progress: string }
  | { type: "DownloadCompletion" }
  | { type: "DownloadError"; message: string };


type LLMHistoryItem = {
  id: string;
  callTimestamp: Date;
  updateTimestamp: Date;
  complete: boolean;
  parameters: {[key: string]: string};
  input: string;
  output: string;
};

type LLMSession = {
  id: string //this is a uuid
  started: Date;
  lastCalled: Date;
  name: string; //We don't get this from the server
  llmUuid: string;
  parameters: {[key: string]: string};
  items: LLMHistoryItem[];
};
type LLMEventType =
  | { type: "PromptProgress"; previous: string; next: string }
  | { type: "PromptCompletion"; previous: string }
  | { type: "PromptError"; message: string }
  | { type: "ChannelClose" }
  | { type: "Other" };

interface LLMEventPayload {
  streamId: string; // historyitem.id
  timestamp: Date; //for ordering
  callTimestamp: Date; // historyitem.timestamp
  parameters: {[key: string]: string}; //historyitem.parameters
  input: string,
  llmUuid: string,
  session?: LLMSession
  event: LLMEventType;
}


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
  LLMHistoryItem,
  LLMEventType,
  LLMEventPayload,
  LLMSession,
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
    familyId: rustLLM.family_id,
    organization: rustLLM.organization,
    name: rustLLM.name,
    description: rustLLM.description,
    parameters: rustLLM.parameters,
    userParameters: rustLLM.user_parameters,
    capabilities: rustLLM.capabilities,
    url: rustLLM.url,
    requirements: rustLLM.requirements,
    license: rustLLM.license,
    createThread: rustLLM.create_thread,
    homepage: rustLLM.homepage,
    tags: rustLLM.tags,

    config: rustLLM.config,
    connectorType: rustLLM.connector_type,
  }
}

function toLLMAvailable(rustLLMAvailable: any): LLMAvailable {
  return {
    ...toLLM(rustLLMAvailable.llm_info),
    downloaded: rustLLMAvailable.downloaded,
    uuid: rustLLMAvailable.uuid,
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
  return {
    llm: toLLM(rustLLMResponse.llm_info),
    sessionId: rustLLMResponse.session_id,
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
      url: rustLLMRequest.url,
    };
  }

  return baseRequest;
}

export { toLLM, toLLMAvailable, toLLMRunning, toLLMRequest, toLLMResponse, toLLMRegistryEntry };

