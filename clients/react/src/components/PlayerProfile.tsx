import { Link, useNavigate } from 'react-router-dom';
import { usePlayer } from '../context/PlayerContext';

export default function PlayerProfile() {
  const { player, logout } = usePlayer();
  const navigate = useNavigate();
  if (!player) return null;

  const handleSignOut = () => {
    logout();
    navigate('/login');
  };

  return (
    <nav style={styles.playerProfile}>
      <a href="#">
        <img src="../../public/bluefelt_logo.jpg" alt="Bluefelt Logo" style={styles.bfLogo} />
      </a>
      <a href="#">
        <div style={styles.playerProfilePicture}>
          <span style={styles.playerProfilePictureLetter}>{player.username.charAt(0)}</span>
        </div>
      </a>
      <div style={styles.playerProfileTop}>
        <div style={styles.playingAsText}>
          <span>
            Playing as
          </span>
          <span style={styles.playerNameName}>
            {player.username}
          </span>
        </div>
      </div>
      <div style={styles.playerProfileBottom}>
        <button onClick={handleSignOut} style={styles.signOutButton}>Sign Out</button>
      </div>
    </nav>
  );
}

const styles = {
  playerProfile: {
    height: "65px",
    backgroundColor: "pink",
  },
  playerProfileTop: {
    height: "37px",
    backgroundColor: "#1E2939",
    display: "flex",
    justifyContent: "end",
    paddingRight: "65px"
  },
  playerProfileBottom: {
    height: "28px",
    backgroundColor: "#D8B260",
    display: "flex",
    justifyContent: "end",
    alignItems: "center",
    paddingRight: "65px"
  },
  playingAsText: {
    display: "flex",
    flexGrow: 1,
    position: "relative",
    justifyContent: "end",
    alignItems: "baseline",
    color: "#D8B260",
    bottom: "-12px",
    fontFamily: "Josefin Sans, sans-serif",
  },
  playerNameName: {
    fontWeight: "bold",
    paddingLeft: "0.25em"
  },
  playerProfilePicture: {
    position: "absolute",
    top: "15px",
    right: "10px",
    height: "43px",
    width: "43px",
    borderRadius: "50%",
    backgroundColor: "black",
    borderWidth: "2px",
    borderColor: "#D8B260",
    display: "flex",
    alignItems: "center",
    justifyContent: "center"
  },
  playerProfilePictureLetter: {
    textTransform: "uppercase",
    fontSize: "1.25em",
    "color": "#D8B260",
  },
  bfLogo: {
    height: "52px",
    width: "52px",
    position: "absolute",
    border: "2px solid #D8B260",
    marginLeft: "10px",
    marginTop: "8px",
  },
  signOutButton: {
    backgroundColor: "#D8B260",
    color: "black",
    border: "2px solid #1E2939",
    shadowOffsetX: "2px",
    shadowOffsetY: "2px",
    shadowColor: "#1E2939",
    fontSize: "0.8rem",
    height: "20px",
    cursor: "pointer",
    padding: "0 0.8em",
    fontFamily: "Josefin Sans, sans-serif",
    fontWeight: "bold",
  }
};