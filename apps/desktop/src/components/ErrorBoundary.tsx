import { Component, type ErrorInfo, type ReactNode } from "react";

interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  message: string | null;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { message: null };

  static getDerivedStateFromError(error: unknown): ErrorBoundaryState {
    return { message: error instanceof Error ? error.message : String(error) };
  }

  componentDidCatch(error: unknown, info: ErrorInfo): void {
    console.error(error, info.componentStack);
  }

  render(): ReactNode {
    if (this.state.message) {
      return (
        <div className="recovery">
          <p className="eyebrow">Interface error</p>
          <h1 className="empty-title">The window could not draw that.</h1>
          <p className="empty-text">{this.state.message}</p>
          <p className="empty-text">
            Your files and projects are untouched. Reload the window to continue; a run that was
            interrupted resumes from its last finished stage.
          </p>
          <button
            type="button"
            className="button primary"
            onClick={() => window.location.reload()}
          >
            Reload
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
