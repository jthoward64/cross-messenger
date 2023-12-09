import "./App.css";
import { ChatWindow } from "./chat/ChatWindow";
import { LoginField } from "./chat/LoginField";

function App() {
  return (
    <div class="container">
      <h1>Welcome to Cross Messenger!</h1>

      <LoginField />
      <ChatWindow />
    </div>
  );
}

export default App;
