import { useState, useEffect } from 'react';
import TestGameView from '../components/TestGameView';
import Button from '../components/ui/Button';

// This component is only available in development

// Consistent styling classes
const inputClasses = "w-full p-2 border rounded bg-gray-600 text-white border-gray-500";
const labelClasses = "block text-sm font-medium mb-1 text-gray-300";
const selectClasses = "w-full p-2 border rounded bg-gray-600 text-white border-gray-500";

interface TestState {
  game: {
    currentPlayer: string;
    phase: string;
    players: string[];
  };
  zones: Record<string, any>;
  ui: {
    actionMap: Record<string, any>;
    layout?: Record<string, string>;
    deckDisplay?: Record<string, any>;
  };
  meta: {
    gameStatus: {
      state: string;
      winner?: string | null;
      tie?: boolean;
    };
  };
}

const PRESET_STATES = {
  'empty-3x3-board': {
    name: 'Empty 3x3 Board',
    state: {
      game: { currentPlayer: 'p1', phase: 'play', players: ['p1', 'p2'] },
      zones: { board: [[null, null, null], [null, null, null], [null, null, null]] },
      ui: { 
        actionMap: {
          '/zones/board/0/0': { action: 'place', direction: 'Click to place' },
          '/zones/board/0/1': { action: 'place', direction: 'Click to place' },
          '/zones/board/0/2': { action: 'place', direction: 'Click to place' },
          '/zones/board/1/0': { action: 'place', direction: 'Click to place' },
          '/zones/board/1/1': { action: 'place', direction: 'Click to place' },
          '/zones/board/1/2': { action: 'place', direction: 'Click to place' },
          '/zones/board/2/0': { action: 'place', direction: 'Click to place' },
          '/zones/board/2/1': { action: 'place', direction: 'Click to place' },
          '/zones/board/2/2': { action: 'place', direction: 'Click to place' }
        }
      },
      meta: { gameStatus: { state: 'active' } }
    }
  },
  'checkerboard-5x5': {
    name: 'Checkerboard 5x5',
    state: {
      game: { currentPlayer: 'p1', phase: 'play', players: ['p1', 'p2'] },
      zones: {
        board: Array(5).fill(null).map((_, row) =>
          Array(5).fill(null).map((_, col) =>
            (row + col) % 2 === 0 
              ? { entity: 'mark_p1', owner: 'p1' }
              : { entity: 'mark_p2', owner: 'p2' }
          )
        )
      },
      ui: { actionMap: {} },
      meta: { gameStatus: { state: 'active' } }
    }
  },
  'large-empty-board': {
    name: '20x20 Empty Board',
    state: {
      game: { currentPlayer: 'p1', phase: 'play', players: ['p1', 'p2'] },
      zones: { board: Array(20).fill(null).map(() => Array(20).fill(null)) },
      ui: { actionMap: {} },
      meta: { gameStatus: { state: 'active' } }
    }
  },
  'card-hand-7': {
    name: '7 Card Hand',
    state: {
      game: { currentPlayer: 'p1', phase: 'play', players: ['p1', 'p2'] },
      zones: {
        hand_p1: [
          { entity: 'card_A_hearts', rank: 'A', suit: 'hearts' },
          { entity: 'card_K_spades', rank: 'K', suit: 'spades' },
          { entity: 'card_Q_diamonds', rank: 'Q', suit: 'diamonds' },
          { entity: 'card_J_clubs', rank: 'J', suit: 'clubs' },
          { entity: 'card_10_hearts', rank: '10', suit: 'hearts' },
          { entity: 'card_9_spades', rank: '9', suit: 'spades' },
          { entity: 'card_8_diamonds', rank: '8', suit: 'diamonds' }
        ]
      },
      ui: { 
        actionMap: Object.fromEntries(
          Array(7).fill(null).map((_, i) => [`/zones/hand_p1/${i}`, { action: 'play', direction: 'Play card' }])
        )
      },
      meta: { gameStatus: { state: 'active' } }
    }
  },
  'choice-zone-ranks': {
    name: 'Rank Choice Zone',
    state: {
      game: { currentPlayer: 'p1', phase: 'choice', players: ['p1', 'p2'] },
      zones: {
        choices: ['A', '2', '3', '4', '5', '6', '7', '8', '9', '10', 'J', 'Q', 'K']
      },
      ui: {
        actionMap: {
          '/zones/choices': { action: 'choose', direction: 'Select a rank' }
        }
      },
      meta: { gameStatus: { state: 'active' } }
    }
  },
  'game-ended-win': {
    name: 'Game Ended (Win)',
    state: {
      game: { currentPlayer: 'p1', phase: 'end', players: ['p1', 'p2'] },
      zones: {
        board: [
          [{ entity: 'mark_p1', owner: 'p1' }, { entity: 'mark_p1', owner: 'p1' }, { entity: 'mark_p1', owner: 'p1' }],
          [{ entity: 'mark_p2', owner: 'p2' }, { entity: 'mark_p2', owner: 'p2' }, null],
          [null, null, null]
        ]
      },
      ui: { actionMap: {} },
      meta: { gameStatus: { state: 'ended', winner: 'p1', tie: false } }
    }
  }
};

