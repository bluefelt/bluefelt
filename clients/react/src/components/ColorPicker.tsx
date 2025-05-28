import { useState, useRef, useEffect } from 'react';
import { PLAYER_COLORS, getColorById } from '../config/colors';
import { usePlayer } from '../context/PlayerContext';

export default function ColorPicker() {
  const [isOpen, setIsOpen] = useState(false);
  const { player, updateColor } = usePlayer();
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  if (!player) return null;

  const currentColor = getColorById(player.color);

  return (
    <div ref={dropdownRef} className="relative">
      {/* Profile circle button */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-10 h-10 rounded-full flex items-center justify-center text-white font-bold transition-transform hover:scale-110"
        style={{ backgroundColor: currentColor.hex }}
        aria-label="Change color"
      >
        {player.username[0].toUpperCase()}
      </button>

      {/* Color dropdown */}
      {isOpen && (
        <div className="absolute right-0 mt-2 w-48 bg-gray-800 rounded-lg shadow-lg p-2 z-50">
          <div className="text-sm text-gray-400 px-2 py-1">Choose your color</div>
          <div className="grid grid-cols-4 gap-2 mt-2">
            {PLAYER_COLORS.map((color) => (
              <button
                key={color.id}
                onClick={() => {
                  updateColor(color.id);
                  setIsOpen(false);
                }}
                className={`w-10 h-10 rounded-full transition-all hover:scale-110 ${
                  color.id === player.color ? 'ring-2 ring-white ring-offset-2 ring-offset-gray-800' : ''
                }`}
                style={{ backgroundColor: color.hex }}
                aria-label={color.name}
                title={color.name}
              />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}