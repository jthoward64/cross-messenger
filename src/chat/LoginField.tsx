import { invoke } from "@tauri-apps/api/tauri";
import { useState } from "preact/hooks";

async function login(
  username: string,
  password: string,
  twoFactorCode?: string
) {
  invoke("authenticate", {
    username,
    password,
    code: twoFactorCode || undefined,
  })
    .then((res) => {
      console.log(res);
    })
    .catch((err) => {
      console.error(err);
    });
}

export function LoginField() {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [twoFactorCode, setTwoFactorCode] = useState("");

  return (
    <form
      class="row"
      onSubmit={(e) => {
        e.preventDefault();
        login(username, password, twoFactorCode);
      }}
    >
      <input
        id="username-input"
        onInput={(e) => setUsername(e.currentTarget.value)}
        placeholder="Enter a username..."
      />
      <input
        id="password-input"
        onInput={(e) => setPassword(e.currentTarget.value)}
        placeholder="Enter a password..."
      />
      <input
        id="two-factor-code-input"
        onInput={(e) => setTwoFactorCode(e.currentTarget.value)}
        placeholder="Enter a two-factor code..."
      />
      <button type="submit">Login</button>
    </form>
  );
}
