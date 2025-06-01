
interface InteractiveCellProps {
  zone: string;
  position: [number, number];
  disabled?: boolean;
  className?: string;
  onAction?: (action: any) => void;
}

export default function InteractiveCell({ 
  zone, 
  position, 
  disabled = false, 
  className = '',
  onAction
}: InteractiveCellProps) {
  const handleClick = () => {
    if (!disabled && onAction) {
      onAction({
        action: 'place', // This should come from game state
        zone,
        row: position[0],
        col: position[1]
      });
    }
  };

  return (
    <div 
      className={className}
      onClick={handleClick}
      role="button"
      tabIndex={disabled ? -1 : 0}
      aria-disabled={disabled}
    />
  );
}