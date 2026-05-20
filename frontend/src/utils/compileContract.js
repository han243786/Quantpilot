export const COMPILE_CONTRACT = {
  order: [
    "strategy_ir semantic preflight",
    "optional formal QuantScript lowering",
    "runtime compile runnable truth"
  ],
  conflictMessage:
    "策略中间表示预检通过并不等于最终可运行。只要运行时编译拒绝输出，就必须以运行时编译结果为准。",
  conflictHint:
    "先看“最终可运行输出遵循”字段，再按 structured diagnostics 修正真正进入运行时编译的工件。",
  runtimeSourceOfTruthLabel: "以 /api/runtime/compile 输出为准"
};

export function compileConflictGuidance() {
  return {
    message: COMPILE_CONTRACT.conflictMessage,
    hint: COMPILE_CONTRACT.conflictHint
  };
}

export function compileConflictSummary({
  strategyIrCheck,
  compileSummary
}) {
  return Boolean(
    strategyIrCheck?.performed &&
      strategyIrCheck?.compilable === true &&
      compileSummary?.compilable === false
  );
}
