# Developing Games for Bluefelt
Games are implemented as a folder inside the `games/` directory. Each major version of the game is stored in a separate folder.

Example: 
```
love-letter/
├─ 1.0/
|  ├─ manifest.yaml 
|  ├─ entities.yaml
|  ├─ src/
|  ├─ script.wasm
|  └─ assets/
├─ 1.1/
|  ├─ manifest.yaml 
|  ├─ entities.yaml
|  ├─ src/
|  ├─ script.wasm
|  └─ assets/
└─ changelog.md
```
There are a number of required files.
## `manifest.yaml`
The `manifest.yaml` file defines the version of the game, its compatibility with the Bluefelt spec, and metadata about the game.

Example:
```yaml
gameId:  "alchemists"
version: "1.0.0"
bluefeltSpecVersion: "1"
metadata:
  name: "Alchemists"
  designer: "Matúš Kotry"
  players: { min: 2, max: 4 }
  description: "Showcase your alchemical know-how by publishing theories."

hash: "sha256-<filled-by-cli>"
```
## `entities.yaml`
The `entities.yaml` file describes the bulk of the game rules. It consists of a number of major sections.

### entities
Entities are the "nouns" of the game.
```yaml
entities:
  guard:
    kind: card
    value: 1
    count: 5
    hook: guard_guess
    ui:
      name: "Guard"
      help: "Guess another player's card. If correct they are eliminated."
  priest:   { kind: card, value: 2, count: 2, hook: priest_peek }
  baron:    { kind: card, value: 3, count: 2, hook: baron_compare }
  handmaid: { kind: card, value: 4, count: 2, hook: handmaid_shield }
  prince:   { kind: card, value: 5, count: 2, hook: prince_discard }
  king:     { kind: card, value: 6, count: 1, hook: king_swap }
  countess: { kind: card, value: 7, count: 1, hook: none }
  princess: { kind: card, value: 8, count: 1, hook: princess_lose }
```
### zones
Zones are any deck, pile, or stack of entities. 
```yaml
zones:
  deck:       { shape: stack, visibility: none }
  burn:       { shape: stack, visibility: none }
  discard:    { shape: stack, visibility: all }
  hands:      { shape: list,  visibility: owner, perPlayer: true }
  immunes:    { shape: flag,  visibility: all, perPlayer: true }
  eliminated: { shape: flag,  visibility: all, perPlayer: true }
  ```
### verbs
Verbs are actions that a player can take in the game.
```yaml
verbs:
  draw:
    pre: [{ zoneNotEmpty: { zone: deck } }]
    effect: [{ move: { from: deck, to: hands, count: 1, playerSlot: actor } }]
    ui: 
      prompt: "Draw a card from the deck"
      picker: "hand"

  play:
    params: { cardId: Id }
    pre:
      - holds: { player: actor, zone: hands, entity: $cardId }
      - notImmune: { player: actor }
    effect:
      - move: { from: hands, to: discard, count: 1,
                entity: $cardId, playerSlot: actor }
    nextPhase: resolve
    ui:
      prompt: "Select one card from your hand to play"
      picker: "hand"

  chooseTarget:
    params: { target: PlayerId }
    pre:
      - differentPlayer: { target: $target }
      - stillInRound: { player: $target }
    effect: []
    ui:
      prompt: "Choose an opponent to target with {{cardName}}"
      paramPrompts:
        target: "Target player"
      picker: "playerList"
```
For each verb, an array of `pre` conditions are defined. These define requirements for a particular verb to be possible. During a player's turn, the client will show affordances for any verb that is possible for the current game state.

A thin React hook can fetch ui.prompt + replace {{cardName}} placeholders from bundle metadata. If the ui: block is missing, you fall back on generic text (“Pick a card”, “Pick a player”).

### phases
```yaml
phases:
  - id: turn
    activePlayer: sequential
    verbs: [draw, play, chooseTarget]

  - id: resolve
    verbs: []
```
### setup
```yaml
setup:
  - shuffle: { zone: deck }
  - move: { from: deck, to: burn, count: 1 }
  - repeat:
      times: 2
      actions:
        - forEachPlayer:
            - draw
  - setTurn: { player: random }
```
### hooks
Hooks are more complex actions that cannot be expressed in yaml. These hooks are method names that are defined in the `src/` folder.
```yaml
hooks:
  guard_guess:     on_after_play
  priest_peek:     on_after_play
  baron_compare:   on_after_play
  handmaid_shield: on_after_play
  prince_discard:  on_after_play
  king_swap:       on_after_play
  princess_lose:   on_after_play
  win_check:       on_phase_end
```

