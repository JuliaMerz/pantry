import {LLMRegistryEntry, LLMRegistryEntryConnector, LLMDownloadState, LLMRegistryRegistry, LLMRegistry, toLLMRegistryEntryExternal, LLMAvailable, toLLMAvailable} from "./interfaces";
import {Store} from "tauri-plugin-store-api";
import {produceEmptyRegistryEntry, fromLLMRegistryEntry} from "./interfaces";
import {invoke} from '@tauri-apps/api/tauri';

const LLM_INFO_SOURCE = "https://raw.githubusercontent.com/JuliaMerz/pantry/master/models/index.json";
const REGISTRIES_STORAGE_KEY = "registries20";

const store = new Store(".local.dat");

export async function getRegistries(forceRemoteRefresh: boolean): Promise<LLMRegistryRegistry> {
  return store.get(REGISTRIES_STORAGE_KEY).
    then(async (registries: any) => {
      // This means we don't ahve any registries stored. Start with default.
      if (!registries) {
        console.log("Didn't find registries, adding");
        // would do setRegistries([]) but it's the default
        const local: LLMRegistry = {
          id: "local",
          url: "local",
          models: {}
        }
        const shared: LLMRegistry = {
          id: "shared",
          url: "shared",
          models: {}
        }
        await store.set(REGISTRIES_STORAGE_KEY, {local: local, shared: shared})
        await store.save()
        let defaultReg: LLMRegistry = {
          id: 'default',
          url: LLM_INFO_SOURCE,
          models: {}
        }
        await addRegistry(defaultReg.id, defaultReg.url);
        //We should hit the else case this time
        return getRegistries(false);
      } else {
        let regs = registries as LLMRegistryRegistry;
        console.log("Found registries, returning");

        // If force refresh, we want to redownload all models
        if (forceRemoteRefresh) {
          console.log("registries:", registries);
          console.log("Updating store registries");
          await store.set(REGISTRIES_STORAGE_KEY, regs);
          await store.save();

          for (let url in Object.keys(regs)) {
            if (url === 'local' || url === 'shared') {
              continue;
            }
            fetch(url).then((response) => {
              console.log(response);
              const remoteData = (response as any).data as any;
              const models = remoteData.models as {[id: string]: LLMRegistryEntry};
              console.log("models:", models);

              // Convert each model to an LLMRegistryEntry and add it to the registryEntries array
              // FOR BACKEND MODELS WE ASSUME UNIQUE IDs. THIS DOES NOT HOLD TRUE FOR LOCAL/SHARED
              // this makes sense when you consider that backend comes from one service
              // whereas local/shared comes from many people
              Object.entries(models).forEach(([key, model], index) => {
                console.log("pushign a model");
                const registryEntry: LLMRegistryEntry = toLLMRegistryEntryExternal(model);
                regs[url].models[key] = registryEntry;
              });

            }).catch((reason) => {
              console.log("error fetching, skipping");
            });
          }

        }

        return regs;
      }
    }).catch(async (err: any) => {
      console.log("Didn't find registries, adding");
      // would do setRegistries([]) but it's the default
      const local: LLMRegistry = {
        id: "local",
        url: "local",
        models: {}
      }
      const shared: LLMRegistry = {
        id: "shared",
        url: "shared",
        models: {}
      }
      await store.set(REGISTRIES_STORAGE_KEY, {local: local, shared: shared})
      await store.save()
      let defaultReg: LLMRegistry = {
        id: 'default',
        url: LLM_INFO_SOURCE,
        models: {}
      }
      await addRegistry(defaultReg.id, defaultReg.url);
      //We should hit the else case this time
      return getRegistries(false);
    });

}


export const validateRegistryEntry = (entry: LLMRegistryEntry) => {
  let errors: {[key: string]: string} = {};

  if (entry.name.trim() === '') {
    errors.name = 'Name is required.';
  }

  if (entry.connectorType == LLMRegistryEntryConnector.Ggml) {
    if (entry.url.trim() === '') {
      errors.url = 'URL is required for ggml models.';
    }


  } else if (entry.connectorType == LLMRegistryEntryConnector.OpenAI) {
    if (entry.url.trim() !== '') {
      errors.url = 'OpenAI models cannot have a url';
    }
  }

  return errors
}

const capitalizeFirstLetter = (string: string) => string.charAt(0).toUpperCase() + string.slice(1);


export async function addRegistry(id: string, url: string): Promise<LLMRegistryRegistry> {
  const newReg: LLMRegistry = {
    id: id,
    url: url,
    models: {},
  };
  console.log("Running add registry");

  // Fetch data from the new URL and extract models
  const response = await fetch(url);
  console.log(response);
  const remoteData = (response as any).data as any;
  const models = remoteData.models as LLMRegistryEntry[];
  console.log("models:", models);


  // Convert each model to an LLMRegistryEntry and add it to the registryEntries array
  for (const model of models) {
    console.log("pushign a model");
    const registryEntry: LLMRegistryEntry = await toLLMRegistryEntryExternal(model);
    newReg.models[registryEntry.id] = registryEntry;
  }


  // Save the updated registryEntries to the store

  // Save the updated registries list
  const registries: LLMRegistryRegistry = await getRegistries(false);
  if (!registries[url]) {
    registries[url] = newReg;
  } else {
    // This registry already exists, we need to either
    if (registries[url].id !== id)
      throw Error("Registry url already exists, but type doesn't match up. Aborting.")
    registries[url] = newReg;
  }

  console.log("Updating store registries");
  await store.set(REGISTRIES_STORAGE_KEY, registries);
  await store.save();

  return registries;
}

export async function addRegistryEntry(model: LLMRegistryEntry, location: string): Promise<string> {
  let regs = await getRegistries(false);
  console.log("looking in", location, regs);
  if (model.id in regs[location].models) {
    let counter = 1
    while (model.id + '-' + counter in regs[location].models) {
      counter += 1
    }
    model.id = model.id + '-' + counter;
  }

  console.log("Final model id: ", model.id);
  regs[location].models[model.id] = model
  await store.set(REGISTRIES_STORAGE_KEY, regs);
  await store.save();
  return model.id
}


export async function downloadLLM(llm: LLMRegistryEntry, regUrl: string) {
  const result = await invoke('download_llm', {llmReg: fromLLMRegistryEntry(llm)});
  const backendUuid = (result as any).data.uuid;

  return getRegistries(false).then((regs) => {

    const targetModel = regs[regUrl].models[llm.id];

    targetModel.backendUuid = backendUuid;
    targetModel.downloadState = LLMDownloadState.Downloading;


    store.set(REGISTRIES_STORAGE_KEY, regs)
    store.save()
  });

}

export async function deleteRegistryEntry(llm: LLMRegistryEntry, registry: LLMRegistry) {
  let regs = await getRegistries(false);

  let location = registry.url;

  let change = regs[location]
  const id = llm.id

  const {[id]: deleted, ...remainder} = change.models;
  change.models = remainder;

  await store.set(REGISTRIES_STORAGE_KEY, regs);
  await store.save();
  return deleted;
}
