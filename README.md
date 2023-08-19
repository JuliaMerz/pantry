# Pantry

Pantry is like homebrew for LLMs. It combines an LLM repository, a local LLM runner,
and a language agnostic integration API. So you can play with or integrate LLMs
without managing the underlying models.

It's also, literally, the easiest way to download and run any GGML compatible LLM.

## Getting Started

Just download one of the builds, download an LLM, turn it on, and go.

## Why?

I built this because I wanted to test different LLMs for a project, and it seemed
harder than it needed to be.

I wanted to build my project to be LLM agnostic, especially for users or organizations
who wanted to insert their own fine-tuned LLMs into the project.

So I built this first.

## Usage

https://github.com/JuliaMerz/pantry/assets/5734804/9e0d11be-5f8b-4220-b87c-f989fc6982fb

Currently Pantry is compatible with all LLMs supported by the
[rustformers/llm project](https://github.com/rustformers/llm). That's an ever-expanding
set of LLMs based on the ggml project.


### APIs
Pantry exposes an API via http-over-socket or localhost, at `/tmp/pantrylocal.sock`
or at port 9404. Some (one) native APIs wrapping those access points also exist.

Running native code is extremely simple, here's an example from the rust API:
``` rust
let perms = UserPermissions {
    perm_superuser: false,
    perm_load_llm: true,
    perm_unload_llm: true,
    perm_download_llm: false,
    perm_session: true,
    perm_request_download: true,
    perm_request_load: true,
    perm_request_unload: true,
    perm_view_llms: true,
};

let pantry = PantryClient::register("project_name".into(), perms).await.unwrap();


// Pause here and use the UI to accept the permission request.

// Use this if you want your code to load an LLM first.
// pantry.load_llm_flex(None, None).await.unwrap();

let sess = pantry.create_session(HashMap::new()).await.unwrap();

let recv = ses.prompt_session("About me: ".into(), HashMap::new()).await.unwrap();
```

- **Web** — Look up the API docs at [docs.rs](https://docs.rs/pantry-rs/latest/pantry_rs/api/struct.PantryAPI.html). Proper API docs coming soon.
- **Rust** — [JuliaMerz/pantry-rs](https://github.com/JuliaMerz/pantry-rs)

## Philosphy and Project Goals (Fancy 'Why?')

Pantry is designed for a future where a large number of LLMs are run locally or on prem. Large
organizations are already banning the usage of ChatGPT due to data control issues,
and as LLMs grow more powerful in _acting_ on data, ownership of data will only
become more valuable.

In the medium term, building your own update flow for integrated LLMs is annoying for
developers, and having a bunch of applications trying to run LLMs independently of
each other on a resource constrained machine is annoying for users.

In the long term organizations will fine-tune their own models to use with pluggable software.

### Project Goals
- Make LLMs more accessible both to enthusiasts and organizations.
- Push LLM computation to the edge, in alignment with the [ggml project](http://ggml.ai/).
- Give developers an easy way to integrate LLMs in a model agnostic way.

## Long Term
Push forward edge-LLMs by providing the tools to better test, measure, and compare them.


## Backlog
- **OpenAI/Other Remote LLM Integration** — The entire architecure is designed to allow this,
and we're not currently taking advantage of it.
- **Headless/Terminal Mode** — Use terminal commands instead of the frontend.
- **Non-Text Models**
- **Better parallelism** — currently the model locks during inference, leading to a
potentially ugly queuing situation is a program is running an LLM in the background
while the user is using a different program with an LLM.

## Todos
Better instrumentation from the UI for when external programs are running things.
Better documentation.
Add system prompt implementation to llmrs.


## How You Can Help
### Add Models and Capability Evaluations
I tried to include a decent set of 'known-good' models in the default model repository.
To be honest I've been too busy building this to spend a lot of time testing them,
so if a model belongs here, please pull request it. Help ranking the capabilities
of existing or new models would also be appreciated.

### Additional APIs
I've built a basic rust API just as a test of function. Improving it, or adding implementations
in other languages, would go a long way.

### Comprehensive Model Evaluation
Pantry should make it relatively simple to build software to more comprehensively evaluate
and compare local LLMs for different use cases. This would be incredibly valuable,
since it would allow the "capabilities" field to be based on more than just "vibes."

### Testing/CI
I'd love to have proper regression testing and automated CI. I just haven't had
the time to do it.

