// src/interfaces.ts
import {sanitizeUrl} from "@braintree/sanitize-url";


interface LLMCapabilities {
  assistant: number,
  coding: number,
  writer: number,
}
interface LLM {
  id: string;
  familyId: string;
  organization: string;

  name: string;
  description: string;
  license: string;
  homepage: string,

  capabilities: LLMCapabilities,
  requirements: string;
  tags: string[];


  url: string;

  config: {[id: string]: string};
  connectorType: string;
  //backend info we maybe don't need
  local: boolean;

  parameters: {[id: string]: string};
  userParameters: string[];
  sessionParameters: {[id: string]: string};
  userSessionParameters: string[];
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
  LLMrs = "llmrs",
  OpenAI = "openai"
}

interface LLMRegistry {
  id: string,
  url: string,
  models: {[id: string]: LLMRegistryEntry},
}

interface LLMRegistryRegistry {
  [url: string]: LLMRegistry
}

enum LLMDownloadState {
  NotDownloaded,
  Downloading,
  Downloaded,
}

interface LLMRegistryEntry extends LLM {

  connectorType: LLMRegistryEntryConnector;
  downloadState: LLMDownloadState;
  backendUuid: string;

}
export const produceEmptyRegistryEntry = (): LLMRegistryEntry => {
  return {
    id: '',
    url: '',
    name: '',
    connectorType: LLMRegistryEntryConnector.LLMrs, // provide a default value based on your LLMRegistryEntryConnector enum
    description: 'A generic rustformers/LLM.rs compatible model. Includes most ggml.',
    tags: [],
    familyId: '',
    organization: '',
    homepage: '',
    capabilities: {
      assistant: -1,
      coding: -1,
      writer: -1,
    }, // initialize with default capabilities object
    downloadState: LLMDownloadState.NotDownloaded,
    backendUuid: '',
    local: true,
    requirements: '',
    license: '',
    parameters: {}, // initialize with default LLMRegistry array
    userParameters: ["top_k", "top_p", "repeat_penalty", "temperature", "bias_token", "repetition_penalty_last_n"],
    sessionParameters: {}, // initialize with default LLMRegistry array
    userSessionParameters: [],
    config: {}, // initialize with default config object
  }
}


// Designed for external use
// This attempt at manually cleaning things up
function toLLMRegistryEntryExternal(remoteData: LLMRegistryEntry): LLMRegistryEntry {
  let entry: LLMRegistryEntry = produceEmptyRegistryEntry();
  console.log("Trying to merge", remoteData);

  Object.keys(entry).forEach((raw_key, index) => {
    let key = raw_key as keyof LLMRegistryEntry
    if (remoteData.hasOwnProperty(key)) {
      if (typeof entry[key] === 'number' && typeof remoteData[key] === 'number') {
        (entry[key] as number) = (remoteData[key] as number);
      }

      if (typeof entry[key] === 'string' && typeof remoteData[key] === 'string') {
        // If the key is not 'url', sanitize the value
        if (key !== 'url') {
          (entry[key] as string) = (remoteData[key] as string).replace(/[^\w-. \/]/g, '');
        } else {
          // validate url field
          entry[key] = sanitizeUrl(remoteData[key]);
        }

      } else if (entry[key] instanceof Array && remoteData[key] instanceof Array) {
        (entry[key] as Array<string>) = (remoteData[key] as Array<string>).map((item: string) => item.replace(/[^\w-. \/]/g, ''));

      } else if (typeof entry[key] === 'object' && typeof remoteData[key] === 'object') {
        (entry[key] as object) = {};
        for (let subKey in (remoteData[key] as object)) {
          if (typeof (remoteData[key] as {[key: string]: number})[subKey] === 'number') {
            ((entry[key] as {[key: string]: number})[subKey] as number) = ((remoteData[key] as {
              [key: string]: number
            })[subKey] as number)


          }

          if (typeof (remoteData[key] as {[key: string]: string})[subKey] === 'string') {
            ((entry[key] as {[key: string]: string})[subKey] as string) = ((remoteData[key] as {[key: string]: string})[subKey] as string).replace(/[^\w-. \/]/g, '');
          }
        }
      }


    }

  });

  return {
    ...entry,
    backendUuid: "",  // uuid populated later when download starts
    downloadState: LLMDownloadState.NotDownloaded  // initially, it's not downloaded
  };
}

