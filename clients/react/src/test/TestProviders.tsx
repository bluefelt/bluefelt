import React, { ReactNode } from 'react';
import { PlayerProvider } from '../context/PlayerContext';
import { AnimationProvider } from '../context/AnimationContext';
import { PlayerPreferencesProvider } from '../context/PlayerPreferencesContext';


interface TestProvidersProps {
  children: ReactNode;
  initialPlayer?: { username: string; color: string };
}

export function TestProviders({ children, initialPlayer }: TestProvidersProps) {
  return (
    <PlayerPreferencesProvider>
      <PlayerProvider initialPlayer={initialPlayer}>
        <AnimationProvider>
          {children}
        </AnimationProvider>
      </PlayerProvider>
    </PlayerPreferencesProvider>
  );
}


// Helper function for tests
export function renderWithProviders(component: ReactNode, options?: { initialPlayer?: { username: string; color: string } }) {
  return (
    <TestProviders initialPlayer={options?.initialPlayer}>
      {component}
    </TestProviders>
  );
}