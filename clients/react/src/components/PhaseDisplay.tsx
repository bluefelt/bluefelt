import { useEffect, useState, useRef } from 'react';

interface PhaseMessage {
  track?: string;
  phase?: string;
  action?: string;
  message: string;
  timestamp: number;
}

interface PhaseDisplayProps {
  phaseMessages?: PhaseMessage[];
  phaseStates?: Record<string, { current: string; count: number }>;
}

export default function PhaseDisplay({ phaseMessages, phaseStates }: PhaseDisplayProps) {
  const [displayMessage, setDisplayMessage] = useState<string>('');
  const [messageQueue, setMessageQueue] = useState<PhaseMessage[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const processedTimestamps = useRef(new Set<number>());

  // Add new messages to queue (only ones we haven't seen before)
  useEffect(() => {
    if (phaseMessages && phaseMessages.length > 0) {
      // Filter out messages we've already processed
      const newMessages = phaseMessages.filter(msg => 
        !processedTimestamps.current.has(msg.timestamp)
      );
      
      if (newMessages.length > 0) {
        // Sort by timestamp and add to queue
        const sortedMessages = [...newMessages].sort((a, b) => a.timestamp - b.timestamp);
        setMessageQueue(prev => [...prev, ...sortedMessages]);
        
        // Mark these messages as processed
        newMessages.forEach(msg => processedTimestamps.current.add(msg.timestamp));
      }
    }
  }, [phaseMessages]);

  // Process message queue
  useEffect(() => {
    if (messageQueue.length > 0 && !isProcessing) {
      setIsProcessing(true);
      const [currentMessage, ...rest] = messageQueue;
      
      // Display the message
      setDisplayMessage(currentMessage.message);
      
      // Clear after minimum 1.5 seconds and process next
      setTimeout(() => {
        setDisplayMessage('');
        setMessageQueue(rest);
        setIsProcessing(false);
      }, 1500);
    }
  }, [messageQueue, isProcessing]);

  // Build phase status string
  const getPhaseStatus = () => {
    if (!phaseStates) return '';
    
    const statusParts: string[] = [];
    
    // Get current phases
    Object.entries(phaseStates).forEach(([track, state]) => {
      if (state.current) {
        // Format phase name (e.g., "game.setup" -> "Game Setup")
        const phaseName = state.current
          .split('.')
          .map(part => part.charAt(0).toUpperCase() + part.slice(1))
          .join(' ');
        
        if (track === 'round' && state.count > 0) {
          statusParts.push(`Round ${state.count}`);
        } else if (track === 'turn') {
          // Don't show turn track in status
        } else {
          statusParts.push(phaseName);
        }
      }
    });
    
    return statusParts.join(' • ');
  };

  return (
    <>
      {/* Phase status bar */}
      {phaseStates && (
        <div style={{
          backgroundColor: 'rgba(0, 0, 0, 0.8)',
          border: '1px solid #444',
          padding: '8px 16px',
          margin: '10px',
          fontFamily: 'Roboto Condensed, sans-serif',
          fontSize: '14px',
          color: '#D8B260',
          textAlign: 'center'
        }}>
          {getPhaseStatus()}
        </div>
      )}
      
      {/* Automatic action message overlay */}
      {displayMessage && (
        <div style={{
          position: 'fixed',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          backgroundColor: 'rgba(0, 0, 0, 0.9)',
          border: '2px solid #D8B260',
          borderRadius: '8px',
          padding: '24px 48px',
          fontFamily: 'Josefin Sans, sans-serif',
          fontSize: '24px',
          color: '#D8B260',
          textAlign: 'center',
          zIndex: 1000,
          animation: 'fadeIn 0.3s ease-in-out'
        }}>
          {displayMessage}
        </div>
      )}
      
      <style>{`
        @keyframes fadeIn {
          from { opacity: 0; transform: translate(-50%, -50%) scale(0.9); }
          to { opacity: 1; transform: translate(-50%, -50%) scale(1); }
        }
      `}</style>
    </>
  );
}