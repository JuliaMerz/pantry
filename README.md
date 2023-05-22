# Pantry

Pantry is a user wrapper on top of LLM Chain, designed as a system service exposing LLMs to other programs.

# Why?

Pantry is designed for a future where LLMs are run primarily locally, on a user's laptop. We're already seeing
major companies forbid their employees from using ChatGPT due to data exfiltration concerns. In such a future,
having many applications on a laptop running cutting edge LLM's is memory-prohibitive. Instead, Pantry
is designed to allow applications to query a local LLM. This enables applciations to add LLM features without
implementing a full LLM stack, something useful from both a development-complexity and resource-efficiency
perspective.

## Brew

## Usage

Services can call pantry either through the local web server, or through a local socket.

When calling Pantry, they can request a certain level of capability, or request a specific
LLM. The specific LLM will be used if available, or, depending on user settings, spun up
if it isn't.

Services can also request a download of a specific model for a specific LLM, depending on user
permissions.

Developers are encouraged to stick to the most general requests possible.


"X would like to download model.bin"

## Configuration

By default, pantry runs a web interface on port 7490. Should this port be occupied, it will increment
by one. Clients interfacing with pantry should, if they find a service on port 7490, attempt the handshake.
Should it fail, increment the port by one and try again, as long as reasonable.



## API Spec

Socket:
**Send:**
```json
{
  "function": "",
  "parameters": {
    "parameter": "value"
  }
}
```
**Return (success):**
```json
{
  "status": "success",
  "value": {
    <return datastructure>
  }
}
```

**Errors (generally)**
```json
{
  "status": "error",
  "message": "<error_string>"
}
```



Web:
Same format (for simplicity) but /call endpoint.

Functions:

### Handshake
```json
{
  "function": "handshake",
  "parameters": {
    "check": <int_value>
  }
}
```

### Call
```json
{
  "function": "call_llm",
  "parameters": {
    "preferred_model": "<model_specifier>",
    "fallback_mode": "<fallback_mode>",
    "model_requirements" ["<model_tag>", "<model_tag>"]
  }
}
```

Fallback Mode:
- "none" (default): Use the requested LLM or return an error.
- "requirements": Use the requirements array to check for a suitable LLM, otherwise error.
- "requirements_load" Use the requirements array to check for a suitable LLM. If one exists but isn't enabled, request it.
- "load": Request the user load the chosen model. Fails if the model isn't downloaded (use download_model to request the user download a new model).

### Load LLM
```json
{
  "function": "load_llm",
  "parameters": {
    "model": "<model_specifier>"
    }
}
```

Depending on user preferences, requests that the user load a particular LLM or loads it automatically.

Note that for remote LLMs, like OpenAI's API, this is a no-op.

### Unload LLM
```json
{
  "function": "unload_llm",
  "parameters": {
    "model": "<model_specifier>"
    }
}
```


