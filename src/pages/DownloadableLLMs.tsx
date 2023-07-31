// src/pages/DownloadableLLMs.tsx

import React, {useEffect, useState} from 'react';
import {
  Box,
  InputAdornment,
  Typography,
  Card,
  CardContent,
  Divider,
  Grid,
  Select,
  InputLabel,
  MenuItem,
  Checkbox,
  FormControlLabel,
  Button,
  Paper,
  Modal,
  TextField,
} from '@mui/material';
import {SelectChangeEvent} from "@mui/material";

import SearchIcon from '@mui/icons-material/Search';
import {LinearProgress} from '@mui/material';

import {ModalBox} from '../theme';

import {Store} from "tauri-plugin-store-api";
import {fetch} from '@tauri-apps/api/http';
import {listen} from '@tauri-apps/api/event'
import {invoke} from '@tauri-apps/api/tauri';
import {validateRegistryEntry, addRegistry, getRegistries, addRegistryEntry, downloadLLM} from '../registryHelpers';

import {LLMRegistryRegistry, LLMRegistry, LLMRegistryEntry, toLLMRegistryEntry, LLMDownloadState, LLMRegistryEntryConnector, LLMAvailable, toLLMAvailable, produceEmptyRegistryEntry} from '../interfaces';
import LLMDownloadableInfo from '../components/LLMDownloadableInfo';

const LLM_INFO_SOURCE = "https://raw.githubusercontent.com/JuliaMerz/pantry/master/models/index.json";

const REGISTRIES_STORAGE_KEY = "registries15";

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
  local: 'Yes if the model runs locally.',
  description: 'Human readable description of the model.',
  requirements: 'Human readable—how much ram? GPU? etc.',
  license: 'MIT/Apache 2.0/etc.',
  parameters: 'Preset parameters, for ex hardcoded temperature. Overrides system defaults.',
  userParameters: 'User overwritable parameters (at calltime, via UI or API).',
  sessionParameters: 'Session Parameters set by the config—there\'s very few or none of these.',
  userSessionParameters: 'User overwritable session parameters.',
  capabilities: 'Rated capabilities-Find the standard capabilities on the pantry github, and apply ratings to them. Capabilities left empty will be stored as "unrated". 0 represents "not capable".',
  tags: 'Comma separated tags, ex: "openai, gpt, conversational, remote"',
  url: 'Download URL for the model. Should be a ggml file atm.',
  config: 'Special config to run the model. Usually unnecessary—see the readme.',
}

interface ProgressState {
  [key: string]: {progress: string, error: boolean};
}

