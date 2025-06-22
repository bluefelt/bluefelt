import { render, screen } from '@testing-library/react';
import { TestProviders } from '../test/TestProviders';
import GameZones from '../components/GameZones';

describe('Go Fish Purple Rectangle Debug', () => {
  it('should identify where purple rectangle comes from', () => {
    const mockZones = {
      choice_p1: {
        type: 'choice',
        items: [],
        prompt: 'Choose a rank'
      },
      hand_p1: []
    };

    const mockZoneMetadata = [{
      id: 'choice_p1',
      renderType: 'choice',
      visibility: 'owner',
      owner: 'p1',
      layout_order: 1
    }];

    const mockMultiStepState = {
      actionType: 'bf.selectChoice',
      currentStepId: 'selectRank',
      stepActionMap: {
        '/ranks/3': { action: 'multiStepSelect', args: { choice: '3', label: '3' } },
        '/ranks/5': { action: 'multiStepSelect', args: { choice: '5', label: '5' } }
      }
    };

    const { container } = render(
      <TestProviders>
        <GameZones
          zones={mockZones}
          zoneMetadata={mockZoneMetadata}
          you="p1"
          multiStepState={mockMultiStepState}
          isMyTurn={true}
        />
      </TestProviders>
    );

    // Check what's actually rendered
    console.log('=== RENDERED HTML ===');
    console.log(container.innerHTML);

    // Look for purple elements
    const purpleElements = container.querySelectorAll('.bg-purple-500, .ring-purple-500');
    console.log('\n=== PURPLE ELEMENTS ===');
    purpleElements.forEach((el, i) => {
      console.log(`Purple element ${i}:`, {
        tagName: el.tagName,
        className: el.className,
        parent: el.parentElement?.className,
        innerHTML: el.innerHTML.substring(0, 100)
      });
    });

    // Check for choice zones
    const choiceZones = container.querySelectorAll('[data-testid="choice-zone"]');
    console.log('\n=== CHOICE ZONES ===');
    console.log('Choice zones found:', choiceZones.length);
    choiceZones.forEach((zone, i) => {
      console.log(`Choice zone ${i}:`, {
        className: zone.className,
        childCount: zone.children.length
      });
    });

    // Check for action indicators
    const actionIndicators = container.querySelectorAll('.absolute.inset-0.pointer-events-none');
    console.log('\n=== ACTION INDICATORS ===');
    console.log('Action indicators found:', actionIndicators.length);

    // The choice zone should have buttons
    const buttons = screen.queryAllByRole('button');
    console.log('\n=== BUTTONS ===');
    console.log('Buttons found:', buttons.length);
    buttons.forEach(btn => {
      console.log('  Button:', btn.textContent);
    });
  });
});