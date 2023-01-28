# Photographic Memory Search
Photographic Memory Search is a tool for saving and recalling **everything** that happens on your computer screens.

## Overview
### Components
There are three components to PMS: the *Server*, the *Client* and the *Web Interface*.

#### Server

#### Client

#### Web Interface

## Running PMS
1. Start the server: `cargo run --bin screenlog-server --release`
2. Start the client: `cargo run --bin screenlog-client --release`

When you want to search through your library, start the web interface `cd web && trunk serve`. By default the interface is served at `localhost:8080`.