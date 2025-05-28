import { usePlayer } from '../context/PlayerContext';
import { getPlayerColor } from '../config/colors';
import { getPlayerEntity, getEntityDisplay } from '../utils/entityUtils';
import Breadcrumbs from './Breadcrumbs';

interface GameHeaderProps {
  lobbyId: string;
  gameId: string;
  gameName: string;
  status: 'waiting' | 'in_progress' | 'finished';
  players: { username: string; isConnected: boolean }[];
  currentPlayer?: string;
  entityDefinitions?: any[];
  turnPrompt?: string;
}

export default function GameHeader({ gameId, gameName, status, players, currentPlayer, entityDefinitions, turnPrompt }: GameHeaderProps) {
  const { player } = usePlayer();

  const getHeaderPlayerColor = (username: string) => {
    if (!player) return '#888';
    
    const myPlayerIndex = players.findIndex(p => p.username === player.username);
    const targetPlayerIndex = players.findIndex(p => p.username === username);
    
    if (myPlayerIndex === -1 || targetPlayerIndex === -1) return '#888';
    
    const color = getPlayerColor(targetPlayerIndex, player.color, myPlayerIndex);
    return color.hex;
  };


  return (
    <div>
      <Breadcrumbs items={[
        { label: 'Lobbies', href: '/lobbies' },
        { label: gameName }
      ]} />
      <div style={styles.gameHeader}>
        <div style={styles.gameNameAndId}>
          <h1 style={styles.gameTitle}>
            {gameName}
          </h1>
          <span style={styles.gameId}>#{gameId}</span>
        </div>
        <div style={styles.gameStatusContainer}>
          <span style={{
            ...styles.gameStatus,
            ...(status === 'in_progress'
              ? styles.gameStatus_inProgress
              : status === 'finished'
              ? styles.gameStatus_finished
              : styles.gameStatus_waiting)
          }} />
          <span>
            {status === 'in_progress' ? 'In Progress' : status === 'finished' ? 'Finished' : 'Waiting'}
          </span>
        </div>

      </div>
      <div className="flex items-center space-x-6 mt-4" style={{margin: "10px"}}>
        {players.map((p, i) => {
          const playerColor = getHeaderPlayerColor(p.username);
          const isCurrentTurn = currentPlayer === p.username;
          const playerIndex = i;
          const playerNum = playerIndex + 1;
          
          // Get the player's representative entity and display info
          const playerEntity = getPlayerEntity(entityDefinitions, playerNum);
          const entityDisplay = getEntityDisplay(playerEntity, playerNum);
          
          // Debug logging
          console.log(`Player ${playerNum} (${p.username}):`, {
            playerEntity,
            entityDisplay,
            entityDefinitions
          });
          
          return (
            <div key={p.username} className="flex items-center space-x-2 relative">
              {/* Color box - filled if current turn, outline if not */}
              <div
                style={{ 
                  backgroundColor: isCurrentTurn ? playerColor : 'transparent',
                  borderColor: playerColor,
                  height: '12px',
                  width: '12px',
                  borderWidth: '2px',
                }}
              />
              <span 
                className={`text-sm font-bold ${!p.isConnected ? 'opacity-50' : ''}`}
                style={{ color: playerColor, fontFamily: 'Josefin Sans, sans-serif', fontSize: '14pt' }}
              >
                {p.username}
              </span>
              {/* Speech bubble with player mark */}
              <div 
                className="relative ml-1"
                style={{
                  backgroundImage: 'url(/cute_bubble.svg)',
                  backgroundSize: 'contain',
                  backgroundRepeat: 'no-repeat',
                  backgroundPosition: 'center',
                  position: 'relative',
                  top: '-16px',
                  left: '-10px',
                  width: '32px',
                  height: '32px',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  fontSize: '14px',
                  fontWeight: 'bold',
                  color: playerColor
                }}
              >
                {entityDisplay.type === 'token' && entityDisplay.value ? (
                  <div 
                    style={{ 
                      width: '14px',
                      height: '14px',
                      backgroundColor: playerColor,
                      maskImage: `url(/tokens/token_${entityDisplay.value}.svg)`,
                      maskSize: 'contain',
                      maskRepeat: 'no-repeat',
                      maskPosition: 'center',
                      WebkitMaskImage: `url(/tokens/token_${entityDisplay.value}.svg)`,
                      WebkitMaskSize: 'contain',
                      WebkitMaskRepeat: 'no-repeat',
                      WebkitMaskPosition: 'center',
                      marginLeft: '3px',
                      marginBottom: '6px' // Adjust for bubble tail
                    }}
                  />
                ) : (
                  <span style={{ marginBottom: '2px' }}>{entityDisplay.text}</span>
                )}
              </div>
            </div>
          );
        })}
      </div>
      {/* Turn indicator banner */}
      {currentPlayer && status === 'in_progress' && (
        <div className="mt-6 relative">
          <div 
            className="px-8 py-4 flex items-center"
            style={{
              backgroundColor: player?.username === currentPlayer ? '#D8B260' : '#727272',
              color: player?.username === currentPlayer ? 'black' : 'white',
              fontFamily: 'Roboto Condensed, sans-serif',
              borderRadius: '0',
              alignItems: 'baseline',
              gap: '0.5rem',
            }}
          >
            <span className="text-xl font-bold uppercase tracking-wide">
              {player?.username === currentPlayer ? 'YOUR TURN' : `${currentPlayer.toUpperCase()}'S TURN`}
            </span>
            <span className="text-base">
              {turnPrompt || 'Make your move'}
            </span>
          </div>
        </div>
      )}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  gameHeader: {
    display: "flex",
    alignItems: "baseline",
    justifyContent: "space-between",
    backgroundColor: "black",
    border: "2px solid #D8B260",
    margin: "10px",
    padding: "10px 12px 0 12px",
  },
  gameTitle: {
    fontFamily: "Josefin Sans, sans-serif",
    fontSize: "24pt",
  },
  gameNameAndId: {
    display: "flex",
    flexDirection: "row",
    alignItems: "baseline",
    gap: "0.5em",
  },
  gameId: {
    fontFamily: "Roboto Condensed, sans-serif",
    fontWeight: "bold",
    color: "#9AA5B3",
    textTransform: "lowercase",
  },
  gameStatusContainer: {
    fontFamily: "Roboto Condensed, sans-serif",
    fontSize: "12pt",
    color: "white",
    fontWeight: "bold",
    padding: "0.25em 0.75em",
    borderRadius: "1em",
    backgroundColor: "#727272",
    alignSelf: "center",
  },
  gameStatus: {
    display: "inline-block",
    width: "12px",
    height: "12px",
    borderRadius: "50%",
    marginRight: "0.5em"
  },
  gameStatus_waiting: {
    backgroundColor: "yellow"
  },
  gameStatus_inProgress: {
    backgroundColor: "#14F600"
  },
  gameStatus_finished: {
    backgroundColor: "white"
  },
}