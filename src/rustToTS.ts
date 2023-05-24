import {
  LLM,
  LLMAvailable,
  LLMActive,
  LLMRequest,
  LLMSource,
  LLMRequestType
} from './interfaces';


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

