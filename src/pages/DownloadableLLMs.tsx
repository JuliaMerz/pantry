// src/pages/DownloadableLLMs.tsx

import React, { useEffect, useState } from 'react';
import Grid from '@mui/material/Grid';
import Button from '@mui/material/Button';
import Paper from '@mui/material/Paper';
import Modal from '@mui/material/Modal';
import TextField from '@mui/material/TextField';
import { Store } from "tauri-plugin-store-api";
import { fetch } from '@tauri-apps/api/http';
import { LLMRegistryRegistry, LLMRegistry, LLMRegistryEntry, toLLMRegistryEntry, LLMDownloadState, LLMRegistryEntryConnector} from '../interfaces';
import LLMDownloadableInfo from '../components/LLMDownloadableInfo';

const LLM_INFO_SOURCE = "https://raw.githubusercontent.com/JuliaMerz/pantry/master/models/index.json";

function DownloadableLLMs() {
  const [downloadableLLMs, setDownloadableLLMs] = useState<[LLMRegistryEntry, LLMRegistry][]>([]);
  const [registries, setRegistries] = useState<any>([]);

  async function getRegistries(): Promise<LLMRegistryRegistry> {
    return store.get("registries").
      then(async (registries:any) => {
        if (!registries) {
          console.log("Didn't find registries, adding");
          // would do setRegistries([]) but it's the default
          await store.set("registries", {local: []})
          await store.save()
          let defaultReg: LLMRegistry = {
            id: 'default',
            url: 'LLM_INFO_SOURCE',
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
          await store.set("registries", {local: []})
          await store.save()
          let defaultReg: LLMRegistry = {
            id: 'default',
            url: 'LLM_INFO_SOURCE',
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

    // Fetch data from the new URL and extract models
    const response = await fetch(url);
    const remoteData = await (response as any).json(); //it's upset about the .json()
    const models = remoteData.models;

    // Convert each model to an LLMRegistryEntry and add it to the registryEntries array
    for (const model of models) {
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
    await store.set("registries", registries);
    await store.save();

    setRegistries(registries)
  }

  useEffect(() => {
    const downloadableLLMs: [LLMRegistryEntry, LLMRegistry][] = [];
    getRegistries().then((regs) => {
      for (let reg_key in Object.keys(regs)) {
        downloadableLLMs.push(...(regs[reg_key].models.map((reg_entry): [LLMRegistryEntry, LLMRegistry] => [reg_entry, regs[reg_key]])));
        setDownloadableLLMs(downloadableLLMs);
      }
    });

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

    const [isModalOpen, setModalOpen] = useState(false);
    const produceEmptyRegistryEntry = (): LLMRegistryEntry => { return {
    id: '',
    name: '',
    family_id: '',
    organization: '',
    path: '',
    type: '',
    download_state: LLMDownloadState.NotDownloaded,
    backend_uuid: '',
    connector: LLMRegistryEntryConnector.Ggml, // provide a default value based on your LLMRegistryEntryConnector enum
    create_thread: false,
    description: '',
    requirements:'',
    licence: '',
    parameters: {}, // initialize with default LLMRegistry array
    user_parameters: [],
    capabilities: {}, // initialize with default capabilities object
    tags: [],
    url: '',
    config: {}, // initialize with default config object
    requirements: '',
  }}
  const [newRegistryEntry, setNewRegistryEntry] = useState<LLMRegistryEntry>(produceEmptyRegistryEntry());


  const validateNewRegistryEntry = (): boolean => {
    return Object.keys(newRegistryErrors).length == 0
  }
  const handleAddRegistry = async () => {
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
      setModalOpen(false);
      setNewRegistryEntry(produceEmptyRegistryEntry()); // reset to initial state
    } else {
      // Handle the validation error (show a message, highlight the invalid fields, etc.)
      throw Error("not validated");
    }
  };


  const [newRegistryErrors, setNewRegistryErrors] = useState<{ [key: string]: string }>({});
  const capitalizeFirstLetter = (string: string) => string.charAt(0).toUpperCase() + string.slice(1);

  const handleRegistryInputChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = event.target;

    // Updating the newRegistryEntry state
    setNewRegistryEntry({
      ...newRegistryEntry,
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




  return (
    <div>
      <h1>Downloadable Large Language Models</h1>
       <Button variant="contained" color="primary" onClick={() => setModalOpen(true)}>
        Add Registry
      </Button>
      {downloadableLLMs.map((pair) => (
        <LLMDownloadableInfo key={pair[0].id} llm={pair[0]} registry={pair[1]} />
      ))}
      <Modal open={isModalOpen} onClose={() => setModalOpen(false)}>
        <div className="new-llm-registry-entry-form">
          <h2>Add a new Registry Entry</h2>
          <Grid item xs={12}>
            {Object.keys(newRegistryEntry).map((key) =>
              key !== "config" && key !== "parameters" && key !== "capabilities" && (
                <TextField
                  error={!!newRegistryErrors[key]}
                  helperText={newRegistryErrors[key]}
                  fullWidth
                  name={key}
                  label={capitalizeFirstLetter(key)}
                  value={newRegistryEntry[key as keyof LLMRegistryEntry]}
                  onChange={handleRegistryInputChange}
                />
              )
            )}
            {/* Dynamic fields for config, parameters and capabilities */}
            {["capabilities"].map((key) =>
              (newRegistryEntry[key as keyof LLMRegistryEntry] as string[]).map((subKey, index) => (
                <Grid container item xs={12} key={index}>
                  <Grid item xs={6}>
                    <TextField
                      error={!!newRegistryErrors[`${key}Key${index}`]}
                      helperText={newRegistryErrors[`${key}Key${index}`]}
                      fullWidth
                      value={subKey}
                      label={`${capitalizeFirstLetter(key)} Key ${index + 1}`}
                      onChange={(e:any) => handleNumericKeyValueChange(e, index, key as NumericField, 0)}
                    />
                  </Grid>
                  <Grid item xs={6}>
                    <TextField
                      error={!!newRegistryErrors[`${key}Value${index}`]}
                      helperText={newRegistryErrors[`${key}Value${index}`]}
                      fullWidth
                      value={(newRegistryEntry[key as keyof LLMRegistryEntry] as {[key: string]: string})[subKey]}
                      label={`${capitalizeFirstLetter(key)} Value ${index + 1}`}
                      onChange={(e:any) => handleNumericKeyValueChange(e, index, key as NumericField, 1)}
                    />
                  </Grid>
                </Grid>
              ))
            )}
          </Grid>

            {["config", "parameters" ].map((key) =>
              (newRegistryEntry[key as keyof LLMRegistryEntry] as string[]).map((subKey, index) => (
                <Grid container item xs={12} key={index}>
                  <Grid item xs={6}>
                    <TextField
                      error={!!newRegistryErrors[`${key}Key${index}`]}
                      helperText={newRegistryErrors[`${key}Key${index}`]}
                      fullWidth
                      value={subKey}
                      label={`${capitalizeFirstLetter(key)} Key ${index + 1}`}
                      onChange={(e:any) => handleStringKeyValueChange(e, index, key as StringField, 0)}
                    />
                  </Grid>
                  <Grid item xs={6}>
                    <TextField
                      error={!!newRegistryErrors[`${key}Value${index}`]}
                      helperText={newRegistryErrors[`${key}Value${index}`]}
                      fullWidth
                      value={(newRegistryEntry[key as keyof LLMRegistryEntry] as {[key: string]: string})[subKey]}
                      label={`${capitalizeFirstLetter(key)} Value ${index + 1}`}
                      onChange={(e:any) => handleStringKeyValueChange(e, index, key as StringField, 1)}
                    />
                  </Grid>
                </Grid>
              ))
            )}
          </Grid>

          <Button variant="contained" color="primary" onClick={handleAddRegistry}>
            Submit
          </Button>
        </div>
      </Modal>
    </div>
  );
}

export default DownloadableLLMs;

