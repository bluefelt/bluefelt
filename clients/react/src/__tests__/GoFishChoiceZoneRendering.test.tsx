import { render, screen, within } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import GameZones from '../components/GameZones';
import { PlayerProvider } from '../context/PlayerContext';

describe('Go Fish Choice Zone Rendering', () => {
  it('should render choice zone with ranks from multi-step state', () => {
    const mockMultiStepState = {
      actionType: 'bf.selectChoice',
      currentStepId: 'selectRank',
      stepActionMap: {
        '/ranks/3': {
          action: 'multiStepSelect',
          args: { choice: '3', label: '3' }
        },
        '/ranks/5': {
          action: 'multiStepSelect',
          args: { choice: '5', label: '5' }
        },
        '/ranks/k': {
          action: 'multiStepSelect',
          args: { choice: 'k', label: 'King' }
        },
        '/ranks/a': {
          action: 'multiStepSelect',
          args: { choice: 'a', label: 'Ace' }
        }
      }
    };

    const mockZones = {
      choice_p1: {
        type: 'choice',
        items: [], // Empty items - dynamic choices come from stepActionMap
        prompt: 'Choose a rank to ask for'
      }
    };

    const mockZoneMetadata = [
      {
        id: 'choice_p1',
        renderType: 'choice',
        visibility: 'owner',
        owner: 'p1'
      }
    ];

    const mockOnChoiceSelect = vi.fn();

    render(
      <PlayerProvider initialPlayer="p1">
        <GameZones
          zones={mockZones}
          entityDefinitions={[]}
          onChoiceSelect={mockOnChoiceSelect}
          isMyTurn={true}
          you="p1"
          zoneMetadata={mockZoneMetadata}
          playerNames={['Alice', 'Bob']}
          actionMap={{ p1: {}, p2: {} }}
          multiStepState={mockMultiStepState}
        />
      </PlayerProvider>
    );

    // Check that the choice zone is rendered
    const choiceZone = screen.getByTestId('choice-zone');
    expect(choiceZone).toBeTruthy();

    // Check that all ranks are displayed
    expect(screen.getByText('3')).toBeTruthy();
    expect(screen.getByText('5')).toBeTruthy();
    expect(screen.getByText('King')).toBeTruthy();
    expect(screen.getByText('Ace')).toBeTruthy();

    // Check the prompt
    expect(screen.getByText('Choose a rank to ask for')).toBeTruthy();
  });

  it('should not render choice zone when no choices available', () => {
    const mockZones = {
      choice_p1: {
        type: 'choice',
        items: [],
        prompt: 'Choose a rank to ask for'
      }
    };

    const mockZoneMetadata = [
      {
        id: 'choice_p1',
        renderType: 'choice',
        visibility: 'owner',
        owner: 'p1'
      }
    ];

    render(
      <PlayerProvider initialPlayer="p1">
        <GameZones
          zones={mockZones}
          entityDefinitions={[]}
          onChoiceSelect={vi.fn()}
          isMyTurn={true}
          you="p1"
          zoneMetadata={mockZoneMetadata}
          playerNames={['Alice', 'Bob']}
          actionMap={{ p1: {}, p2: {} }}
          multiStepState={null} // No multi-step state
        />
      </PlayerProvider>
    );

    // Choice zone should not be rendered
    expect(screen.queryByTestId('choice-zone')).toBeFalsy();
  });
});