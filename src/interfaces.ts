// src/interfaces.ts
//
interface LLM  {
  id: string;
  name: string;
  description: string;
};

interface LLMAvailable extends LLM {
  downloaded: string;
  lastCalled: Date;
}
interface LLMActive extends LLMAvailable {
  activated: string;
}

enum LLMRegistryEntrySource{
  GitHub = "github",
  External = "external",

}

enum LLMRegistryEntryConnector {
  Ggml = "ggml",
  OpenAI = "openai"
}

interface LLMRegistry {
  id: string,
  url: string,
}

interface LLMRegistryEntry {
    id: string,
    name: string,
    source: LLMRegistryEntrySource, //maybe enum
    path: string,
    type: string
    connector: LLMRegistryEntryConnector
    create_thread: boolean,
    description: string,
    licence: string,
    parameters: LLMRegistry[],
    user_parameters: string[],
}

enum LLMSource {
  Github = "github",
  External = "external"
}


interface LLMDownloadable extends LLM {
  // Source should basically always be github, unless we develop a... okay fine.
  source: LLMSource,
  path: string,
  type: string,
  license: string,

}

enum LLMRequestType {
  Load = "load",
  Download = "download",
  Unload = "unload"
}

interface LLMRequest extends LLM{
  type: LLMRequestType,
  requester: string,
  [addlInfo: string]: unknown
}

interface LLMDownloadRequest extends LLMRequest {
  type: LLMRequestType.Download,
  source: LLMSource,
  url: string
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
  LLMActive,
  LLMRequest,
  LLMDownloadable,
  LLMDownloadRequest,
  LLMLoadRequest,
  LLMUnloadRequest
}
export {
  LLMRequestType,
  LLMSource,
  keysToCamelUnsafe,
}

function toLLM(rustLLM: any): LLM {
  return {
    id: rustLLM.llm_info.id,
    name: rustLLM.llm_info.name,
    description: rustLLM.llm_info.description,
  };
}

function toLLMAvailable(rustLLMAvailable: any): LLMAvailable {
  return {
    ...toLLM(rustLLMAvailable),
    downloaded: rustLLMAvailable.downloaded,
    lastCalled: new Date(rustLLMAvailable.last_called.time),
  };
}

function toLLMActive(rustLLMRunning: any): LLMActive {
  return {
    ...toLLMAvailable(rustLLMRunning),
    activated: rustLLMRunning.activated,
  };
}

function toLLMRequest(rustLLMRequest: any): LLMRequest {
  const type = rustLLMRequest.type.toLowerCase() as LLMRequestType;

  const baseRequest: LLMRequest = {
    ...toLLM(rustLLMRequest),
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

export { toLLM, toLLMAvailable, toLLMActive, toLLMRequest };

