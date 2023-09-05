# Pantry — An LLM Manager

Pantry is a cross between Homebrew and Docker, for LLMs. It combines an LLM repository, a local LLM runner,
and a remote API, accessed via UI or CLI.

![An AI generated pair of smiling computers.](./pantry_fancy.png)

Out of the box, Pantry includes one click downloads, custom sharapable configurations (with deep links!),
the ability to run multiple LLMs in parallel, a UI, a CLI, and an API.

## Getting Started

Just download one of the builds, download an LLM, turn it on, and go.

You can either use the UI to access the LLM, use `pantry path <llm_id>` to get the LLM's
local path and plug it into your favorite local LLM software, or use the [pantry-rs](https://github.com/JuliaMerz/pantry-rs)
api to integrate the LLMs into your own application. If you're feeling extra fancy
you can use HTTP, though you'll have to use the [docs.rs](https://docs.rs/pantry-rs/latest/pantry_rs/api/struct.PantryAPI.html)
to figure out the API.

## Usage

https://github.com/JuliaMerz/pantry/assets/5734804/9e0d11be-5f8b-4220-b87c-f989fc6982fb

https://github.com/JuliaMerz/pantry/assets/5734804/1d4692c3-5c83-4431-a025-c8f8fd94b244

Currently Pantry is compatible with all LLMs supported by the
[rustformers/llm project](https://github.com/rustformers/llm). That's an ever-expanding
set of LLMs based on the ggml project.


### CLI

You'll need to add the CLI to your path in order to use it. The UI has instructions for doing so,
or you can create an alias to the install location manually. Once you've done so, `pantry --help`
will give you a list of commands.

By default, the CLI uses keychain based authentication to connect to your localhost Pantry instance.
In order to use it an instance of pantry must already be running (you can close the window, it runs in your menubar).

You can set
```
PANTRY_CLI_TARGET
PANTRY_CLI_USER
PANTRY_CLI_KEY
```
to get rid of the keychain request, using the command `pantry new_cli_user`. You can also open the UI for more instructions.

The CLI currently does not allow you to query the LLM, you'll have to use either the UI or a program running [pantry-rs](https://github.com/JuliaMerz/pantry-rs) or making http requests.

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

## Limitations

The system is currently great at running multiple LLMs at once, though obviously performance suffers. It unfortunately doesn't allow you to access
the same LLM in parallel, because the model locks while it's running. This is only likely to be an issue if you're using the UI and the API at
the same time, or if you're running multiple API programs accessing the same LLM at once.

## How You Can Help
### Add Models and Capability Evaluations
I tried to include a decent set of 'known-good' models in the default model repository.
To be honest I've been too busy building this to spend a lot of time testing them,
so if there's a ggml model you like, please pull request it. Help ranking the capabilities
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

## Backlog
- **OpenAI/Other Remote LLM Integration** — The entire architecure is designed to allow this,
and we're not currently taking advantage of it.
- **Non-Text Models**
- **Better parallelism** — currently the model locks during inference, leading to a
potentially ugly queuing situation is a program is running an LLM in the background
while the user is using a different program with an LLM.
- **Expand the CLI** — currently limited to only basic commands.

