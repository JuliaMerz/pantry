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
  Github = "github",
  URL = "url"
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
