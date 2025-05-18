# Bluefelt v0.1
RFC 0001
## Scope and Goals
This defines the minimum viable grammar that can express a hidden-information card game with a single sequential turn order.

This spec describes a three-part contract:
1. Bundle contract - A fixed directory structure with `manifest.yaml`, `entities.yaml`, optional `script.wasm`.
2. Runtime contract - deterministic host API + event envelope.
3. Version contract - semantic guarantees allowing old matches to finish unbroken.
### Out of Scope
- Real-time dexterity
- Simultaneous phases
- Programmable powers
- Custom UI hints
## Bundle layout
- my-game
  - manifest.yml
  - entities.yml
  - script.wasm
  - assets/...
## manifest.yml
```yaml
gameId: love-letter
version: 1.0.0
specVersion: 0.1
metadata:
  name: "Love Letter"
  author: "Seiji Kanai"
  players: { min: 2, max: 4 }
  description: "Win the princess's trust..."
hash: "sha256-ab12...ff" #calculated over remaining files
```
The hash anchors live matches to this exact bundle.
## Core concepts
|Term| Definition                                                          |
---|---------------------------------------------------------------------
|Entity| Atom with properties (`id`, optional `value`, arbitrary key/vals.)  |
|Zone| Container holding an ordered or unordered set of entity references. |
|Verb| Atomic state transition exposed to clients and/or scripts.          |
|Phase| Ordered collection of verbs that forms a turn.                      |
|Hook| WASM function subscribed to a lifecycle event; returns diff(s).     |
|Visbility| Static rule controlling who may query a zone or entity property.    |
## entities.yml
### schema excerpt
```yaml
# Top-level keys
entities:    # Dict<string, EntityTemplate>
zones:       # Dict<string, ZoneTemplate>
verbs:       # Dict<string, VerbTemplates>
phases:      # Array<PhaseTemplate>
setup:       # Array<SetupStep>, executed once per match
hooks:       # Dict<EventName, FnId>, pointer into script.wasm
```
### Entity template
```yaml
card_guard:
  kind: card
  value: 1
  tags: ["eliminate"]
```

### Zone template
```yaml
deck:
  shape: stack    # enum: stack | queue | bag | list
  visibility: none # none | owner | all | topPublic
  mutable: true # false -> cards can't leave (e.g., discard log)
```
Effects MUST be purely declarative; calculations happen in hooks.
### Verb template
```yaml
id: turn
activePlayer: sequential # enum: sequential | simultaneous
verbs: [draw, choose, commit, reveal, score]

```

## Illustrative Love Letter snippets
```yaml
# entities.yaml (partial)
verbs:
  guard_guess:
    params: { targetPlayer: Id, cardId: Id }
    pre:
      - playersAlive: { countMin: 2 }
      - targetNotSelf: { target: $targetPlayer }
    effect:
      - eliminateIf: { player: $targetPlayer, holds: $cardId }
hooks:
  eliminateIf: on_after_effect     #implemented in script.wasm
```
```wat
;; TinyGo/AssemblyScript compiled to WASM
;; Pseudocode: eliminateIf hook
func eliminateIf(ptr i32, len i32( {
   let json = host.readJson(ptr,len);
   if (playerHolds(json.player, json.hold)) {
    host.emit(eliminate(json.player));
   }
}
```