function DownloadableLLMs() {
  const [downloadableLLMs, setDownloadableLLMs] = useState<[LLMRegistryEntry, LLMRegistry][]>([]);
  const [availableLLMs, setAvailableLLMs] = useState<LLMAvailable[]>([]);
  const [registries, setRegistries] = useState<any>([]);
  const [llmFilter, setLLMFilter] = useState("");

  // Special fields for special handling

  // used for the llmrs connector
  const [modelArchitecture, setModelArchitecture] = useState('llama');

  useEffect(() => {
    refreshData(false)

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



  const [newRegistryEntry, setNewRegistryEntry] = useState<LLMRegistryEntry>(produceEmptyRegistryEntry());
  const [newRegistry, setNewRegistry] = useState<{id: string, url: string}>({id: '', url: ''});
  const [newRegistryEntryErrors, setNewRegistryEntryErrors] = useState<{[key: string]: string}>({});
  const [newRegistryErrors, setNewRegistryErrors] = useState<{[key: string]: string}>({});
  //
  // Create dynamic fields
  type StringField = 'config' | 'parameters' | 'sessionParameters';
  type NumericField = 'capabilities';

  const [dynamicKeyValuePairs, setDynamicKeyValuePairs] = useState<Record<StringField | NumericField, [string, string][]>>({
    config: [['', '']],
    parameters: [['', '']],
    sessionParameters: [['', '']],
    capabilities: Object.entries(newRegistryEntry.capabilities).filter(
      ([key, value], index) => key !== '' || value !== ''
    )
  });



  const validateNewRegistryEntry = (): boolean => {
    return Object.keys(newRegistryEntryErrors).length == 0
  }

  const handleAddRegistry = async () => {
    addRegistry(newRegistry.id, newRegistry.url).then((registries: LLMRegistryRegistry) => {
      setRegistries(registries);
      setNewRegistry({id: '', url: ''});
    });
  }

  const handleAddRegistryEntry = async () => {
    if (validateNewRegistryEntry()) {
      // Fetch the local registry and add the new entry

      applySpecialFields(newRegistryEntry);
      addRegistryEntry(newRegistryEntry, 'local').then(() => refreshData(false));

      // Close the modal and reset the newRegistryEntry state
      setRegistryEntryModalOpen(false);
      setNewRegistryEntry(produceEmptyRegistryEntry()); // reset to initial state
    } else {
      // Handle the validation error (show a message, highlight the invalid fields, etc.)
      throw Error("not validated");
    }
  };

  const refreshData = async (forceRefresh: boolean) => {

    const downloadableLLMs: [LLMRegistryEntry, LLMRegistry][] = [];

    getRegistries(forceRefresh).then(async (regs) => {
      const result: {data: LLMAvailable[]} = await invoke<{data: LLMAvailable[]}>('available_llms');
      const llmAvail = result.data.map(toLLMAvailable);
      setAvailableLLMs(llmAvail);

      for (let regKey of Object.keys(regs)) {
        console.log("reg key {}, regs  models {}", regKey, regs[regKey], llmAvail)
        // Filter out already downloaded models based on id and backendUuid
        const filteredModels = Object.entries(regs[regKey].models).filter((regEntry) =>
          !llmAvail.some(llm => llm.id === regEntry[1].id && llm.uuid === regEntry[1].backendUuid)
        ).map(([key, value]) => value);
        const removedModels = Object.entries(regs[regKey].models).filter(([key, regEntry]) =>
          llmAvail.some(llm => llm.id === regEntry.id && llm.uuid === regEntry.backendUuid)
        ).map(([key, value]) => value);

        removedModels.map((value) => {
          // If we have them in llm_available then their download is actually complete already!
          value.downloadState = LLMDownloadState.Downloaded
        })

        downloadableLLMs.push(...(filteredModels.map((reg_entry): [LLMRegistryEntry, LLMRegistry] => [reg_entry, regs[regKey]])));
      }
      setRegistries(regs);

      // downloadableLLMs.push(...(regs[regKey].models.map((reg_entry): [LLMRegistryEntry, LLMRegistry] => [reg_entry, regs[regKey]])));
      setDownloadableLLMs(downloadableLLMs);
    });

  }


  const capitalizeFirstLetter = (string: string) => string.charAt(0).toUpperCase() + string.slice(1);

  const handleCheckboxInputChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const {name} = event.target;
    setNewRegistryEntry({
      ...newRegistryEntry,
      [name]: !newRegistryEntry[name as keyof LLMRegistryEntry]
    })
    //We skip error checking because it's a _checkbox_
  }


  const handleRegistryInputChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const {name, value} = event.target;

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
      const {[name]: _, ...remainingErrors} = newRegistryErrors; // Remove error
      setNewRegistryErrors(remainingErrors);
    }
  };


  const handleRegistryEntryInputChange = (event: SelectChangeEvent | React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    const {name, value} = event.target;

    // Updating the newRegistryEntry state
    setNewRegistryEntry({
      ...newRegistryEntry,
      [name]: value,
    });

    // Use imported error function from here on
    setNewRegistryErrors(validateRegistryEntry({
      ...newRegistryEntry,
      [name]: value,
    }));
  };



  // Handle changes for dynamic fields

  const handleStringKeyValueChange = (event: React.ChangeEvent<HTMLInputElement>, index: number, fieldType: StringField, keyOrValue: number) => {
    const newPairs = {...dynamicKeyValuePairs};
    newPairs[fieldType][index][keyOrValue] = event.target.value;
    let len = newPairs[fieldType].length;
    newPairs[fieldType] = newPairs[fieldType].filter(
      ([key, value], index) => key !== '' || value !== '' || index == len - 1
    );
    setDynamicKeyValuePairs(newPairs);

    const newEntry = {...newRegistryEntry};
    newEntry[fieldType] = Object.fromEntries(newPairs[fieldType].filter(([key, value]) => key !== '' || value !== ''));
    console.log("newEntry", newEntry, newEntry[fieldType]);
    setNewRegistryEntry(newEntry);
  };

  //TODO: if there's a numbers field that isn't capabilites this will protest
  const handleNumberButReallyCapabilitiesChange = (event: React.ChangeEvent<HTMLInputElement>, index: number, fieldType: NumericField, keyOrValue: number) => {
    const newPairs = {...dynamicKeyValuePairs};
    newPairs[fieldType][index][keyOrValue] = event.target.value
    let len = newPairs[fieldType].length;
    newPairs[fieldType] = newPairs[fieldType].filter(
      ([key, value], index) => key !== '' || value !== '' || index == len - 1
    );
    setDynamicKeyValuePairs(newPairs);

    const newEntry = {...newRegistryEntry};
    newEntry[fieldType] = Object.fromEntries(newPairs[fieldType].filter(([key, value]) => key !== '' || value !== '').map(([key, value]) => [key, parseFloat(value)])) as {assistant: number, coding: number, writer: number};
    console.log("newEntry", newEntry, newEntry[fieldType]);
    setNewRegistryEntry(newEntry);
  };


  // Automatically add new rows
  useEffect(() => {
    (Object.keys(dynamicKeyValuePairs) as StringField[] | NumericField[]).forEach((fieldType: StringField | NumericField) => {
      //for now don't grow capabiliteis
      if (fieldType == "capabilities")
        return
      const lastPair = dynamicKeyValuePairs[fieldType][dynamicKeyValuePairs[fieldType].length - 1];
      if (lastPair[0] !== '' || lastPair[1] !== '') {
        setDynamicKeyValuePairs(prev => ({
          ...prev,
          [fieldType]: [...prev[fieldType], ['', '']],
        }));
      }
    });
  }, [dynamicKeyValuePairs]);



  const beginDownload = async (llm: LLMRegistryEntry, regUrl: string) => {
    // We should be able to do this with just saving then refresh_data
    await downloadLLM(llm, regUrl);

    refreshData(false);
  }


  // const beginDownload = (llmId: string, regUrl: string, index: number, uuid: string) => {
  // store.get(REGISTRIES_STORAGE_KEY)
  //   .then(async (out) => {
  //     console.log(out);
  //     // We get the registries back
  //     const registries: {[id: string]: LLMRegistry} = out as {[id: string]: LLMRegistry}
  //     const targetModel = registries[regUrl].models.find((value) => value.id == llmId) as LLMRegistryEntry;
  //     targetModel.backendUuid = uuid;
  //     targetModel.downloadState = LLMDownloadState.Downloading;

  //     setDownloadableLLMs(prevState => {
  //       const newState = [...prevState];
  //       newState[index] = [targetModel, newState[index][1]]; // Updating the specific item at index
  //       return newState;
  //     });

  //     setRegistries((prevRegistries: {[id: string]: LLMRegistry}) => {
  //       const newRegistries = {...prevRegistries};
  //       newRegistries[regUrl] = {
  //         ...newRegistries[regUrl], models: newRegistries[regUrl].models.map((model: LLMRegistryEntry) =>
  //           model.id === llmId ? targetModel : model // Updating the specific model in the registry
  //         )
  //       };
  //       store.set(REGISTRIES_STORAGE_KEY, newRegistries);
  //       store.save()
  //       return newRegistries;
  //     });
  //   });


  const completeDownload = (llmId: string, regUrl: string) => {
    refreshData(false);
  };

  const renderSpecialFields = () => {
    if (newRegistryEntry.connectorType == LLMRegistryEntryConnector.LLMrs) {
      return (
        <Box sx={{
          display: 'flex',
          flexDirection: 'row',
          padding: 1,
          margin: 2,
          alignItems: 'center',
          border: 1,
          borderRadius: 1,
          borderColor: 'grey.300',

        }}>
          <Typography>
            Model Architectue
          </Typography>
          <Select
            value={modelArchitecture}
            onChange={(event) => {setModelArchitecture(event.target.value)}}
            name="modelArchitecture"
            sx={{
              marginX: 2
            }}
          >
            <MenuItem value="llama">LLaMA</MenuItem>
            <MenuItem value="bloom">BLOOM</MenuItem>
            <MenuItem value="gpt2">GPT-2</MenuItem>
            <MenuItem value="gptj">GPT-J</MenuItem>
            <MenuItem value="neox">GPT-NeoX</MenuItem>
          </Select></Box>
      )
    }
  }

  const applySpecialFields = (llmRegEntry: LLMRegistryEntry) => {
    if (newRegistryEntry.connectorType == LLMRegistryEntryConnector.LLMrs) {
      llmRegEntry.config.model_architecture = modelArchitecture;
    }
  }

  const renderField = (key: keyof LLMRegistryEntry) => {
    switch (key) {
      case "backendUuid":
      case "downloadState":
        return null;
      case "capabilities":
        return (
          <Box>
            <Typography variant="subtitle2">{NEW_REGISTRY_HELPER_TEXT[key as keyof typeof NEW_REGISTRY_HELPER_TEXT]}</Typography>
            {Object.keys(dynamicKeyValuePairs[key as NumericField]).map((subKey, index) => (
              <Grid container item xs={12} key={index}>
                <Grid item xs={6}>
                  <TextField
                    error={!!newRegistryEntryErrors[`${key}Key${index}`]}
                    helperText={newRegistryEntryErrors[`${key}Value${index}`]}
                    fullWidth
                    value={dynamicKeyValuePairs[key as NumericField][index][0]}
                    label={`${capitalizeFirstLetter(key)} Key ${index + 1}`}
                    onChange={(e: any) => handleNumberButReallyCapabilitiesChange(e, index, key as NumericField, 0)}
                    disabled
                  />
                </Grid>
                <Grid item xs={6}>
                  <TextField
                    error={!!newRegistryEntryErrors[`${key}Value${index}`]}
                    helperText={newRegistryEntryErrors[`${key}Value${index}`]}
                    fullWidth
                    value={dynamicKeyValuePairs[key as NumericField][index][1]}
                    label={`${capitalizeFirstLetter(key)} Value ${index + 1}`}
                    onChange={(e: any) => handleNumberButReallyCapabilitiesChange(e, index, key as NumericField, 1)}
                  />
                </Grid>
              </Grid>
            ))}
          </Box>
        )

        break;

      // Boolean
      case "local":
        return (
          <Box>
            <FormControlLabel labelPlacement="start" control={<Checkbox
              checked={newRegistryEntry[key as keyof LLMRegistryEntry] as boolean}
              onChange={handleCheckboxInputChange}
              name={key}
              color="primary"
            />} label={capitalizeFirstLetter(key)} />
          </Box>
        )

      //doubles
      case "config":
      case "parameters":
      case "sessionParameters":
        return (
          <Box>
            <Typography variant="subtitle2">{NEW_REGISTRY_HELPER_TEXT[key as keyof typeof NEW_REGISTRY_HELPER_TEXT]}</Typography>
            {Object.keys(dynamicKeyValuePairs[key as StringField]).map((subKey, index) => (
              <Grid container item xs={12} key={index}>
                <Grid item xs={6}>
                  <TextField
                    error={!!newRegistryEntryErrors[`${key}Key${index}`]}
                    helperText={newRegistryEntryErrors[`${key}Key${index}`]}
                    fullWidth
                    value={dynamicKeyValuePairs[key as StringField][index][0]}
                    label={`${capitalizeFirstLetter(key)} Key ${index + 1}`}
                    onChange={(e: any) => handleStringKeyValueChange(e, index, key as StringField, 0)}
                  />
                </Grid>
                <Grid item xs={6}>
                  <TextField
                    error={!!newRegistryEntryErrors[`${key}Value${index}`]}
                    helperText={newRegistryEntryErrors[`${key}Value${index}`]}
                    fullWidth
                    value={dynamicKeyValuePairs[key as StringField][index][1]}
                    label={`${capitalizeFirstLetter(key)} Value ${index + 1}`}
                    onChange={(e: any) => handleStringKeyValueChange(e, index, key as StringField, 1)}
                  />
                </Grid>
              </Grid>
            ))}
          </Box>
        );
        break


      case "connectorType":
        return (
          <Box>
            <FormControlLabel labelPlacement="start" control={
              <Select
                value={newRegistryEntry[key as keyof LLMRegistryEntry] as string}
                onChange={handleRegistryEntryInputChange}
                name={key}
                sx={{
                  marginX: 2

                }}
              >
                <MenuItem value="llmrs">llmrs</MenuItem>
                <MenuItem value="openai">OpenAI</MenuItem>
              </Select>
            } label={capitalizeFirstLetter(key)} />
            {renderSpecialFields()}
            <Divider role="presentation">If you intend to share publicly, please also fill in...</Divider>
          </Box>
        );
        break;

      case "userParameters":
      case "userSessionParameters":
        return (
          <Box>
            <TextField
              error={!!newRegistryEntryErrors[key]}
              helperText={newRegistryEntryErrors[key] ? newRegistryEntryErrors[key] : NEW_REGISTRY_HELPER_TEXT[key as keyof typeof NEW_REGISTRY_HELPER_TEXT]}
              fullWidth
              name={key}
              label={capitalizeFirstLetter(key)}
              value={newRegistryEntry[key as keyof LLMRegistryEntry]}
              onChange={handleRegistryEntryInputChange}
            />
          </Box>);
        break;



      default:
        return (
          <Box>
            <TextField
              error={!!newRegistryEntryErrors[key]}
              helperText={newRegistryEntryErrors[key] ? newRegistryEntryErrors[key] : NEW_REGISTRY_HELPER_TEXT[key as keyof typeof NEW_REGISTRY_HELPER_TEXT]}
              fullWidth
              name={key}
              label={capitalizeFirstLetter(key)}
              value={newRegistryEntry[key as keyof LLMRegistryEntry]}
              onChange={handleRegistryEntryInputChange}
            />
            {key == "name" && false ? <Divider role="presentation">If you intend to share publicly, please also fill in...</Divider> : null}
            {key == "license" ? <Divider role="presentation">Advanced Config </Divider> : null}
          </Box>);
        break;
    }
  }


  // {
  //   ["config", "parameters", "sessionParameters"].map((key) =>

  //     key !== "config" && key !== "parameters" && key !== "capabilities" && key !== "backendUuid" && key !== "downloadState" && key !== "sessionParameters" && (typeof newRegistryEntry[key as keyof LLMRegistryEntry] === "boolean" ? (
  //     ): (
  //       ))))
  // }
  // {
  //   ["capabilities"].map((key) =>
  //             }
  // {
  //   ["config", "parameters", "sessionParameters"].map((key) =>
  //             )
  // }
  // }
  const handleLLMFilterChange = (event: any) => {
    setLLMFilter(event.target.value);
  }

  return (
    <Box>
      <Typography variant="h3">Downloadable Large Language Models</Typography>

      <Box sx={{my: 2}}>
        <Button variant="contained" color="primary" onClick={() => setRegistryEntryModalOpen(true)}>
          Add Registry Entry
        </Button>
        <Button variant="contained" color="primary" onClick={() => refreshData(false)}>
          Refresh Data
        </Button>
        <Button variant="outlined" color="primary" onClick={() => setRegistryModalOpen(true)}>
          Add Registry
        </Button>
        <Button variant="outlined" color="primary" onClick={() => refreshData(true)}>
          Force Refresh Remote
        </Button>
      </Box>

      <Box>
        <TextField size="small" label="Filter" onChange={handleLLMFilterChange} value={llmFilter} InputProps={{
          startAdornment: (
            <InputAdornment position="start">
              <SearchIcon />
            </InputAdornment>
          ),
        }}
        />
      </Box>

      {downloadableLLMs.filter((pair, index) => {
        console.log(llmFilter);
        return pair[0].id.includes(llmFilter) || pair[0].name.includes(llmFilter) || pair[0].familyId.includes(llmFilter) || pair[0].license.includes(llmFilter) || pair[0].organization.includes(llmFilter);

      }).map((pair, index) => (
        <LLMDownloadableInfo
          key={pair[0].id}
          llm={pair[0]}
          registry={pair[1]}
          beginDownload={() => {beginDownload(pair[0], pair[1].url)}}
          completeDownload={() => {completeDownload(pair[0].id, pair[1].url)}}
        />
      ))}

      <Modal open={isRegistryModalOpen} onClose={() => setRegistryModalOpen(false)}>
        <ModalBox>
          <Card className="registry-form">
            <CardContent>
              <Typography variant="h5">Add a new Registry</Typography>
              {Object.keys(newRegistry).map((key) =>
                <TextField
                  error={!!newRegistryErrors[key]}
                  helperText={newRegistryErrors[key] ? newRegistryErrors[key] : NEW_REG_HELPER_TEXT[key as keyof typeof NEW_REG_HELPER_TEXT]}
                  fullWidth
                  name={key}
                  label={capitalizeFirstLetter(key)}
                  value={newRegistry[key as keyof typeof newRegistry]}
                  onChange={handleRegistryInputChange}
                />
              )}
              <Button variant="contained" color="primary" onClick={handleAddRegistry}>
                Submit
              </Button>

            </CardContent>
          </Card>
        </ModalBox>
      </Modal>

      <Modal open={isRegistryEntryModalOpen} onClose={() => setRegistryEntryModalOpen(false)}>
        <ModalBox>
          <Card className="registry-entry-form">
            <CardContent>
              <Typography variant="h5">Add a new Registry Entry</Typography>
              <Grid item xs={12}>
                {Object.keys(newRegistryEntry).map((key) => (
                  <Box
                    sx={{
                      padding: 0.5
                    }}>{renderField(key as keyof LLMRegistryEntry)}
                  </Box>
                ))}

              </Grid>

              <Button variant="contained" color="primary" onClick={handleAddRegistryEntry}>
                Submit
              </Button>
            </CardContent>
          </Card>
        </ModalBox>
      </Modal>
    </Box>
  );

}

export default DownloadableLLMs;

