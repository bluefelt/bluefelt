import React from 'react';
import type { GameInstance, EntityUI } from '../types/game-types';
import { Board } from './zones/Board';
import CardZone from './CardZone';
import { ChoiceZone } from './zones/ChoiceZone';
import ViewZone from './zones/ViewZone';
import GameHeader from './GameHeader';

interface GameViewProps {
  game: GameInstance;
  entityUI: Record<string, EntityUI>;
  onEntityClick: (entityId: string, actionId: string) => void;
  onZoneClick: (zoneId: string, position?: [number, number], actionId?: string) => void;
}

export function GameView({ 
  game, 
  entityUI,
  onEntityClick,
  onZoneClick 
}: GameViewProps) {
  if (!game) {
    return <div>Loading game...</div>;
  }
  
  const { state, ui, you } = game;
  
  if (!state || !ui) {
    return <div>Loading game state...</div>;
  }
  
  const yourEntityUI = entityUI[you] || {};
  
  // Get zone display information from UI metadata
  const zoneMetadata = ui?.zones || [];
  const zoneDisplay = zoneMetadata.reduce((acc, zone) => {
    acc[zone.id] = zone;
    return acc;
  }, {} as Record<string, any>);
  
  // Render zone based on its type and tier
  const renderZone = (zoneId: string, zone: any) => {
    const display = zoneDisplay[zoneId] || {};
    const tier = display.tier || 0;
    
    // Get entities in this zone with their UI data
    const getEntityUI = (entityId: string | null) => {
      if (!entityId || typeof entityId !== 'string') return null;
      return yourEntityUI[entityId];
    };
    
    // Check if zone itself has interactions
    const zoneInteractions = display.interactions || [];
    
    if (zone.cells) {
      // Grid zone (board)
      return (
        <Board
          key={zoneId}
          zone={zone}
          zoneId={zoneId}
          getEntityUI={getEntityUI}
          onEntityClick={onEntityClick}
          onCellClick={(row, col) => {
            // Check if there are zone-level interactions for empty cells
            if (!zone.cells[row][col] && zoneInteractions.length > 0) {
              onZoneClick(zoneId, [row, col], zoneInteractions[0].actionId);
            }
          }}
          tier={tier}
        />
      );
    } else if (zone.items) {
      // List zone (hand, deck, etc.)
      if (display.display === 'choice') {
        return (
          <ChoiceZone
            key={zoneId}
            zone={zone}
            zoneId={zoneId}
            getEntityUI={getEntityUI}
            onEntityClick={onEntityClick}
          />
        );
      } else if (display.visibility === 'private' && zone.owner !== you) {
        // Hidden zone for other players
        return (
          <ViewZone
            key={zoneId}
            zone={zone}
            zoneId={zoneId}
            label={display.label || zoneId}
            count={zone.items.length}
          />
        );
      } else {
        // Regular card zone
        return (
          <CardZone
            key={zoneId}
            zone={zone}
            zoneId={zoneId}
            label={display.label || zoneId}
            getEntityUI={getEntityUI}
            onEntityClick={onEntityClick}
            layout={display.layout || 'stack'}
            maxDisplay={display.maxDisplay}
          />
        );
      }
    }
    
    return null;
  };
  
  // Group zones by tier for layout
  const stateZones = state.zones || {};
  console.log('[GameView] State zones:', stateZones);
  console.log('[GameView] Zone display metadata:', zoneDisplay);
  
  const zonesByTier = Object.entries(stateZones).reduce((acc, [zoneId, zone]) => {
    const tier = zoneDisplay[zoneId]?.tier || 0;
    if (!acc[tier]) acc[tier] = [];
    acc[tier].push({ zoneId, zone });
    return acc;
  }, {} as Record<number, Array<{ zoneId: string; zone: any }>>);
  
  console.log('[GameView] Zones by tier:', zonesByTier);
  
  return (
    <div className="game-view">
      <GameHeader
        lobbyId={game.id || 'unknown'}
        gameId={game.gameId || 'unknown'}
        gameName={ui?.gameMetadata?.metadata?.name || game.gameId || 'Unknown Game'}
        status={state.gameStatus?.state === 'ended' ? 'finished' : (state.gameStatus ? 'in_progress' : 'waiting')}
        players={state.players || []}
        currentPlayer={state.currentPlayer}
        entityDefinitions={ui?.entities}
        turnPrompt={state.turnPrompt}
      />
      
      <div className="game-zones">
        {/* Render zones by tier */}
        {Object.entries(zonesByTier)
          .sort(([a], [b]) => Number(a) - Number(b))
          .map(([tier, zones]) => (
            <div key={tier} className={`zone-tier tier-${tier}`}>
              {zones.map(({ zoneId, zone }) => renderZone(zoneId, zone))}
            </div>
          ))}
      </div>
      
      {/* Game log */}
      {ui.gameLog && ui.gameLog.length > 0 && (
        <div className="game-log">
          <h3>Game Log</h3>
          <div className="log-entries">
            {ui.gameLog.slice(-5).map((entry, idx) => (
              <div key={idx} className={`log-entry log-${entry.type}`}>
                {entry.message}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}