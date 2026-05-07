export function createTutorialSteps(t) {
  return [
    {
      title: t("创建策略"),
      description:
        t("从左侧模块面板拖入数据源节点和意图节点，用连线搭建你的第一个交易策略。"),
      target: '[data-testid="module-sidebar"]',
    },
    {
      title: t("配置参数"),
      description:
        t("点击画布上的节点，在右侧属性面板中设置交易对、K 线周期、指标参数和仓位大小。"),
      target: '[data-testid="property-panel"]',
    },
    {
      title: t("编译检查"),
      description:
        t("点击顶部工具栏的「编译」按钮，系统将校验策略图并生成可执行代码。"),
      target: '[data-testid="toolbar-compile-action"]',
    },
    {
      title: t("回测验证"),
      description:
        t("点击「运行回测」按钮，使用模拟数据验证策略的收益、回撤和胜率。"),
      target: '[data-testid="toolbar-start-backtest-action"]',
    },
    {
      title: t("查看结果"),
      description:
        t("切换到研究面板，查看回测详情、事件流和指标对比。"),
      target: '[data-testid="strategy-research-console"]',
    },
  ];
}

