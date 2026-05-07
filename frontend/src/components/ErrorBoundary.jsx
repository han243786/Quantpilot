import React from "react";

export default class ErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false };
  }
  static getDerivedStateFromError() {
    return { hasError: true };
  }
  render() {
    if (this.state.hasError) {
      return (
        <div className="error-boundary-fallback" style={{ padding: 40, textAlign: "center" }}>
          <h3 style={{ color: "#e2e8f0", margin: 0 }}>
            {this.props.fallbackTitle || "界面加载失败"}
          </h3>
          <p style={{ color: "#64748b", fontSize: 13, marginTop: 8 }}>
            {this.props.fallbackText || "请刷新页面重试，或返回上一页。"}
          </p>
          {this.props.onRetry ? (
            <button
              className="ghost-btn"
              onClick={this.props.onRetry}
              style={{ marginTop: 12 }}
            >
              重试
            </button>
          ) : null}
        </div>
      );
    }
    return this.props.children;
  }
}
