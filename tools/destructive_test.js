import { readFileSync, writeFileSync } from 'fs';
const B = readFileSync('tests/scenarios/stress_03_error_boundary.qs', 'utf-8');
async function call(src) {
  const r = await fetch('http://127.0.0.1:3000/api/test/scenario/run', {
    method: 'POST', headers: {'Content-Type':'application/json'}, body: JSON.stringify({source: src})
  });
  return { s: r.status, b: await r.text() };
}
let issues = [];

function log(msg) { console.log(msg); }

// ═══ Round A: Extreme Values ═══
log('═══ Round A: Extreme Values ═══\n');

// A1: MAX seed
let src = B.replace('"deterministic_mock"', '"deterministic_mock", seed: 18446744073709551615');
let r = await call(src); let j = JSON.parse(r.b);
log('A1 u64::MAX seed: ' + j.steps[1]?.status + ' ' + (j.steps[1]?.data_snapshot?.backtest_total_fills || 0) + ' fills');

// A2: seed=0
src = B.replace('"deterministic_mock"', '"deterministic_mock", seed: 0');
r = await call(src); j = JSON.parse(r.b);
log('A2 seed=0: ' + j.steps[1]?.status + ' ' + (j.steps[1]?.data_snapshot?.backtest_total_fills || 0) + ' fills');

// A3: duration=MAX
src = B.replace('duration: 5s', 'duration: 99999999s');
r = await call(src); j = JSON.parse(r.b);
let runStep = j.steps.find(s => s.name?.includes('运行'));
log('A3 HUGE duration: ' + runStep?.status + ' | ' + (runStep?.message || '').slice(0,100));

// A4: Negative volatility
src = B.replace('"deterministic_mock"', '"deterministic_mock", volatility: -999');
r = await call(src); j = JSON.parse(r.b);
log('A4 volatility=-999: ' + j.steps[1]?.status + ' | ' + (j.steps[1]?.message || '').slice(0,100));

// A5: lookback=1
src = B.replace('lookback=300', 'lookback=1');
r = await call(src); j = JSON.parse(r.b);
log('A5 lookback=1: ' + j.steps[0]?.status + ' | ' + (j.steps[0]?.message || '').slice(0,100));

// A6: lookback=99999
src = B.replace('lookback=300', 'lookback=99999');
r = await call(src); j = JSON.parse(r.b);
log('A6 lookback=99999: ' + j.steps[0]?.status + ' | ' + (j.steps[0]?.message || '').slice(0,100));

// ═══ Round B: State Machine ═══
log('\n═══ Round B: State Machine Attacks ═══\n');

// B1: Double compile in one step
let b1Src = B.replace(
  '@step("首次编译应成功（最小可用策略）") {\n    @compile\n    @assert compile.compilable == true\n}',
  '@step("dbl") { @compile @compile }'
);
r = await call(b1Src); j = JSON.parse(r.b);
let dbl = j.steps.find(s => s.name === 'dbl');
log('B1 double compile: ' + dbl?.status + ' | ' + (dbl?.message || '').slice(0,100));

// B2: @wait with impossible condition
let b2Src = B.replace(
  '@step("运行最小策略 5秒") {\n    @run { mode: "paper", duration: 5s }\n    @assert run.events.length > 0\n    @assert run.equity > 0\n}',
  '@step("w") { @wait { condition: "run.equity > 999999999", timeout: 1s } }'
);
r = await call(b2Src); j = JSON.parse(r.b);
let w = j.steps.find(s => s.name === 'w');
log('B2 impossible @wait: ' + w?.status + ' | ' + (w?.message || '').slice(0,100));

// B3: @save_run with nothing — use file-based source
let b3Src = readFileSync('tests/scenarios/stress_05_compile_fail.qs', 'utf-8').replace('@step("编译应失败") {\n    @compile\n    @assert compile.compilable == true\n}', '@step("sv") { @save_run }');
r = await call(b3Src); j = JSON.parse(r.b);
log('B3 @save_run no prior: ' + j.steps[0]?.status + ' | ' + (j.steps[0]?.message || '').slice(0,100));

// B4: @compare with invalid indices
let b4Src = B.replace(
  '@step("回测最小策略") {\n    @backtest { source: "deterministic_mock" }\n    @assert backtest.metrics.step_count >= 100\n}',
  '@step("bt") { @backtest { source: "deterministic_mock" } }\n@step("cmp") { @compare_backtests { left: 99, right: 100 } }'
);
r = await call(b4Src); j = JSON.parse(r.b);
let cmp = j.steps.find(s => s.name === 'cmp');
log('B4 @compare OOB: ' + cmp?.status + ' | ' + (cmp?.message || '').slice(0,100));

// ═══ Round C: Concurrency ═══
log('\n═══ Round C: Concurrency ═══\n');

let proms = [];
for (let i = 0; i < 10; i++) {
  let s = B.replace('压力测试3', 'Concurrent_' + i);
  proms.push(call(s));
}
let results = await Promise.all(proms);
let cOk = results.filter(r => r.s === 200).length;
log('C1 10 concurrent different: ' + cOk + '/10 OK');
if (cOk < 10) issues.push('C1: ' + (10-cOk) + ' concurrent failures');

let seqOk = 0;
for (let i = 0; i < 30; i++) {
  r = await call(B);
  if (r.s === 200) seqOk++;
}
log('C2 30 sequential: ' + seqOk + '/30 OK');
if (seqOk < 30) issues.push('C2: ' + (30-seqOk) + ' sequential failures');

// ═══ Round D: Security ═══
log('\n═══ Round D: Security ═══\n');

// D1: Path traversal in scenario name
let d1Src = B.replace('压力测试3', '../../../etc/passwd');
r = await call(d1Src);
log('D1 path traversal name: ' + r.s + ' (OK if not 500)');
if (r.s >= 500) issues.push('D1: server crashed on path traversal');

// D2: RTL unicode
let d2Src = B.replace('压力测试3', '‮test‬');
r = await call(d2Src); j = JSON.parse(r.b);
log('D2 RTL unicode: ' + r.s + ' | scenario: ' + (j.scenario_name || '').slice(0,30));

// D3: Emoji everywhere
let d3Src = B.replace('BTCUSDT', '🪙🪙🪙');
r = await call(d3Src);
log('D3 emoji instrument: ' + r.s);

// D4: Incredibly long cover array
let d4Src = B.replace('["STRESS-ERR-001"]', '["' + Array(500).fill('X').join('') + '"]');
r = await call(d4Src); j = JSON.parse(r.b);
log('D4 500-char cover: ' + r.s + ' | cover[0].length=' + (j.cover?.[0]?.length || 0));

// ═══ Summary ═══
log('\n═══════════════════════════════');
log('Issues found: ' + issues.length);
issues.forEach(i => log(' ⚠ ' + i));
process.exit(issues.length > 0 ? 1 : 0);
