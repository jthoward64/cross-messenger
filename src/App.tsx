import { invoke } from "@tauri-apps/api/tauri";
import { useState } from "preact/hooks";
import "./App.css";
import preactLogo from "./assets/preact.svg";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [data, setData] = useState<any>(undefined);

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setGreetMsg(await invoke("greet", { name }));
  }

  function getData() {
    invoke("get_validation_data")
      .then((res) => {
        setData(res);
      })
      .catch((err) => {
        console.error(err);
      });
  }

  return (
    <div class="container">
      <h1>Welcome to Tauri!</h1>

      <div class="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" class="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" class="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://preactjs.com" target="_blank">
          <img src={preactLogo} class="logo preact" alt="Preact logo" />
        </a>
      </div>

      <p>Click on the Tauri, Vite, and Preact logos to learn more.</p>

      <form
        class="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onInput={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>

      <button onClick={getData}>Get Data</button>

      <p>{JSON.stringify(data)}</p>

      <p>{greetMsg}</p>
    </div>
  );
}

export default App;
