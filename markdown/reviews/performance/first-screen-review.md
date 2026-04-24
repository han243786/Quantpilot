# 首屏加载时间对照

- 生成时间：2026/4/14 18:12:40
- 采样环境：Playwright + Edge（preview 构建）
- 视口：1280 x 900
- 样本数：每个页面 3 次冷启动
- 指标说明：
  - DOMContentLoaded：文档解析完成时间
  - load：页面 load 事件完成时间
  - 首屏关键锚点可见：页面关键容器首次稳定可见时间
  - FCP：浏览器 First Contentful Paint

## 结论排序

1. 回测对比页：首屏关键锚点平均 128.97 ms，DOMContentLoaded 平均 46.47 ms，load 平均 46.9 ms
2. 回测详情页：首屏关键锚点平均 188 ms，DOMContentLoaded 平均 51.23 ms，load 平均 51.3 ms
3. 编辑器首页：首屏关键锚点平均 240.13 ms，DOMContentLoaded 平均 54.57 ms，load 平均 55.07 ms

## 明细

## 编辑器首页

- 路径：`/`
- 关键锚点：`.main-workspace`
- DOMContentLoaded：平均 54.57 ms，范围 49.1 - 64.5 ms
- load：平均 55.07 ms，范围 49.8 - 64.6 ms
- 首屏关键锚点可见：平均 240.13 ms，范围 209.2 - 262.8 ms
- First Contentful Paint：平均 134.67 ms，范围 96 - 208 ms

| 样本 | DOMContentLoaded (ms) | load (ms) | 关键锚点可见 (ms) | FCP (ms) |
| --- | ---: | ---: | ---: | ---: |
| 1 | 64.5 | 64.6 | 262.8 | 208 |
| 2 | 49.1 | 49.8 | 209.2 | 96 |
| 3 | 50.1 | 50.8 | 248.4 | 100 |

## 回测详情页

- 路径：`/backtests/backtest_smoke_001`
- 关键锚点：`.analysis-summary-grid`
- DOMContentLoaded：平均 51.23 ms，范围 48.7 - 52.8 ms
- load：平均 51.3 ms，范围 48.8 - 52.8 ms
- 首屏关键锚点可见：平均 188 ms，范围 185.9 - 191.5 ms
- First Contentful Paint：平均 94.67 ms，范围 88 - 100 ms

| 样本 | DOMContentLoaded (ms) | load (ms) | 关键锚点可见 (ms) | FCP (ms) |
| --- | ---: | ---: | ---: | ---: |
| 1 | 52.8 | 52.8 | 186.6 | 100 |
| 2 | 48.7 | 48.8 | 191.5 | 88 |
| 3 | 52.2 | 52.3 | 185.9 | 96 |

## 回测对比页

- 路径：`/backtests/compare?ids=backtest_smoke_001,backtest_compare_002`
- 关键锚点：`.analysis-card-grid`
- DOMContentLoaded：平均 46.47 ms，范围 41.5 - 52.8 ms
- load：平均 46.9 ms，范围 42.1 - 52.9 ms
- 首屏关键锚点可见：平均 128.97 ms，范围 125.5 - 135.1 ms
- First Contentful Paint：平均 94.67 ms，范围 88 - 104 ms

| 样本 | DOMContentLoaded (ms) | load (ms) | 关键锚点可见 (ms) | FCP (ms) |
| --- | ---: | ---: | ---: | ---: |
| 1 | 52.8 | 52.9 | 135.1 | 104 |
| 2 | 41.5 | 42.1 | 126.3 | 88 |
| 3 | 45.1 | 45.7 | 125.5 | 92 |
