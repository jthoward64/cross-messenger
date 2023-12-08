# Cross Messenger

## Legal

First of all, I am not a lawyer, and will not be consulting one for this project. I have tried to stick to [Apple's guidance](https://www.apple.com/legal), but if you find something that is not in compliance, please let me know. If you are Apple and have an issue with this project, please contact me and I will be happy to work with you to resolve it, or take the project down if necessary.

iMessage® is a trademark of Apple Inc., registered in the U.S. and other countries and regions. Apple does not endorse or support this project in any way.

## What is this?

This is a GUI frontend for [rustpush](https://github.com/TaeHagen/rustpush), which is a Rust implementation of the Apple Push Notification Service (APNS) protocol (and iMessage)

## Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/en/download/)
- [Bun](https://bun.sh)
- [Python 3](https://www.python.org/downloads/)
- [pip](https://pip.pypa.io/en/stable/installing/)

To run the project you need Rust and cargo set up. After that you can use `bun install` to install the front-end dependencies. Then make sure you are in a Python environment with the packages `requests` and `unicorn`. Now you can run `cargo tauri dev` to start the project.
