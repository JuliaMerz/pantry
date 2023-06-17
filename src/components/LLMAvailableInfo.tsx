
import Link from '@mui/material/Link';
import LLMInfo from '/LLMInfo';

type LLMAvailableInfo = {
  llm: LLM
};

function LLMAvailableInfo() {


  return (
      <div>
        <LLMInfo key={llm.id} llm={llm}  />
        <Link href={"/history/"+llm.id}>Last Called: {llm.lastCalled ? llm.lastCalled.toString() : "Never"}</Link>
      <div><small>Downloaded: {llm.downloaded}</small></div>
    </div>
    )
}

export default LLMAvailableInfo;
