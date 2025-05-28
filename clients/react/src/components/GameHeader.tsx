import { Link } from 'react-router-dom';
import { usePlayer } from '../context/PlayerContext';
import { getColorById, getPlayerColor } from '../config/colors';

interface GameHeaderProps {
  lobbyId: string;
  gameId: string;
  gameName: string;
  status: 'waiting' | 'in_progress' | 'finished';
  players: { username: string; isConnected: boolean }[];
  currentPlayer?: string;
}

export default function GameHeader({ lobbyId, gameId, gameName, status, players, currentPlayer }: GameHeaderProps) {
  const { player } = usePlayer();

  const getHeaderPlayerColor = (username: string) => {
    if (!player) return '#888';
    
    const myPlayerIndex = players.findIndex(p => p.username === player.username);
    const targetPlayerIndex = players.findIndex(p => p.username === username);
    
    if (myPlayerIndex === -1 || targetPlayerIndex === -1) return '#888';
    
    const color = getPlayerColor(targetPlayerIndex, player.color, myPlayerIndex);
    return color.hex;
  };

  const getInitials = (username: string) => {
    return username.slice(0, 2).toUpperCase();
  };

  return (
    <div>
      <div>
        <Link to="/lobbies" className="text-blue-400 hover:text-blue-300">
          Lobbies
        </Link>
      </div>
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
      <div className="flex items-center space-x-3">
        {players.map((p, i) => (
          <div key={p.username} className="flex items-center space-x-2">
            <div
              className="w-4 h-4 rounded-sm"
              style={{ backgroundColor: getHeaderPlayerColor(p.username) }}
            />
            <span className={`text-sm ${p.isConnected ? 'text-gray-300' : 'text-gray-500'}`}>
              {p.username}
              {p.username === currentPlayer && ' ●'}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}

const styles = {
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