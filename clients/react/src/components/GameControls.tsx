import Button from './ui/Button';
import { theme } from '../theme/constants';

interface GameControlsProps {
  joined: boolean;
  isGameStarted: boolean;
  canStartGame: boolean;
  onJoin: () => void;
  onStart: () => void;
  onLeave: () => void;
  playerUsername: string;
}

export default function GameControls({
  joined,
  isGameStarted,
  canStartGame,
  onJoin,
  onStart,
  onLeave,
  playerUsername
}: GameControlsProps) {
  return (
    <div className={`${theme.spacing.section} flex flex-wrap gap-4`}>
      {!joined ? (
        <Button onClick={onJoin} variant="primary">
          Join Game
        </Button>
      ) : (
        <>
          {!isGameStarted && (
            <>
              {canStartGame && (
                <Button onClick={onStart} variant="success">
                  Start Game
                </Button>
              )}
              <Button onClick={onLeave} variant="danger">
                Leave Game
              </Button>
            </>
          )}
        </>
      )}
    </div>
  );
}