### Download Model
```json
{
  "function": "download_model",
  "parameters": {
    "model": "<model_specifier>"
    "url": "<optional: downloads a model from a url>"
  }
}
```
Downloads a specific model from Pantry's model database. Users can enable
URL based downloads, but note that this is disabled by default. URL based
downloads should use a format compatible with [rustformer's LLM library](https://github.com/rustformers/llm).

Note that for remote LLMs, like OpenAI's API, this is a no-op.


## Supported Models

Remote models may require API keys. Tags like `<YEAR>_<ADV/MED/BASIC>` are meant
to convey models that are/were state of the art at a specific point in time.
This is an acknowledgement of current development velocity, and included
for the sake of forwards compatibility.

### OpenAI

Note: credits to [llm-chain](https://github.com/sobelio/llm-chain) for the implementation

| Name | Description | Tags |
| `openai_gpt_4` | OpenAI's GPT 4 | `2023_ADV`, `CONVERSATIONAL` |
| `openai_gpt_3.5` | OpenAI's GPT 3.5 | `2023_MED`, `CONVERSATIONAL` |

llm_chain

### LLaMa

Note: credits to [llm-chain](https://github.com/sobelio/llm-chain) for the implementation

| Name | Description | Tags |

### Alpaca

Note: credits to [llm-chain](https://github.com/sobelio/llm-chain) for the implementation

| Name | Description | Tags |

### llm.rs

Using (rustformer's llm.rs)

| Name | Description | Tags |
GPT-2
GPT-J
LLaMA: LLaMA, Alpaca, Vicuna, Koala, GPT4All v1, GPT4-X, Wizard
GPT-NeoX: GPT-NeoX, StableLM, RedPajama, Dolly v2
BLOOM: BLOOMZ
MPT



### NineLivesAI

Note: I'm putting these up to serve as redirects to useful models, stemming largely
from my needs for the [llm_real_escape_strings](https://github.com/juliamerz/llm_real_escape_strings) project.
| Name | Description | Tags |



Sure, here's a brief description of each of these models. Keep in mind these descriptions are based on my knowledge as of September 2021, and newer versions of these models may have additional features or improvements.

```markdown
## Supported Models

### OpenAI

| Tag | Name | Description | Tags |
| `openai_gpt_4` | OpenAI's GPT 4 | GPT-4 by OpenAI | `2023_ADV`, `CONVERSATIONAL` |
| `openai_gpt_3.5` | OpenAI's GPT 3.5 | GPT-3.5 by OpenAI |`2023_MED`, `CONVERSATIONAL` |

### LLaMa

| Name | Description | Tags |
| `LLaMA` | LLaMA (Low-Level Assembler for a Modern Architecture) is a series of models designed for resource-efficient AI. | |
| `Alpaca` | Alpaca is a variant of LLaMA, designed for efficient large-scale language model training. | |

### Alpaca

| Name | Description | Tags |
| `Alpaca` | Alpaca is a variant of LLaMA, designed for efficient large-scale language model training. | |

### llm.rs

| Name | Description | Tags |
| `GPT-2` | GPT-2 is an earlier version of OpenAI's Generative Pretrained Transformer models. It is capable of generating coherent and contextually relevant sentences by predicting subsequent words in a given piece of text. | |
| `GPT-J` | GPT-J is a large-scale, powerful language model trained by EleutherAI. It is known for its ability to generate high-quality text that is coherent, contextually relevant, and stylistically consistent. | |
| `LLaMA` | LLaMA (Low-Level Assembler for a Modern Architecture) is a series of models designed for resource-efficient AI. | |
| `Alpaca` | Alpaca is a variant of LLaMA, designed for efficient large-scale language model training. | |
| `Vicuna` | Vicuna is another variant of LLaMA, designed for even more resource-efficient training compared to Alpaca. | |
| `Koala` | Koala is a variant of LLaMA, designed for high-quality text generation with resource efficiency in mind. | |
| `GPT4All v1` | GPT4All v1 is a variant of LLaMA that aims to provide the capabilities of GPT-4 to a wider range of devices and applications. | |
| `GPT4-X` | GPT4-X is an extended version of GPT4All, designed to offer even higher-quality text generation. | |
| `Wizard` | Wizard is a variant of LLaMA, designed for high-quality, interactive conversational AI. | |
| `GPT-NeoX` | GPT-NeoX is a large-scale model designed for high-quality text generation. | |
| `StableLM` | StableLM is a variant of LLaMA, designed for stable and reliable text generation across a wide range of contexts and applications. | |
| `RedPajama` | RedPajama is a variant of LLaMA, designed for high-quality text generation in a wide range of styles and tones. | |
| `Dolly v2` | Dolly v2 is a variant of LLaMA, designed for high-quality, creative text and image generation. | |
| `BLOOM` | BLOOM is a series of models designed for high-quality text generation, with a focus on creativity and versatility. | |
| `BLOOMZ` | BLOOMZ is a variant of BLOOM, designed for

Based on the models you've asked about and the research I've conducted, here's the information I've found:

### LLaMa

| Name | Description | Tags |
| ---- | ----------- | ---- |
| `LLaMa` | A large language model developed by OpenAI, trained on a dataset of 1.2 trillion tokens【12†source】. | `2022_ADV`, `CONVERSATIONAL` |

### Alpaca

| Name | Description | Tags |
| ---- | ----------- | ---- |
| `Alpaca` | A language model developed by Stanford, designed to be better at following instructions and supporting interactive conversations【8†source】. | `2022_ADV`, `CONVERSATIONAL` |

### llm.rs

| Name | Description | Tags |
| ---- | ----------- | ---- |
| `GPT-2` | OpenAI's second generation transformer model, smaller and more manageable than its successors | `2018_ADV`, `CONVERSATIONAL` |
| `GPT-J` | A large language model created by EleutherAI, designed to be a free alternative to GPT-3, trained on a diverse range of internet text | `2021_ADV`, `CONVERSATIONAL` |
| `LLaMA` | See LLaMa section above |
| `Alpaca` | See Alpaca section above |
| `Vicuna` | A fine-tuned version of LLaMa developed by StabilityAI's CarperAI team【36†source】 | `2022_ADV`, `CONVERSATIONAL` |
| `Koala` | Dialogue based model trained by academic research labs. | |
| `GPT4All v1` | I couldn't find detailed information about GPT4All v1 within the time constraints |
| `GPT4-X` | I couldn't find detailed information about GPT4-X within the time constraints |
| `Wizard` | I wasn't able to find a detailed description of the Wizard model in the time provided |

### GPT-NeoX

| Name | Description | Tags |
| ---- | ----------- | ---- |
| `GPT-NeoX` | A 20 billion parameter autoregressive language model trained on the Pile by EleutherAI【30†source】. | `2022_ADV`, `CONVERSATIONAL` |
| `StableLM` | A language model developed by StabilityAI's CarperAI team, trained on a new dataset built on The Pile【37†source】. | `2022_ADV`, `CONVERSATIONAL` |
| `RedPajama` | A project aiming to create a set of leading, fully open-source models, starting with the LLaMa training dataset of over 1.2 trillion tokens【43†source】【44†source】【45†source】【46†source】. | `2023_ADV`, `CONVERSATIONAL` |
| `Dolly v2` | I couldn't find detailed information about Dolly v2 within the time constraints |

### BLOOM

| Name | Description | Tags |
| ---- | ----------- | ---- |
| `BLOOMZ` | BigScience Large Open-science Open-access Multilingual Language Model, a transformer-based large language model created by over 1000 AI researchers, trained on around 366 billion tokens【20†source】. | `2022_ADV`, `CONVERSATIONAL`, `MULTILINGUAL` |

### MPT

| Name | Description | Tags |
| ---- | ----------- | ---- |
| `MPT-7B` | A large language model standard developed by MosaicML, for open-source, commercially usable LLMs, trained on a diverse


