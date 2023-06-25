// src/pages/DownloadableLLMs.tsx

import React, { useEffect, useState } from 'react';
import Grid from '@mui/material/Grid';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import Checkbox from '@mui/material/Checkbox';
import FormControlLabel from '@mui/material/FormControlLabel';
import Button from '@mui/material/Button';
import Paper from '@mui/material/Paper';
import Modal from '@mui/material/Modal';
import TextField from '@mui/material/TextField';
import { LinearProgress } from '@mui/material';

import { Store } from "tauri-plugin-store-api";
import { fetch } from '@tauri-apps/api/http';
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/tauri';

import { LLMRegistryRegistry, LLMRegistry, LLMRegistryEntry, toLLMRegistryEntry, LLMDownloadState, LLMRegistryEntryConnector, LLMAvailable, toLLMAvailable } from '../interfaces';
import LLMDownloadableInfo from '../components/LLMDownloadableInfo';

const LLM_INFO_SOURCE = "https://raw.githubusercontent.com/JuliaMerz/pantry/master/models/index.json";

const REGISTRIES_STORAGE_KEY = "registries11";

const NEW_REG_HELPER_TEXT = {
  id: 'id/name of the registry',
  url: 'url of the models.json file'
};

const NEW_REGISTRY_HELPER_TEXT = {
    id: 'Technical Id, like openai-ada-high-temp-1',
    name: 'Human readable name',
    familyId: 'Family Id, ex "openai" or "llama"',
    organization: 'Human readable organization. Could be a github user.',
    homepage: 'URL for more information, like a HuggingFace page.',
    connector: 'Connector pantry needs to use to run this. When in doubt, probably GGML.',
    createThread: 'Yes if the model runs locally.',
    description: 'Human readable description of the model.',
    requirements:'Human readable—how much ram? GPU? etc.',
    license: 'MIT/Apache 2.0/etc.',
    parameters: 'Parameters set by the config, for ex hardcoded temperature.',
    userParameters: 'Parameters settable by the user when they call this model.',
    capabilities: 'Rated capabilities-Find the standard capabilities on the pantry github, and apply ratings to them. Capabilities left empty will be stored as "unrated". 0 represents "not capable".',
    tags: 'Comma separated tags, ex: "openai, gpt, conversational, remote"',
    url: 'Download URL for the model. Should be a ggml file atm.',
    config: 'Config to run the model. See pantry/github readme for details on what\'s required.',
  }

interface ProgressState {
  [key: string]: {progress: string, error: boolean};
}

