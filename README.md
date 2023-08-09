# Pantry

Pantry is a local LLM runner, designed to easily download, test, and integrate local LLMs.

## Getting Started

Just download one of the builds, download an LLM, turn it on, and go.

# Why?

As a an organization, you'll eventually want to fine tune an LLM model on your organization's data, then have all of your LLM enabled software using it.

As a develpoment team building more specialized software, you'll want an easy way to deploy updated models to your developer's computers.

As an enthusiast, you already want an easy way to download and try new LLMs.

# Philosphy (Fancy 'Why?')

Pantry is designed for a future where LLMs are run primarily locally. Large
organizations are already banning the usage of ChatGPT due to data control issues,
and as LLMs grow more powerful in _acting_ on data, ownership of data will only
become more valuable.

In the medium term, running every local application with its own LLM is unrealistic
due to resource constraints and worse, annoying for developers. In the long term
organizations will fine-tune their own models to use with pluggable software.

Also I wanted to run and LLM locally and it looked annoying and I thought "we can
do better."

# Project Goals
Make LLMs accessible both to enthusiasts and more casual users.
Push LLM computation to the edge, in alignment with the [ggml project](http://ggml.ai/).
Give developers an easy way to integrate LLMs in a model agnostic way.

# Long Term
Push forward edge-LLMs by providing the tools to better test, measure, and compare them.

# Todos
Better instrumentation from the UI for when external programs are running things.
Better documentation.
Add system prompt implementation to llmrs.

# Backlog
- **OpenAI/Other Remote LLM Integration** — The entire architecure is designed to allow this,
and we're not currently taking advantage of it.
- **Headless/Terminal Mode** — Use terminal commands instead of the frontend.
- **Non-Text Models** —
- **Better parallelism** — (currently only one program/ui can use each LLM at a time).

# How You Can Help
## Add Models
I tried to include a decent set of 'known-good' models in the default model repository.
To be honest I've been too busy building this to spend a lot of time testing them,
so if a model belongs here, please pull request it. Help ranking the capabilities
of existing or new models would also be appreciated.

## Additional APIs
I've built a basic rust API just as a test of function. Improving it, or adding implementations
in other languages, would go a long way.

## Comprehensive Model Evaluation
Pantry should make it relatively simple to build software to more comprehensively evaluate
and compare local LLMs for different use cases. This would be incredibly valuable,
since it would allow the "capabilities" field to be based on more than just "vibes."

## Testing/CI
I'd love to have proper regression testing and automated CI. I just haven't had
the time to do it.

## Money stuff/Licences
This project itself is bundled under LGPL. You're free to bundle it into your
software, but if you integrate it into your code, your software must also be LGPL. What
that means is you're free to distribute the client with your own software (ie, to
use it to download and run LLMs), but if you want to use it as the basis for a new
client, that client must also be LGPL.

The offline client is open source and free forever. Any features added to it will be
free forever as well. I have some ideas for "teams" features for developers and
larger organizations. Those, by their nature, will require a web infrastructure
for management, customer support, etc etc. As a result those features will
almost certainly cost money.