export default function UITestPage() {
  const [testState, setTestState] = useState<TestState>(PRESET_STATES['empty-3x3-board'].state);
  const [selectedPreset, setSelectedPreset] = useState('empty-3x3-board');
  const [lastUpdate, setLastUpdate] = useState(new Date());
  
  // Redirect if in production
  useEffect(() => {
    if (import.meta.env.PROD) {
      window.location.href = '/';
    }
  }, []);
  
  // Update timestamp when testState changes
  useEffect(() => {
    setLastUpdate(new Date());
  }, [testState]);
  
  // Board configuration
  const [boardRows, setBoardRows] = useState(3);
  const [boardCols, setBoardCols] = useState(3);
  const [fillPattern, setFillPattern] = useState('empty');
  const [entityType, setEntityType] = useState('mark');
  
  // Card configuration
  const [cardCount, setCardCount] = useState(7);
  const [cardLayout, setCardLayout] = useState('hand');
  const [cardsSelectable, setCardsSelectable] = useState(true);
  
  // Choice configuration
  const [choiceType, setChoiceType] = useState('rank');
  const [choiceCount, setChoiceCount] = useState(13);
  
  // Game state configuration
  const [currentPlayer, setCurrentPlayer] = useState('p1');
  const [phase, setPhase] = useState('play');
  const [gameStatus, setGameStatus] = useState('active');
  const [winner, setWinner] = useState<string | null>(null);
  
  // Action map configuration
  const [actionPattern, setActionPattern] = useState('all');
  const [actionType, setActionType] = useState('place');

  const applyPreset = (presetId: string) => {
    const preset = PRESET_STATES[presetId as keyof typeof PRESET_STATES];
    if (preset) {
      setTestState(preset.state);
      setSelectedPreset(presetId);
    }
  };

  const generateBoard = () => {
    const board = Array(boardRows).fill(null).map(() => Array(boardCols).fill(null));
    
    switch (fillPattern) {
      case 'checkerboard':
        for (let row = 0; row < boardRows; row++) {
          for (let col = 0; col < boardCols; col++) {
            if ((row + col) % 2 === 0) {
              board[row][col] = { entity: `${entityType}_p1`, owner: 'p1' };
            } else {
              board[row][col] = { entity: `${entityType}_p2`, owner: 'p2' };
            }
          }
        }
        break;
      case 'random':
        for (let row = 0; row < boardRows; row++) {
          for (let col = 0; col < boardCols; col++) {
            if (Math.random() > 0.5) {
              const player = Math.random() > 0.5 ? 'p1' : 'p2';
              board[row][col] = { entity: `${entityType}_${player}`, owner: player };
            }
          }
        }
        break;
      case 'full':
        for (let row = 0; row < boardRows; row++) {
          for (let col = 0; col < boardCols; col++) {
            const player = (row * boardCols + col) % 2 === 0 ? 'p1' : 'p2';
            board[row][col] = { entity: `${entityType}_${player}`, owner: player };
          }
        }
        break;
    }
    
    return board;
  };

  const generateActionMap = (board: any[][]) => {
    const actionMap: Record<string, any> = {};
    
    switch (actionPattern) {
      case 'all':
        for (let row = 0; row < board.length; row++) {
          for (let col = 0; col < board[row].length; col++) {
            if (!board[row][col]) {
              actionMap[`/zones/board/${row}/${col}`] = {
                action: actionType,
                direction: `Click to ${actionType}`
              };
            }
          }
        }
        break;
      case 'alternating':
        for (let row = 0; row < board.length; row++) {
          for (let col = 0; col < board[row].length; col++) {
            if ((row + col) % 2 === 0 && !board[row][col]) {
              actionMap[`/zones/board/${row}/${col}`] = {
                action: actionType,
                direction: `Click to ${actionType}`
              };
            }
          }
        }
        break;
      case 'random':
        for (let row = 0; row < board.length; row++) {
          for (let col = 0; col < board[row].length; col++) {
            if (Math.random() > 0.7 && !board[row][col]) {
              actionMap[`/zones/board/${row}/${col}`] = {
                action: actionType,
                direction: `Click to ${actionType}`
              };
            }
          }
        }
        break;
    }
    
    return actionMap;
  };

  const generateCards = () => {
    const suits = ['hearts', 'diamonds', 'clubs', 'spades'];
    const ranks = ['A', '2', '3', '4', '5', '6', '7', '8', '9', '10', 'J', 'Q', 'K'];
    const cards = [];
    
    for (let i = 0; i < cardCount; i++) {
      const suit = suits[i % 4];
      const rank = ranks[i % 13];
      cards.push({
        entity: `card_${rank}_${suit}`,
        rank,
        suit,
        owner: 'p1'
      });
    }
    
    return cards;
  };

  const generateChoices = () => {
    switch (choiceType) {
      case 'rank':
        return ['A', '2', '3', '4', '5', '6', '7', '8', '9', '10', 'J', 'Q', 'K'].slice(0, choiceCount);
      case 'suit':
        return ['hearts', 'diamonds', 'clubs', 'spades'];
      case 'color':
        return ['red', 'black'];
      case 'custom':
        return Array(choiceCount).fill(null).map((_, i) => `Choice ${i + 1}`);
      default:
        return [];
    }
  };

  const updateBoardState = () => {
    const board = generateBoard();
    const actionMap = generateActionMap(board);
    
    setTestState(prev => ({
      ...prev,
      zones: { ...prev.zones, board },
      ui: { ...prev.ui, actionMap }
    }));
  };

  const updateCardState = () => {
    const cards = generateCards();
    const actionMap = cardsSelectable 
      ? Object.fromEntries(cards.map((_, i) => [`/zones/hand_p1/${i}`, { action: 'select', direction: 'Select card' }]))
      : {};
    
    setTestState(prev => ({
      ...prev,
      zones: { ...prev.zones, hand_p1: cards },
      ui: { ...prev.ui, actionMap, layout: { hand_p1: cardLayout } }
    }));
  };

  const updateChoiceState = () => {
    const choices = generateChoices();
    
    setTestState(prev => ({
      ...prev,
      zones: { ...prev.zones, choices },
      ui: {
        ...prev.ui,
        actionMap: {
          '/zones/choices': { action: 'choose', direction: 'Select one' }
        }
      }
    }));
  };

  const updateGameState = () => {
    setTestState(prev => ({
      ...prev,
      game: {
        ...prev.game,
        currentPlayer,
        phase,
        players: ['p1', 'p2', 'p3', 'p4'].slice(0, 2) // Support for more players later
      },
      meta: {
        gameStatus: {
          state: gameStatus,
          winner: gameStatus === 'ended' ? winner : null,
          tie: gameStatus === 'ended' && !winner
        }
      }
    }));
  };

  const exportState = () => {
    const stateJson = JSON.stringify(testState, null, 2);
    const blob = new Blob([stateJson], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'test-state.json';
    a.click();
  };

  const importState = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = (e) => {
        try {
          const state = JSON.parse(e.target?.result as string);
          setTestState(state);
        } catch (error) {
          alert('Invalid JSON file');
        }
      };
      reader.readAsText(file);
    }
  };

  return (
    <div className="fixed inset-0 bg-gray-100 z-50">
      <div className="flex h-full">
        {/* Control Panel */}
        <div className="w-96 bg-gray-800 text-white shadow-lg overflow-y-auto">
          <div className="p-6 space-y-6">
            <h1 className="text-2xl font-bold text-white">UI Test Harness</h1>
            <div className="bg-blue-900 border border-blue-700 rounded-lg p-3 text-sm">
              <p className="text-blue-200">
                💡 Remember to click "Apply" buttons after changing settings to update the preview.
              </p>
            </div>
            
            {/* Presets */}
            <div className="bg-gray-700 rounded-lg p-4">
              <h2 className="text-lg font-semibold mb-3 text-white">Presets</h2>
                <select 
                  value={selectedPreset}
                  onChange={(e) => applyPreset(e.target.value)}
                  className="w-full p-2 border rounded bg-gray-600 text-white border-gray-500"
                >
                  {Object.entries(PRESET_STATES).map(([id, preset]) => (
                    <option key={id} value={id} className="bg-gray-600">{preset.name}</option>
                  ))}
                </select>
              </div>

            {/* Board Configuration */}
            <div className="bg-gray-700 rounded-lg p-4 space-y-3">
              <h2 className="text-lg font-semibold text-white">Board Configuration</h2>
                
                <div className="grid grid-cols-2 gap-3">
                  <div>
                    <label className="block text-sm font-medium mb-1 text-gray-300">Rows</label>
                    <input
                      type="number"
                      value={boardRows}
                      onChange={(e) => setBoardRows(parseInt(e.target.value) || 3)}
                      className="w-full p-2 border rounded bg-gray-600 text-white border-gray-500"
                      min="1"
                      max="50"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium mb-1 text-gray-300">Columns</label>
                    <input
                      type="number"
                      value={boardCols}
                      onChange={(e) => setBoardCols(parseInt(e.target.value) || 3)}
                      className="w-full p-2 border rounded bg-gray-600 text-white border-gray-500"
                      min="1"
                      max="50"
                    />
                  </div>
                </div>
                
                <div>
                  <label className="block text-sm font-medium mb-1 text-gray-300">Fill Pattern</label>
                  <select
                    value={fillPattern}
                    onChange={(e) => setFillPattern(e.target.value)}
                    className="w-full p-2 border rounded bg-gray-600 text-white border-gray-500"
                  >
                    <option value="empty" className="bg-gray-600">Empty</option>
                    <option value="checkerboard" className="bg-gray-600">Checkerboard</option>
                    <option value="random" className="bg-gray-600">Random</option>
                    <option value="full" className="bg-gray-600">Full</option>
                  </select>
                </div>
                
                <div>
                  <label className="block text-sm font-medium mb-1 text-gray-300">Entity Type</label>
                  <select
                    value={entityType}
                    onChange={(e) => setEntityType(e.target.value)}
                    className="w-full p-2 border rounded bg-gray-600 text-white border-gray-500"
                  >
                    <option value="mark" className="bg-gray-600">Mark (X/O)</option>
                    <option value="piece" className="bg-gray-600">Piece</option>
                    <option value="stone" className="bg-gray-600">Stone</option>
                    <option value="checker" className="bg-gray-600">Checker</option>
                  </select>
                </div>
                
                <div>
                  <label className="block text-sm font-medium mb-1 text-gray-300">Action Pattern</label>
                  <select
                    value={actionPattern}
                    onChange={(e) => setActionPattern(e.target.value)}
                    className="w-full p-2 border rounded bg-gray-600 text-white border-gray-500"
                  >
                    <option value="none" className="bg-gray-600">None</option>
                    <option value="all" className="bg-gray-600">All Empty</option>
                    <option value="alternating" className="bg-gray-600">Alternating</option>
                    <option value="random" className="bg-gray-600">Random</option>
                  </select>
                </div>
                
                <Button onClick={updateBoardState} className="w-full">
                  Apply Board Settings
                </Button>
              </div>

            {/* Card Configuration */}
            <div className="bg-gray-700 rounded-lg p-4 space-y-3">
              <h2 className="text-lg font-semibold text-white">Card Configuration</h2>
                
                <div>
                  <label className="block text-sm font-medium mb-1">Card Count</label>
                  <input
                    type="number"
                    value={cardCount}
                    onChange={(e) => setCardCount(parseInt(e.target.value) || 7)}
                    className="w-full p-2 border rounded"
                    min="0"
                    max="200"
                  />
                </div>
                
                <div>
                  <label className="block text-sm font-medium mb-1">Layout</label>
                  <select
                    value={cardLayout}
                    onChange={(e) => setCardLayout(e.target.value)}
                    className="w-full p-2 border rounded"
                  >
                    <option value="hand">Hand (fan)</option>
                    <option value="stack">Stack</option>
                    <option value="grid">Grid</option>
                    <option value="spread">Spread</option>
                  </select>
                </div>
                
                <div>
                  <label className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      checked={cardsSelectable}
                      onChange={(e) => setCardsSelectable(e.target.checked)}
                      className="rounded"
                    />
                    <span className="text-sm font-medium">Cards Selectable</span>
                  </label>
                </div>
                
                <Button onClick={updateCardState} className="w-full">
                  Apply Card Settings
                </Button>
              </div>

            {/* Choice Configuration */}
            <div className="bg-gray-700 rounded-lg p-4 space-y-3">
              <h2 className="text-lg font-semibold text-white">Choice Configuration</h2>
                
                <div>
                  <label className="block text-sm font-medium mb-1">Choice Type</label>
                  <select
                    value={choiceType}
                    onChange={(e) => setChoiceType(e.target.value)}
                    className="w-full p-2 border rounded"
                  >
                    <option value="rank">Card Ranks</option>
                    <option value="suit">Card Suits</option>
                    <option value="color">Colors</option>
                    <option value="custom">Custom</option>
                  </select>
                </div>
                
                {choiceType === 'custom' && (
                  <div>
                    <label className="block text-sm font-medium mb-1">Choice Count</label>
                    <input
                      type="number"
                      value={choiceCount}
                      onChange={(e) => setChoiceCount(parseInt(e.target.value) || 10)}
                      className="w-full p-2 border rounded"
                      min="1"
                      max="100"
                    />
                  </div>
                )}
                
                <Button onClick={updateChoiceState} className="w-full">
                  Apply Choice Settings
                </Button>
              </div>

            {/* Game State Configuration */}
            <div className="bg-gray-700 rounded-lg p-4 space-y-3">
              <h2 className="text-lg font-semibold text-white">Game State</h2>
                
                <div>
                  <label className="block text-sm font-medium mb-1">Current Player</label>
                  <select
                    value={currentPlayer}
                    onChange={(e) => setCurrentPlayer(e.target.value)}
                    className="w-full p-2 border rounded"
                  >
                    <option value="p1">Player 1</option>
                    <option value="p2">Player 2</option>
                    <option value="p3">Player 3</option>
                    <option value="p4">Player 4</option>
                  </select>
                </div>
                
                <div>
                  <label className="block text-sm font-medium mb-1">Phase</label>
                  <select
                    value={phase}
                    onChange={(e) => setPhase(e.target.value)}
                    className="w-full p-2 border rounded"
                  >
                    <option value="setup">Setup</option>
                    <option value="play">Play</option>
                    <option value="choice">Choice</option>
                    <option value="end">End</option>
                  </select>
                </div>
                
                <div>
                  <label className="block text-sm font-medium mb-1">Game Status</label>
                  <select
                    value={gameStatus}
                    onChange={(e) => setGameStatus(e.target.value)}
                    className="w-full p-2 border rounded"
                  >
                    <option value="active">Active</option>
                    <option value="ended">Ended</option>
                  </select>
                </div>
                
                {gameStatus === 'ended' && (
                  <div>
                    <label className="block text-sm font-medium mb-1">Winner</label>
                    <select
                      value={winner || 'tie'}
                      onChange={(e) => setWinner(e.target.value === 'tie' ? null : e.target.value)}
                      className="w-full p-2 border rounded"
                    >
                      <option value="tie">Tie</option>
                      <option value="p1">Player 1</option>
                      <option value="p2">Player 2</option>
                    </select>
                  </div>
                )}
                
                <Button onClick={updateGameState} className="w-full">
                  Apply Game State
                </Button>
              </div>

            {/* Import/Export */}
            <div className="bg-gray-700 rounded-lg p-4 space-y-3">
              <h2 className="text-lg font-semibold text-white">Import/Export</h2>
                
                <Button onClick={exportState} className="w-full">
                  Export Current State
                </Button>
                
                <div>
                  <label className="block text-sm font-medium mb-1">Import State</label>
                  <input
                    type="file"
                    accept=".json"
                    onChange={importState}
                    className="w-full p-2 border rounded"
                  />
                </div>
              </div>
          </div>
        </div>

        {/* Preview Area */}
        <div className="flex-1 bg-gray-200 overflow-auto p-8">
          <div className="bg-gray-800 rounded-lg shadow-lg p-6">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-xl font-semibold text-white">Preview</h2>
              <div className="text-right">
                <span className="text-sm text-gray-300 block">Console warnings are expected in test mode</span>
                <span className="text-xs text-blue-400">
                  Last update: {lastUpdate.toLocaleTimeString()}
                </span>
              </div>
            </div>
            <div className="border-2 border-gray-600 rounded-lg p-6 bg-white min-h-96">
              <TestGameView 
                initialState={testState}
              />
            </div>
          </div>
          
          {/* State Viewer */}
          <div className="mt-8 bg-gray-800 rounded-lg shadow-lg p-6">
            <h2 className="text-xl font-semibold mb-4 text-white">Current State (JSON)</h2>
            <pre className="bg-gray-900 text-gray-100 p-4 rounded overflow-auto text-xs max-h-96">
              {JSON.stringify(testState, null, 2)}
            </pre>
          </div>
        </div>
      </div>
    </div>
  );
}