function DownloadableLLMs() {
  const [downloadableLLMs, setDownloadableLLMs] = useState<[LLMRegistryEntry, LLMRegistry][]>([]);
  const [availableLLMs, setAvailableLLMs] = useState<LLMAvailable[]>([]);
  const [registries, setRegistries] = useState<any>([]);

  async function getRegistries(): Promise<LLMRegistryRegistry> {
    return store.get(REGISTRIES_STORAGE_KEY).
      then(async (registries:any) => {
        if (!registries) {
          console.log("Didn't find registries, adding");
          // would do setRegistries([]) but it's the default
          const local: LLMRegistry = {
            id: "local",
            url: "local",
            models: []
          }
          await store.set(REGISTRIES_STORAGE_KEY, {local: local})
          await store.save()
          let defaultReg: LLMRegistry = {
            id: 'default',
            url: LLM_INFO_SOURCE,
            models: []
          }
          await addRegistry(defaultReg.id, defaultReg.url);
          //We should hit the else case this time
          return getRegistries();
        } else {
          console.log("Found registries, returning");
          return registries;
        }
    }).catch(async (err:any) => {
          console.log("Didn't find registries, adding");
          // would do setRegistries([]) but it's the default
          await store.set(REGISTRIES_STORAGE_KEY, {local: []})
          await store.save()
          let defaultReg: LLMRegistry = {
            id: 'default',
            url: LLM_INFO_SOURCE,
            models: []
          }
          await addRegistry(defaultReg.id, defaultReg.url);
          //We should hit the else case this time
          return getRegistries();
    });

  }

  const store = new Store(".local.dat");
  useEffect(() => {

  });

  async function addRegistry(id: string, url: string) {
    const newReg:LLMRegistry = {
      id: id,
      url: url,
      models: [],
    };
    console.log("Running add registry");

    // Fetch data from the new URL and extract models
    const response = await fetch(url);
    console.log(response);
    const remoteData = response.data as any;
    const models = remoteData.models as LLMRegistryEntry[];
    console.log("models:", models);


    // Convert each model to an LLMRegistryEntry and add it to the registryEntries array
    for (const model of models) {
      console.log("pushign a model");
      const registryEntry:LLMRegistryEntry = await toLLMRegistryEntry(model);
      newReg.models.push(registryEntry);
    }


    // Save the updated registryEntries to the store

    // Save the updated registries list
    const registries:LLMRegistryRegistry = await getRegistries()
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

    setRegistries(registries)
  }

  const refreshData = async () => {

    const downloadableLLMs: [LLMRegistryEntry, LLMRegistry][] = [];

    getRegistries().then(async (regs) => {
      const result: {data: LLMAvailable[]} = await invoke<{data: LLMAvailable[]}>('available_llms');
      const llmAvail = result.data.map(toLLMAvailable);
      setAvailableLLMs(llmAvail);

      for (let regKey of Object.keys(regs)) {
        console.log("reg key {}, regs  models {}", regKey, regs[regKey], llmAvail)
              // Filter out already downloaded models based on id and backendUuid
        const filteredModels = regs[regKey].models.filter((regEntry) =>
          !llmAvail.some(llm => llm.id === regEntry.id && llm.uuid === regEntry.backendUuid)
        );
        const removedModels = regs[regKey].models.filter((regEntry) =>
          llmAvail.some(llm => llm.id === regEntry.id && llm.uuid === regEntry.backendUuid)
        );

        removedModels.map((value) => {
          // If we have them in llm_available then their download is actually complete already!
          value.downloadState = LLMDownloadState.Downloaded
        })

        downloadableLLMs.push(...(filteredModels.map((reg_entry): [LLMRegistryEntry, LLMRegistry] => [reg_entry, regs[regKey]])));
      }
      console.log("registries:", regs);
      setRegistries(regs);
      console.log("Updating store registries");
      await store.set(REGISTRIES_STORAGE_KEY, regs);
      await store.save();

      // downloadableLLMs.push(...(regs[regKey].models.map((reg_entry): [LLMRegistryEntry, LLMRegistry] => [reg_entry, regs[regKey]])));
      setDownloadableLLMs(downloadableLLMs);
    });

  }

  useEffect(() => {
    refreshData()

//     const fetchDownloadableLLMs = async () => {
//       try {
//         const response = await fetch(LLM_INFO_SOURCE);
//         console.log(response);
//         const data = await response.data
//         setDownloadableLLMs((data as any).models);
//       } catch (err) {
//         console.error(err);
//       }
//     };

  }, []);

    const [isRegistryEntryModalOpen, setRegistryEntryModalOpen] = useState(false);
    const [isRegistryModalOpen, setRegistryModalOpen] = useState(false);



    const produceEmptyRegistryEntry = (): LLMRegistryEntry => { return {
    id: '',
    name: '',
    familyId: '',
    organization: '',
    homepage: '',
    downloadState: LLMDownloadState.NotDownloaded,
    backendUuid: '',
    connectorType: LLMRegistryEntryConnector.Ggml, // provide a default value based on your LLMRegistryEntryConnector enum
    createThread: false,
    description: '',
    requirements:'',
    license: '',
    parameters: {}, // initialize with default LLMRegistry array
    userParameters: [],
    capabilities: {}, // initialize with default capabilities object
    tags: [],
    url: '',
    config: {}, // initialize with default config object
  }}
  const [newRegistryEntry, setNewRegistryEntry] = useState<LLMRegistryEntry>(produceEmptyRegistryEntry());
  const [newRegistry, setNewRegistry] = useState<{id: string, url: string}>({id: '', url: ''});


  const validateNewRegistryEntry = (): boolean => {
    return Object.keys(newRegistryEntryErrors).length == 0
  }
  const handleAddRegistryEntry = async () => {
    if (validateNewRegistryEntry()) {
      // Fetch the local registry and add the new entry
      const localRegistry:any = await store.get("local") || {};
      if (!localRegistry.entries) {
        localRegistry.entries = [];
      }
      localRegistry.entries.push(newRegistryEntry);

      // Save the updated local registry to the store
      await store.set("local", localRegistry);
      await store.save();

      // Close the modal and reset the newRegistryEntry state
      setRegistryEntryModalOpen(false);
      setNewRegistryEntry(produceEmptyRegistryEntry()); // reset to initial state
    } else {
      // Handle the validation error (show a message, highlight the invalid fields, etc.)
      throw Error("not validated");
    }
  };


  const [newRegistryEntryErrors, setNewRegistryEntryErrors] = useState<{ [key: string]: string }>({});
  const [newRegistryErrors, setNewRegistryErrors] = useState<{ [key: string]: string }>({});
  const capitalizeFirstLetter = (string: string) => string.charAt(0).toUpperCase() + string.slice(1);

  const handleCheckboxInputChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const { name } = event.target;
    setNewRegistryEntry({
      ...newRegistryEntry,
      [name]: !newRegistryEntry[name as keyof LLMRegistryEntry]
    })
    //We skip error checking because it's a _checkbox_
  }


  const handleRegistryInputChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = event.target;

    // Updating the newRegistryEntry state
    setNewRegistry({
      ...newRegistry,
      [name]: value,
    });

    // Performing validation
    if (value.trim() === '') {
      setNewRegistryErrors({
        ...newRegistryErrors,
        [name]: `${capitalizeFirstLetter(name)} is required.`,
      });
    } else {
      const { [name]: _, ...remainingErrors } = newRegistryErrors; // Remove error
      setNewRegistryErrors(remainingErrors);
    }
  };


  const handleRegistryEntryInputChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = event.target;

    // Updating the newRegistryEntry state
    setNewRegistryEntry({
      ...newRegistryEntry,
      [name]: value,
    });

    // Performing validation
    if (value.trim() === '') {
      setNewRegistryEntryErrors({
        ...newRegistryEntryErrors,
        [name]: `${capitalizeFirstLetter(name)} is required.`,
      });
    } else {
      const { [name]: _, ...remainingErrors } = newRegistryEntryErrors; // Remove error
      setNewRegistryEntryErrors(remainingErrors);
    }
  };


  // Create dynamic fields
  type StringField = 'config' | 'parameters';
  type NumericField = 'capabilities';

  const [dynamicKeyValuePairs, setDynamicKeyValuePairs] = useState<Record<StringField|NumericField, [string, string][]>>({
    config: [['', '']],
    parameters: [['', '']],
    capabilities: [['', '']],
  });


  // Handle changes for dynamic fields

  const handleStringKeyValueChange = (event: React.ChangeEvent<HTMLInputElement>, index: number, fieldType: StringField, keyOrValue: number) => {
    const newPairs = { ...dynamicKeyValuePairs };
    newPairs[fieldType][index][keyOrValue] = event.target.value;
    setDynamicKeyValuePairs(newPairs);

    const newEntry = { ...newRegistryEntry };
    newEntry[fieldType] = Object.fromEntries(newPairs[fieldType].filter(([key, value]) => key !== '' || value !== ''));
    setNewRegistryEntry(newEntry);
  };

  const handleNumericKeyValueChange = (event: React.ChangeEvent<HTMLInputElement>, index: number, fieldType: NumericField, keyOrValue: number) => {
    const newPairs = { ...dynamicKeyValuePairs };
    newPairs[fieldType][index][keyOrValue] = event.target.value
    setDynamicKeyValuePairs(newPairs);

    const newEntry = { ...newRegistryEntry };
    newEntry[fieldType] = Object.fromEntries(newPairs[fieldType].filter(([key, value]) => key !== '' || value !== '').map(([key, value])=> [key, parseFloat(value)]));
    setNewRegistryEntry(newEntry);
  };


  // Automatically add new rows
  useEffect(() => {
    (Object.keys(dynamicKeyValuePairs) as StringField[]|NumericField[]).forEach((fieldType: StringField|NumericField) => {
      const lastPair = dynamicKeyValuePairs[fieldType][dynamicKeyValuePairs[fieldType].length - 1];
      if (lastPair[0] !== '' || lastPair[1] !== '') {
        setDynamicKeyValuePairs(prev => ({
          ...prev,
          [fieldType]: [...prev[fieldType], ['', '']],
        }));
      }
    });
  }, [dynamicKeyValuePairs]);



  const beginDownload = (llmId: string, regUrl: string, index:number, uuid: string) => {
    store.get(REGISTRIES_STORAGE_KEY)
      .then(async (out) => {
        console.log(out);
        // We get the registries back
        const registries: {[id: string]: LLMRegistry} = out as {[id: string]: LLMRegistry}
        const targetModel = registries[regUrl].models.find((value) => value.id == llmId) as LLMRegistryEntry;
        targetModel.backendUuid = uuid;
        targetModel.downloadState = LLMDownloadState.Downloading;

        setDownloadableLLMs(prevState => {
          const newState = [...prevState];
          newState[index] = [targetModel, newState[index][1]]; // Updating the specific item at index
          return newState;
        });

        setRegistries((prevRegistries:{[id: string]:LLMRegistry}) => {
          const newRegistries = {...prevRegistries};
          console.error("newRegistries", newRegistries);
          newRegistries[regUrl] = {...newRegistries[regUrl], models: newRegistries[regUrl].models.map((model: LLMRegistryEntry) =>
            model.id === llmId ? targetModel : model // Updating the specific model in the registry
          )};
          store.set(REGISTRIES_STORAGE_KEY, newRegistries);
          store.save()
          return newRegistries;
        });
      });

  }

  const completeDownload = (llmId: string, regUrl: string) => {
    refreshData();
  };


  return (
    <div>
      <h1>Downloadable Large Language Models</h1>
       <Button variant="contained" color="primary" onClick={() => setRegistryModalOpen(true)}>
        Add Registry
      </Button>
       <Button variant="contained" color="primary" onClick={() => setRegistryEntryModalOpen(true)}>
        Add Registry Entry
      </Button>
       <Button variant="contained" color="primary" onClick={refreshData}>
        Refresh
      </Button>
      {downloadableLLMs.map((pair, index) => (
        <LLMDownloadableInfo key={pair[0].id} llm={pair[0]} registry={pair[1]} beginDownload={(uuid) => {beginDownload(pair[0].id, pair[1].url, index, uuid)}} completeDownload={() => { completeDownload(pair[0].id, pair[1].url)}}/>
      ))}
      <Modal open={isRegistryModalOpen} className="form-modal" onClose={() => setRegistryModalOpen(false)}>
        <div className="new-llm-registry-form form-div">
          <h2>Add a new Registry</h2>
            {Object.keys(newRegistry).map((key) =>
            <TextField
                className="input-field"
                error={!!newRegistryErrors[key]}
                helperText={newRegistryErrors[key] ? newRegistryErrors[key] : NEW_REG_HELPER_TEXT[key as keyof typeof NEW_REG_HELPER_TEXT]}
                fullWidth
                name={key}
                label={capitalizeFirstLetter(key)}
                value={newRegistry[key as keyof typeof newRegistry]}
                onChange={handleRegistryInputChange}
              />
                                         )}
        </div>
      </Modal>
      <Modal open={isRegistryEntryModalOpen} className="form-modal" onClose={() => setRegistryEntryModalOpen(false)}>
        <div className="new-llm-registry-entry-form form-div">
          <h2>Add a new Registry Entry</h2>
          <Grid item xs={12}>
            {Object.keys(newRegistryEntry).map((key) =>
              key !== "config" && key !== "parameters" && key !== "capabilities" && key !== "backendUuid" && key !== "downloadState" && (typeof newRegistryEntry[key as keyof LLMRegistryEntry] === "boolean" ? (

        <FormControlLabel labelPlacement="start" control={<Checkbox
checked={newRegistryEntry[key as keyof LLMRegistryEntry] as boolean}
          onChange={handleCheckboxInputChange}
          name={key}
          color="primary"
        />} label={capitalizeFirstLetter(key)} />

      ) : key === "connector" ? (

        <FormControlLabel labelPlacement="start" control={
        <Select
          value={newRegistryEntry[key as keyof LLMRegistryEntry]}
          onChange={handleRegistryEntryInputChange}
          name={key}
        >
          <MenuItem value="ggml">GGML</MenuItem>
          <MenuItem value="openai">OpenAI</MenuItem>
        </Select>
        } label={capitalizeFirstLetter(key)} />

      ) : (

              <TextField
                  className="input-field"
                  error={!!newRegistryEntryErrors[key]}
                  helperText={newRegistryEntryErrors[key] ? newRegistryEntryErrors[key] : NEW_REGISTRY_HELPER_TEXT[key as keyof typeof NEW_REGISTRY_HELPER_TEXT]}
                  fullWidth
                  name={key}
                  label={capitalizeFirstLetter(key)}
                  value={newRegistryEntry[key as keyof LLMRegistryEntry]}
                  onChange={handleRegistryEntryInputChange}
                />
              )
            ))}
            {["capabilities"].map((key) =>
              [<h6>{NEW_REGISTRY_HELPER_TEXT[key as keyof typeof NEW_REGISTRY_HELPER_TEXT]}</h6>,
              (Object.keys(dynamicKeyValuePairs[key as NumericField])).map((subKey, index) => (
                <Grid container item xs={12} key={index}>
                  <Grid item xs={6}>
                    <TextField
                      className="input-field"
                      error={!!newRegistryEntryErrors[`${key}Key${index}`]}
                      helperText={newRegistryEntryErrors[`${key}Value${index}`]}
                      fullWidth
                      value={dynamicKeyValuePairs[key as NumericField][index][0]}
                      label={`${capitalizeFirstLetter(key)} Key ${index + 1}`}
                      onChange={(e:any) => handleNumericKeyValueChange(e, index, key as NumericField, 0)}
                    />
                  </Grid>
                  <Grid item xs={6}>
                    <TextField
                      className="input-field"
                      error={!!newRegistryEntryErrors[`${key}Value${index}`]}
                      helperText={newRegistryEntryErrors[`${key}Value${index}`]}
                      fullWidth
                      value={dynamicKeyValuePairs[key as NumericField][index][0]}
                      label={`${capitalizeFirstLetter(key)} Value ${index + 1}`}
                      onChange={(e:any) => handleNumericKeyValueChange(e, index, key as NumericField, 1)}
                    />
                  </Grid>
                </Grid>
              ))]
            )}
          </Grid>

            {["config", "parameters" ].map((key) =>
              [<h6>{NEW_REGISTRY_HELPER_TEXT[key as keyof typeof NEW_REGISTRY_HELPER_TEXT]}</h6>,
              (Object.keys(dynamicKeyValuePairs[key as StringField]) ).map((subKey, index) => (
                <Grid container item xs={12} key={index}>
                  <Grid item xs={6}>
                    <TextField
                      className="input-field"
                      error={!!newRegistryEntryErrors[`${key}Key${index}`]}
                      helperText={newRegistryEntryErrors[`${key}Key${index}`]}
                      fullWidth
                      value={dynamicKeyValuePairs[key as StringField][index][0]}
                      label={`${capitalizeFirstLetter(key)} Key ${index + 1}`}
                      onChange={(e:any) => handleStringKeyValueChange(e, index, key as StringField, 0)}
                    />
                  </Grid>
                  <Grid item xs={6}>
                    <TextField
                      className="input-field"
                      error={!!newRegistryEntryErrors[`${key}Value${index}`]}
                      helperText={newRegistryEntryErrors[`${key}Value${index}`]}
                      fullWidth
                      value={dynamicKeyValuePairs[key as StringField][index][1]}
                      label={`${capitalizeFirstLetter(key)} Value ${index + 1}`}
                      onChange={(e:any) => handleStringKeyValueChange(e, index, key as StringField, 1)}
                    />
                  </Grid>
                </Grid>
              ))]
            )}

          <Button variant="contained" color="primary" onClick={handleAddRegistryEntry}>
            Submit
          </Button>
        </div>
      </Modal>
    </div>
  );
}

export default DownloadableLLMs;

