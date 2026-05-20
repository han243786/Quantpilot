import React from "react";
import { translateText } from "../i18n";

export default class ErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false };
  }
  static getDerivedStateFromError() {
    return { hasError: true };
  }
  componentDidCatch(error, _errorInfo) {
    // 仅输出 error.message，不泄漏组件栈内部结构
    console.error("[ErrorBoundary]", error.message);
  }
  render() {
    if (this.state.hasError) {
      return (
        <div className="error-boundary-fallback" style={{ padding: 40, textAlign: "center" }}>
          <h3 style={{ color: "var(--ad-text)", margin: 0 }}>
            {this.props.fallbackTitle || translateText("界面加载失败")}
          </h3>
          <p style={{ color: "var(--ad-text-muted)", fontSize: 13, marginTop: 8 }}>
            {this.props.fallbackText || translateText("请刷新页面重试，或返回上一页。")}
          </p>
          {this.props.onRetry ? (
            <button
              className="ghost-btn"
              onClick={this.props.onRetry}
              style={{ marginTop: 12 }}
            >
              {translateText("重试")}
            </button>
          ) : null}
        </div>
      );
    }
    return this.props.children;
  }
}
