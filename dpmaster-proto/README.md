| message                 | serialize | deserialize |
| ----------------------- | :-------: | :---------: |
| `heartbeat`             | ✓         | ✓         |
| `getinfo`               | ✓         | ✓         |
| `infoResponse`          | ✓         | ✓         |
| `getservers`            | ✓         | ✓         |
| `getserversResponse`    | ✓         | ✓         |
| `getserversExt`         | ❌         | ❌         |
| `getserversExtResponse` | ❌         | ❌         |

```mermaid
sequenceDiagram
    Game server->>Master server: heartbeat
    Master server->>Game server: getinfo
    Note left of Master server: A_ch4Lleng3
    Game server->>Master server: infoResponse
    Note right of Game server: A_ch4Lleng3
```

```mermaid
sequenceDiagram
    Game client->>Master server: getservers
    loop EOT
        Master server->>Game client: getserversResponse
    end
```