async function toLLMRegistryEntry(remoteData: any): Promise<LLMRegistryEntry> {
  return {
    ...remoteData,  // this will spread all the existing fields from the remoteData
    backendUuid: "",  // uuid populated later when download starts
    downloadState: LLMDownloadState.NotDownloaded  // initially, it's not downloaded
  };
}

function fromLLMRegistryEntry(frontendEntry: LLMRegistryEntry): any {
  const backendEntry: any = {
    id: frontendEntry.id,
    family_id: frontendEntry.familyId,
    organization: frontendEntry.organization,
    name: frontendEntry.name,
    homepage: frontendEntry.homepage,
    description: frontendEntry.description,
    license: frontendEntry.license,
    capabilities: frontendEntry.capabilities,
    tags: frontendEntry.tags,
    requirements: frontendEntry.requirements,
    url: frontendEntry.url,
    backend_uuid: frontendEntry.backendUuid,
    connector_type: frontendEntry.connectorType,
    local: frontendEntry.local,
    // Some registry entries are written in snake case, because that's our javascript norm
    // Ideally config etc are already written to be rust compatible but here we make sure.
    config: keysToSnakeCaseUnsafe(frontendEntry.config),
    parameters: keysToSnakeCaseUnsafe(frontendEntry.parameters),
    user_parameters: keysToSnakeCaseUnsafe(frontendEntry.userParameters),
    session_parameters: keysToSnakeCaseUnsafe(frontendEntry.sessionParameters),
    user_session_parameters: keysToSnakeCaseUnsafe(frontendEntry.userSessionParameters),
  };
  console.log("backend entry:", backendEntry);

  return backendEntry;
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
const keysToCamelUnsafe = function (o: any) {
  const toCamel = (s: any) => {
    return s.replace(/([-_][a-z])/ig, ($1: any) => {
      return $1.toUpperCase()
        .replace('-', '')
        .replace('_', '');
    });
  };
  const isArray = function (a: any) {
    return Array.isArray(a);
  };
  const isObject = function (o: any) {
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
    return o.map((i: any) => {
      return keysToCamelUnsafe(i);
    });
  }

  return o;
};

// Credits to ChatGPT based on the above function from Matthias
const keysToSnakeCaseUnsafe = function (o: any) {
  const toSnakeCase = function (s: any) {
    return s.replace(/([A-Z])/g, ($1: any) => {
      return '_' + $1.toLowerCase();
    });
  };
  const isArray = function (a: any) {
    return Array.isArray(a);
  };
  const isObject = function (o: any) {
    return o === Object(o) && !isArray(o) && typeof o !== 'function';
  };
  if (isObject(o)) {
    const n = {};

    Object.keys(o).forEach((k) => {
      (n as any)[toSnakeCase(k)] = keysToSnakeCaseUnsafe(o[k]);
    });

    return n;
  } else if (isArray(o)) {
    return o.map((i: any) => {
      return keysToSnakeCaseUnsafe(i);
    });
  }

  return o;
};



type DownloadEventType =
  | {type: "DownloadProgress"; progress: string}
  | {type: "DownloadCompletion"}
  | {type: "DownloadError"; message: string}
  | {type: "ChannelClose"};


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
  session_parameters: {[key: string]: string};
  userId: string;
  items: LLMHistoryItem[];
};

type LLMSessionStub = {
  id: string //this is a uuid
  started: Date;
  lastCalled: Date;
  name: string; //We don't get this from the server
  llmUuid: string;
  session_parameters: {[key: string]: string};
  userId: string;
};


type LLMEventType =
  | {type: "PromptProgress"; previous: string; next: string}
  | {type: "PromptCompletion"; previous: string}
  | {type: "PromptError"; message: string}
  | {type: "ChannelClose"}
  | {type: "Other"};

