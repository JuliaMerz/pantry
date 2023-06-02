// src/pages/DownloadableLLMs.tsx

import React, { useEffect, useState } from 'react';
import { Store } from "tauri-plugin-store-api";
import { fetch } from '@tauri-apps/api/http';
import { LLMDownloadable, LLMRegistry, LLMRegistryEntry } from '../interfaces';
import LLMOnlineInfo from '../components/LLMOnlineInfo';

const LLM_INFO_SOURCE = "https://raw.githubusercontent.com/JuliaMerz/pantry/master/models/index.json";

function DownloadableLLMs() {
  const [downloadableLLMs, setDownloadableLLMs] = useState<LLMDownloadable[]>([]);
  const [registries, setRegistries] = useState<any>([]);


  const store = new Store(".settings.dat");
  useEffect(() => {

    store.get("registries").
      then((registries) => {
        if (!registries) {
          // would do setRegistries([]) but it's the default
          addRegistry('default', LLM_INFO_SOURCE);
        } else
          setRegistries(registries);
    });
  });

  // Function to append a new registry to the list of registries
  async function addRegistry(id: string, url: string) {
    const newRegistries = registries.concat([{ id, url }]);
    await store.set("registries", registries);
    await store.save();
    setRegistries(newRegistries)
  }

  // Function to build an object of registry entries
  async function buildRegistryEntries() {
    const registries:LLMRegistry[] | null= await store.get("registries");
    if (registries == undefined)
      return //this won't happen
    const registryEntries = {};
    for (const { id, url } of registries) {
      const response = await fetch(url);
      const data = await response.json();
      registryEntries[id] = data;
    }
    await store.set("registryEntries", registryEntries);
    await store.save();
  }


  useEffect(() => {
    const fetchDownloadableLLMs = async () => {
      try {
        const response = await fetch(LLM_INFO_SOURCE);
        console.log(response);
        const data = await response.data
        setDownloadableLLMs((data as any).models);
      } catch (err) {
        console.error(err);
      }
    };

    fetchDownloadableLLMs();
  }, []);

  return (
    <div>
      <h1>Downloadable Large Language Models</h1>
      {downloadableLLMs.map((llm) => (
        <LLMOnlineInfo key={llm.id} {...llm} />
      ))}
    </div>
  );
}

export default DownloadableLLMs;

