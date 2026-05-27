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
      const retry = this.props.onRetry || (() => {
        this.setState({ hasError: false });
        window.location.reload();
      });
      return (
        <div className="error-boundary-fallback" role="alert">
          <div className="error-boundary-fallback__icon" aria-hidden="true">!</div>
          <h3>
            {this.props.fallbackTitle || translateText("界面加载失败")}
          </h3>
          <p>
            {this.props.fallbackText || translateText("请刷新页面重试，或返回上一页。")}
          </p>
          <button className="ad-btn ad-btn--ghost" onClick={retry}>
            {translateText("重试")}
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
