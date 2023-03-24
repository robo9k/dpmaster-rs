# Implementation status

| message                 | serialize | deserialize |
| ----------------------- | :-------: | :---------: |
| `heartbeat`             | ✓         | ✓         |
| `getinfo`               | ✓         | ✓         |
| `infoResponse`          | ✓         | ✓         |
| `getservers`            | ✓         | ✓         |
| `getserversResponse`    | ✓         | ✓         |
| `getserversExt`         | ❌         | ❌         |
| `getserversExtResponse` | ❌         | ❌         |

## Message flows

Lorem ipsum.

### `heartbeat` message flow

❶ Game server sends `heartbeat` message to master server

❷ Master server sends `getinfo` message with new challenge back to game server

❸ Game server sends `infoResponse` message with same challenge back to master server
```mermaid
sequenceDiagram
    autonumber
    Game server->>Master server: heartbeat
    Master server->>Game server: getinfo
    Note left of Master server: A_ch4Lleng3
    Game server->>Master server: infoResponse
    Note right of Game server: A_ch4Lleng3
```

### `getservers` message flow

❶ Game client sends `getservers` message to master server

❷ Master server sends `getserversResponse` message(s) back to game client
```mermaid
sequenceDiagram
    autonumber
    Game client->>Master server: getservers
    loop EOT
        Master server->>Game client: getserversResponse
    end
```
