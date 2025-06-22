import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */
const sidebars: SidebarsConfig = {
  tutorialSidebar: [
    // Core Platform
    {
      type: 'category',
      label: 'Bluefelt Server',
      link: {
        type: 'doc',
        id: 'server',
      },
      items: [
        'state-structure',
        'game-log-parameters',
      ],
    },
    
    {
      type: 'category',
      label: 'Clients',
      link: {
        type: 'doc',
        id: 'clients-overview',
      },
      items: [
        'clients-implementing',
        'clients-react',
        'clients-unity',
      ],
    },
    
    'bluefelt-design-philosophy',
    'business-model-strategy',
    
    // Game Development
    {
      type: 'category',
      label: 'Developing Games',
      link: {
        type: 'doc',
        id: 'developing-games',
      },
      items: [
        'developing-games-actions',
        'developing-games-entities',
        'developing-games-zones',
        'developing-games-phases',
        'developing-games-ui',
        'math-calculation-system',
        'view-zones',
        'zone-hierarchy',
        'multi-step-actions',
      ],
    },
    
    // Development Process
    {
      type: 'category',
      label: 'Testing',
      link: {
        type: 'doc',
        id: 'testing',
      },
      items: [
        'testing-implementation',
      ],
    },
    'development-roadmap',
    'action-execution-refactor',
  ],
};

export default sidebars;
