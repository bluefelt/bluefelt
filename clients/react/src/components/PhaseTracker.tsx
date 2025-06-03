interface PhaseState {
  current: string;
  count: number;
  actionsProcessed: number;
}

interface PhaseTrackerProps {
  phaseStates?: Record<string, PhaseState>;
}

export default function PhaseTracker({ phaseStates }: PhaseTrackerProps) {
  if (!phaseStates) return null;

  // Sort phase sets for consistent display order
  const sortedPhaseSets = Object.entries(phaseStates).sort(([a], [b]) => {
    // Define a preferred order
    const order = ['game', 'round', 'turn', 'turnPhase', 'playerTurn'];
    const aIndex = order.indexOf(a);
    const bIndex = order.indexOf(b);
    
    if (aIndex !== -1 && bIndex !== -1) {
      return aIndex - bIndex;
    }
    if (aIndex !== -1) return -1;
    if (bIndex !== -1) return 1;
    return a.localeCompare(b);
  });

  return (
    <div style={{
      backgroundColor: 'rgba(0, 0, 0, 0.1)',
      border: '1px solid #ddd',
      borderRadius: '4px',
      padding: '12px',
      margin: '10px',
      fontFamily: 'monospace',
      fontSize: '13px',
    }}>
      <div style={{ 
        fontWeight: 'bold', 
        marginBottom: '8px',
        color: '#666'
      }}>
        Phase Tracker (Debug)
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
        {sortedPhaseSets.map(([setName, state]) => (
          <div key={setName} style={{ 
            display: 'flex',
            alignItems: 'center',
            gap: '8px'
          }}>
            <span style={{ 
              fontWeight: 'bold',
              color: '#333',
              minWidth: '80px'
            }}>
              {setName}:
            </span>
            <span style={{ 
              color: state.current === 'null' ? '#999' : '#0066cc'
            }}>
              {setName}.{state.current}
            </span>
            {state.count > 0 && (
              <span style={{ 
                fontSize: '11px', 
                color: '#666',
                marginLeft: '8px'
              }}>
                (iteration: {state.count})
              </span>
            )}
            {state.actionsProcessed > 0 && (
              <span style={{ 
                fontSize: '11px', 
                color: '#666'
              }}>
                [actions: {state.actionsProcessed}]
              </span>
            )}
          </div>
        ))}
      </div>
      <div style={{ 
        marginTop: '8px',
        paddingTop: '8px',
        borderTop: '1px solid #eee',
        fontSize: '12px',
        color: '#999'
      }}>
        Current phases: {sortedPhaseSets
          .map(([set, state]) => `${set}.${state.current}`)
          .join(' - ')}
      </div>
    </div>
  );
}