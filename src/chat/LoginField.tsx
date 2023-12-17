import { useEffect, useState } from "preact/hooks";
import { getUser, login } from "../ipc";

export function LoginField() {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [twoFactorCode, setTwoFactorCode] = useState("");

  const [selectedHandle, setSelectedHandle] = useState("");

  useEffect(() => {
    setInterval(() => {
      getUser()
        .then((user) => {
          if (user.tag === "ok") {
            setSelectedHandle(user.val?.selectedHandle ?? "");
          } else {
            console.error(user.val);
            setSelectedHandle("");
          }
        })
        .catch((err) => {
          console.error(err);
          setSelectedHandle("");
        });
    }, 1000);
  }, []);

  return (
    <form
      class="row"
      onSubmit={(e) => {
        e.preventDefault();
        login(username, password, twoFactorCode || null);
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
      <p>
        {selectedHandle ? `Logged in as ${selectedHandle}` : "Not logged in"}
      </p>
    </form>
  );
}