interface LLMEventPayload {
  streamId: string; // historyitem.id
  timestamp: Date; //for ordering
  callTimestamp: Date; // historyitem.timestamp
  parameters: {[key: string]: string}; //historyitem.parameters
  input: string,
  llmUuid: string,
  session?: LLMSessionStub,
  event: LLMEventType;
}

type PantryEvent =
  DownloadEventType | LLMEventPayload;

interface EmitterEvent {
  event: string,
  id: number,
  payload: {
    event: PantryEvent,
    stream_id: string
  }
}

type DeepLinkEventType =
  | {type: "DownloadEvent", base64: string}
  | {type: "URLError", message: string}
  | {type: "DebugEvent", debug1: string, debug2: string};

interface DeepLinkEvent {
  payload: DeepLinkEventType,
  raw: string,
}



function toLLMSession(rustSession: any, historyItems: any[]): LLMSession {
  return {
    id: rustSession.id,
    started: new Date(rustSession.started),
    lastCalled: new Date(rustSession.last_called),
    name: "",  // Since this isn't provided by the server, set an empty string or some default value.
    llmUuid: rustSession.llm_uuid,
    session_parameters: rustSession.session_parameters,
    userId: rustSession.userId,
    items: historyItems.map((item: any) => toLLMHistoryItem(item))
  }
}

function toLLMSessionStub(rustSession: any): LLMSessionStub {
  return {
    id: rustSession.id,
    started: new Date(rustSession.started),
    lastCalled: new Date(rustSession.last_called),
    name: "",  // Since this isn't provided by the server, set an empty string or some default value.
    llmUuid: rustSession.llm_uuid,
    session_parameters: rustSession.session_parameters,
    userId: rustSession.userId,
  }
}

function toLLMHistoryItem(rustHistoryItem: any): LLMHistoryItem {
  return {
    id: rustHistoryItem.id,
    callTimestamp: new Date(rustHistoryItem.call_timestamp),
    updateTimestamp: new Date(rustHistoryItem.updated_timestamp),
    complete: rustHistoryItem.complete,
    parameters: rustHistoryItem.parameters,
    input: rustHistoryItem.input,
    output: rustHistoryItem.output,
  }
}

function toLLMEventPayload(rustEvent: any): LLMEventPayload {
  if (rustEvent.type !== "LLMResponse") {
    console.error("Potentially invalid type conversion.")
  }
  return {
    streamId: rustEvent.stream_id,
    timestamp: new Date(rustEvent.timestamp),
    callTimestamp: new Date(rustEvent.call_timestamp),
    parameters: rustEvent.parameters,
    input: rustEvent.input,
    llmUuid: rustEvent.llm_uuid,
    session: rustEvent.session ? toLLMSessionStub(rustEvent.session) : undefined,
    event: toLLMEventType(rustEvent.event),
  }
}

function toLLMEventType(rustEventInternal: any): LLMEventType {
  switch (rustEventInternal.type) {
    case "PromptProgress":
      return {type: "PromptProgress", previous: rustEventInternal.previous, next: rustEventInternal.next};
    case "PromptCompletion":
      return {type: "PromptCompletion", previous: rustEventInternal.previous};
    case "PromptError":
      return {type: "PromptError", message: rustEventInternal.message};
    case "Other":
    default:
      return {type: "Other"};
  }
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
  DeepLinkEvent,
  EmitterEvent,
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
    sessionParameters: rustLLM.session_parameters,
    userSessionParameters: rustLLM.user_session_parameters,
    capabilities: rustLLM.capabilities,
    url: rustLLM.url,
    requirements: rustLLM.requirements,
    license: rustLLM.license,
    local: rustLLM.local,
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



export {toLLM, toLLMAvailable, toLLMRunning, toLLMRequest, toLLMResponse, toLLMRegistryEntry, toLLMRegistryEntryExternal, fromLLMRegistryEntry, toLLMSession, toLLMHistoryItem, toLLMEventPayload, };

