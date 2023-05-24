import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import { NavLink } from 'react-router-dom';

import { BrowserRouter as Router, Route, Routes } from "react-router-dom";
import Home from "./pages/Home";
import History from "./pages/History";
import AvailableLLMs from "./pages/AvailableLLMs";
import DownloadLLMs from "./pages/DownloadableLLMs";
import Requests from "./pages/Requests";
import Settings from "./pages/Settings";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setGreetMsg(await invoke("greet", { name }));
  }
  // src/App.tsx


  return (
    <div className="App">
    <Router>
      <header>
        <nav>
          <ul>
            <li><NavLink to="/" >Home</NavLink></li>
            <li><NavLink to="/available-llms" >Available LLMs</NavLink></li>
            <li><NavLink to="/download-llms" >Download LLMs</NavLink></li>
            <li><NavLink to="/requests" >Requests</NavLink></li>
            <li><NavLink to="/settings" >Settings</NavLink></li>
          </ul>
        </nav>
      </header>

      <main>
      <Routes>
      <Route path="/" element={<Home />} />
      <Route path="/history/:id" element={<History />} />
      <Route path="/available-llms" element={<AvailableLLMs />} />
      <Route path="/download-llms" element={<DownloadLLMs />} />
      <Route path="/requests" element={<Requests />} />
      <Route path="/settings" element={<Settings />} />
      </Routes>
      </main>
    </Router>
    </div>
  );
}

export default App;