Depending on a developer's preference, they may want to implement even simple actions that could be described in yaml as hooks.

Note: Maybe we need some prefix for hooks when they are referred to inside yaml.
## Hooks (WebAssembly)
All but the most simple game will require some advanced scripting to fully define the game behavior.

Hooks can be written in any language the developer chooses which is capable of being compiled to WebAssembly. Some popular examples include:
- JavaScript
- Python
- Go
- Rust

Instructions about this process goes here.
```rust
use bluefelt_sdk::{host, JsonValue};

/// Guard: guess a card in target’s hand, eliminate if correct
///
/// JSON args: { "target":"p2", "guess":"priest" }
#[no_mangle]
pub extern "C" fn guard_guess(ptr: u32, len: u32) {
    let j: JsonValue = host::read_json(ptr, len);
    let target = j["target"].as_str().unwrap();
    let guess  = j["guess"].as_str().unwrap();

    if host::player_holds(target, guess) {
        host::emit(host::eliminate(target));
    }
}
```

## Client-Server Communication
Below is a typical example of how the client and server work together.

Client joins lobby, requests game state
```
GET /lobbies/9f42b913/ws
Sec-WebSocket-Protocol: bluefelt.v0
```
Server sends "welcome" payload
```json
{
  "type": "welcome",
  "bundleMeta": {
    "gameId": "love-letter",
    "version": "1.0.0",
    "cards": {
      "guard":   { "name": "Guard",   "value":1, "asset":"guard.webp" },
      "priest":  { "name": "Priest",  "value":2, "asset":"priest.webp" },
      ...
    }
  },
  "initialState": {
    "zones": {
      "deck":    ["§hash1","§hash2", ...],
      "burn":    ["§hashX"],
      "discard": [],
      "hands": {
        "p1": ["§hashA"],
        "p2": ["§hashB"]
      },
      "immunes": { "p1": false, "p2": false },
      "eliminated": { "p1": false, "p2": false }
    },
    "turn": { "player":"p1", "phase":"turn", "tick":1 }
  },
  "you": "p1"
}
```
Client hydrates its internal state with `initialState`.

Client takes some action.
```json
{ "verb":"draw", "args":{} }    
```
Server validates that action is legal, applies effect, then broadcasts event envelope to everyone:
```json
{
  "type": "event",
  "tick": 2,
  "actor": "p1",
  "verb": "draw",
  "args": {},
  "diff": [
    { "op":"remove","path":"/zones/deck/0" },
    { "op":"add", "path":"/zones/hands/p1/-", "value":"§hash1" }
  ]
}
```
Note: Need to prevent users from inspecting diffs and understand which card a different player drew by recognizing the hash.

All clients patch local state with the JSON-Patch diff.

Client takes another action.
```json
// player plays Guard
{ "verb":"play",
     "args": { "cardId":"§hashA" }
}

// guard_guess hook
{ "verb":"chooseTarget",
  "args": { "target":"p2", "guess":"priest" }
}
```

Server merges both envelopes into the same tick
```json lines
{
  "type":"event",
  "tick":3,
  "actor":"p1",
  "verb":"play",
  "args":{ "cardId":"§hashA" },
  "diff":[
    { "op":"remove", "path":"/zones/hands/p1/0" },
    { "op":"add", "path":"/zones/discard/-", "value":"§hashA" }
  ]
}

{
  "type":"event",
  "tick":3,
  "actor":"p1",
  "verb":"chooseTarget",
  "args":{ "target":"p2", "guess":"priest" },
  "diff":[]
}
```
Above shows guard guess being wrong, so no diff value. If the guess was right, the diff would contain `{"op":"replace","path":"/zones/eliminated/p2","value":true}`

Sever rotates turn
```json
{
  "type":"event",
  "t":4,
  "actor":"system",
  "verb":"endPhase",
  "args":{},
  "diff":[
    { "op":"replace", "path":"/turn/player", "value":"p2" },
    { "op":"replace", "path":"/turn/phase",  "value":"turn" }
  ]
}
```
The win_check hook fires before sending this. This result indicates that the win state has not been reached and the game continues.