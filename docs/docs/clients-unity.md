# Official Client: Unity

The Unity client will be Bluefelt's 3D and VR/AR implementation, bringing turn-based games into immersive environments. While still in planning stages, this document outlines the vision and technical approach.

## Status: Planned

The Unity client is not yet in development. This document captures design decisions and requirements for future implementation.

## Vision

The Unity client will demonstrate how Bluefelt's presentation-separated architecture enables radically different visual experiences while maintaining perfect game logic compatibility. Players in VR will compete seamlessly with players on web or mobile.

## Planned Features

### Multi-Platform Support

- **Desktop** - Traditional 3D view with mouse/keyboard
- **VR** - Full immersion with hand tracking (Quest, SteamVR)
- **AR** - Tabletop projection on real surfaces (ARCore, ARKit)
- **Console** - Gamepad-optimized interfaces

### 3D Presentation Metaphors

Based on the entity representation research in our design philosophy:

**Spatial Tokens**
- Physical pieces players can grab and move
- Rotating tokens for hidden information reveals
- Stackable pieces with physics simulation

**Holographic Displays**
- Floating information panels for complex game state
- Gaze-activated detail views
- Spatial organization of game zones

**Environmental Integration**
- Game boards as 3D environments (not just flat boards)
- Atmospheric effects matching game theme
- Dynamic lighting for turn indicators

### VR-Specific Interactions

**Natural Gestures**
- Grab and place pieces with hand tracking
- Flick cards to play them
- Point to select from choices
- Physical dice rolling

**Comfort Features**
- Teleport movement around large boards
- Seated and standing play modes
- Comfort vignetting options
- Smooth vs snap turning

**Social Presence**
- Avatar hands for other players
- Spatial voice chat
- Emote system
- Spectator areas

## Technical Architecture

### Core Systems

```csharp
// WebSocket client matching server protocol
public class BluefeltClient : MonoBehaviour {
    private WebSocket websocket;
    private GameState currentState;
    
    async void Connect(string lobbyId, string playerName) {
        string url = $"ws://{serverUrl}/api/lobbies/{lobbyId}/ws?player={playerName}&join=true";
        websocket = new WebSocket(url);
        
        websocket.OnMessage += HandleMessage;
        await websocket.Connect();
    }
    
    void HandleMessage(byte[] data) {
        var message = JsonUtility.FromJson<ServerMessage>(data);
        
        switch (message.type) {
            case "welcome":
                InitializeGame(message.state);
                break;
            case "patches":
                ApplyPatches(message.patches);
                break;
        }
    }
}
```

### Zone Rendering System

```csharp
// Abstract zone renderer
public abstract class ZoneRenderer : MonoBehaviour {
    public abstract void RenderZone(string zoneId, ZoneData data);
    public abstract void UpdateActionMap(Dictionary<string, ActionInfo> actions);
}

// Grid zone for board games
public class GridZoneRenderer : ZoneRenderer {
    public GameObject cellPrefab;
    public float cellSpacing = 0.1f;
    
    public override void RenderZone(string zoneId, ZoneData data) {
        var grid = data as GridZoneData;
        
        for (int row = 0; row < grid.rows; row++) {
            for (int col = 0; col < grid.cols; col++) {
                var position = new Vector3(col * cellSpacing, 0, row * cellSpacing);
                var cell = Instantiate(cellPrefab, position, Quaternion.identity);
                
                // Configure cell based on content
                var entity = grid.cells[row][col];
                if (entity != null) {
                    RenderEntity(cell, entity);
                }
            }
        }
    }
}
```

### VR Interaction System

