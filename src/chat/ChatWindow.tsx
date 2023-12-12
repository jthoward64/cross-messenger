import { invoke } from "@tauri-apps/api/primitives";
import { useState } from "preact/hooks";

function sendMessage(message: string, to: string) {
  invoke("send_message", { message, to })
    .then((res) => {
      console.log(res);
    })
    .catch((err) => {
      console.error(err);
    });
}

export function ChatWindow() {
  const [message, setMessage] = useState("");
  const [to, setTo] = useState("");

  return (
    <form
      class="row"
      onSubmit={(e) => {
        e.preventDefault();
        sendMessage(message, to);
      }}
    >
      <input
        id="message-input"
        onInput={(e) => setMessage(e.currentTarget.value)}
        placeholder="Enter a message..."
      />
      <input
        id="to-input"
        onInput={(e) => setTo(e.currentTarget.value)}
        placeholder="Enter a recipient..."
      />
      <button type="submit">Send</button>
    </form>
  );
}
