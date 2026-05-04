# ═══ 压力测试5：编译应失败 — 缺少策略函数 ═══
# 故意不包含 fn strategy() — 验证系统正确处理编译失败

import math

@test {
    name: "压力测试5：编译失败 — 缺失策略函数"
    cover: ["STRESS-FAIL-001"]
}

@step("编译应失败") {
    @compile
    @assert compile.compilable == true
}
