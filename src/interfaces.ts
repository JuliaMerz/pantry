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

enum LLMSource {
  Github,
  URL
}


interface LLMDownloadable extends LLM {
  // Source should basically always be github, unless we develop a... okay fine.
  source: LLMSource,
  url: string

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
}