```csharp
// Hand controller for VR interactions
public class VRHandController : MonoBehaviour {
    public XRController controller;
    private GameObject hoveredObject;
    private GameObject grabbedObject;
    
    void Update() {
        // Cast ray from hand
        if (Physics.Raycast(transform.position, transform.forward, out RaycastHit hit)) {
            var interactive = hit.collider.GetComponent<InteractiveElement>();
            
            if (interactive != null) {
                SetHovered(interactive.gameObject);
                
                if (controller.selectAction.triggered) {
                    HandleSelect(interactive);
                }
            }
        }
        
        // Handle grab/release
        if (controller.gripAction.triggered && hoveredObject != null) {
            grabbedObject = hoveredObject;
            grabbedObject.transform.SetParent(transform);
        }
        
        if (controller.gripAction.released && grabbedObject != null) {
            HandleDrop();
        }
    }
}
```

## Design Principles

### 1. Spatial Advantage

Leverage 3D space meaningfully:
- **Height** for information hierarchy  
- **Depth** for grouping related elements
- **Animation** for state changes
- **Particle effects** for feedback

### 2. Comfort First

VR comfort is non-negotiable:
- Stable horizon reference
- Minimal forced camera movement  
- Clear visual anchors
- Adjustable play area scale

### 3. Cross-Platform Play

Ensure fair play across platforms:
- No VR-exclusive information
- Equivalent interaction speed
- Synchronized animations
- Platform-agnostic game flow

### 4. Accessibility

Support diverse player needs:
- Colorblind modes
- Subtitle options
- One-handed play modes
- Height adjustment
- Motion sickness reduction

## Entity Representation Examples

### Card Games in VR

Traditional 2D cards become:
- **Floating holograms** - Hover in space at comfortable viewing angle
- **Physical cards** - Hold in hand, flip naturally
- **3D tokens** - Chips and coins with weight and physics

### Board Games in VR

Flat boards transform into:
- **Miniature worlds** - Pieces are characters in environments
- **Holographic tables** - Float at perfect height, scale to preference  
- **Room-scale arenas** - Walk around massive game boards

### Abstract Games

Pure information displays:
- **Data sculptures** - 3D visualization of game state
- **Particle systems** - Resources flow between players
- **Spatial graphs** - Connections and relationships in 3D

## Implementation Roadmap

### Phase 1: Foundation
- [ ] Basic WebSocket client
- [ ] State synchronization  
- [ ] Simple 3D board rendering
- [ ] Mouse interaction

### Phase 2: VR Support
- [ ] VR camera rig
- [ ] Hand controllers
- [ ] Basic grab/place mechanics
- [ ] Comfort options

### Phase 3: Polish
- [ ] Multiple board environments
- [ ] Avatar system
- [ ] Spatial audio
- [ ] Particle effects

### Phase 4: Advanced Features
- [ ] AR mode
- [ ] Hand tracking
- [ ] Voice chat
- [ ] Spectator system

## Development Guidelines

### Performance Targets

- **90 FPS** - Essential for VR comfort
- **< 20ms latency** - Controller to visual feedback
- **Mobile VR** - Optimize for Quest 2 baseline
- **LOD system** - Scale detail by platform

### Best Practices

1. **Pool objects** - Avoid instantiation during gameplay
2. **Batch draws** - Combine meshes where possible
3. **Optimize shaders** - Simple shaders for VR
4. **Profile constantly** - Monitor frame timing

### Testing Requirements

- **Hardware variety** - Test on multiple VR headsets
- **Comfort testing** - Extended play sessions
- **Cross-platform** - Verify parity with web client
- **Accessibility** - Test all comfort options

## Future Possibilities

### AI Integration
- Natural language commands
- AI-assisted tutorials
- Adaptive difficulty

### Social Features
- Persistent avatars
- Friend lobbies
- Tournament spaces
- Streaming integration

### Content Creation
- Board editor in VR
- Custom piece designer
- Environment themes
- Share creations

## Getting Involved

While the Unity client isn't yet in development, you can:

1. **Share Ideas** - What would make VR board gaming amazing?
2. **Prototype** - Try building a proof of concept
3. **Research** - Study VR interaction patterns
4. **Plan** - Help design the architecture

The Unity client will showcase how Bluefelt's architecture enables innovation in game presentation while maintaining the integrity of game logic. When it launches, players will experience their favorite games in entirely new ways.