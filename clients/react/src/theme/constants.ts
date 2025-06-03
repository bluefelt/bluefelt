// Theme constants for consistent styling across the application

export const theme = {
  colors: {
    background: {
      primary: 'bg-gray-900',
      secondary: 'bg-gray-800',
      tertiary: 'bg-gray-700',
      hover: 'hover:bg-gray-600'
    },
    text: {
      primary: 'text-white',
      secondary: 'text-gray-300',
      muted: 'text-gray-400',
      error: 'text-red-500',
      success: 'text-green-500',
      warning: 'text-yellow-500'
    },
    border: {
      default: 'border-gray-700',
      highlight: 'border-yellow-400',
      white: 'border-white'
    },
    button: {
      primary: 'bg-blue-600 hover:bg-blue-700',
      secondary: 'bg-gray-700 hover:bg-gray-600',
      success: 'bg-green-600 hover:bg-green-700',
      danger: 'bg-red-600 hover:bg-red-700',
      warning: 'bg-yellow-600 hover:bg-yellow-700'
    }
  },
  spacing: {
    container: 'max-w-7xl mx-auto px-4 py-8',
    section: 'mb-6',
    card: 'p-6 rounded-lg'
  },
  transitions: {
    default: 'transition-colors',
    all: 'transition-all duration-300 ease'
  }
};

// Utility function to combine classes
export function cn(...classes: (string | undefined | false)[]) {
  return classes.filter(Boolean).join(' ');
}