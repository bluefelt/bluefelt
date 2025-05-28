import React, { Component, type ReactNode } from 'react';
import { type NavigateFunction } from 'react-router-dom';

interface Props {
  children: ReactNode;
  navigate: NavigateFunction;
}

interface State {
  hasError: boolean;
}

export default class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(): State {
    return { hasError: true };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Error caught by boundary:', error, errorInfo);
  }

  componentDidUpdate(_prevProps: Props, prevState: State) {
    if (!prevState.hasError && this.state.hasError) {
      // Redirect to home on error
      this.props.navigate('/', { replace: true });
    }
  }

  render() {
    if (this.state.hasError) {
      return null; // Don't render anything while redirecting
    }

    return this.props.children;
  }
}

// Hook to use ErrorBoundary with navigation
export function withErrorBoundary<P extends object>(
  Component: React.ComponentType<P>
): React.ComponentType<P> {
  return function WithErrorBoundaryComponent(props: P) {
    const navigate = useNavigate();
    
    return (
      <ErrorBoundary navigate={navigate}>
        <Component {...props} />
      </ErrorBoundary>
    );
  };
}

// Import at the top level where it's used
import { useNavigate } from 'react-router-dom';