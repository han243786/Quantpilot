const fs=require('fs');
const path=require('path');
const execSync = require('child_process').execSync;

// Kill old processes
try { execSync('taskkill /f /im quantpilot.exe 2>nul', {stdio:'ignore'}); } catch(e) {}

// v2.3.4: 使用项目根目录相对路径，不再硬编码 D:/rust-js-pr/...
const SRC = path.resolve(__dirname, '..');
const DST = path.resolve(SRC, '..', 'quantpilot-release');

// Remove old
if(fs.existsSync(DST)) { fs.rmSync(DST, {recursive:true, force:true}); console.log('old removed'); }
fs.mkdirSync(DST, {recursive:true});
fs.mkdirSync(DST+'/dist/assets', {recursive:true});
fs.mkdirSync(DST+'/storage/backtests', {recursive:true});

// Copy exe
const exe = SRC+'/target/release/quantpilot.exe';
fs.copyFileSync(exe, DST+'/quantpilot.exe');
console.log('exe copied: '+(fs.statSync(DST+'/quantpilot.exe').size/1024/1024).toFixed(0)+'MB');

// Copy dist
fs.cpSync(SRC+'/frontend/dist', DST+'/dist', {recursive:true});
console.log('frontend copied');

// Generate backtest data
const configs = [
  {label:'sc_a_aggressive', drift:0.0012, vol:0.018, fee:5, trades:25},
  {label:'sc_b_conservative', drift:0.0004, vol:0.010, fee:15, trades:12}
];

for (const cfg of configs) {
  const points=[], benchPoints=[];
  let eq=100000,bench=100000,peak=eq,maxDD=0;
  let seed=cfg.label.charCodeAt(0)*137+cfg.label.charCodeAt(1)*31;
  const rand=()=>{seed=(seed*1664525+1013904223)>>>0;return(seed>>>0)/4294967296;};
  const norm=()=>{let s=0;for(let i=0;i<12;i++)s+=rand();return(s-6)/1.0;};
  for(let i=0;i<300;i++){
    const r=norm();
    let dailyRet=cfg.drift+cfg.vol*r;
    if(rand()<0.15)dailyRet=cfg.drift*3+cfg.vol*2*r+0.01*(rand()-0.5);
    eq*=(1+dailyRet); bench*=(1+cfg.drift*0.5+cfg.vol*0.3*r);
    if(eq>peak)peak=eq; const dd=(peak-eq)/peak; if(dd>maxDD)maxDD=dd;
    points.push({ts_ms:1730000000000+i*86400000, equity:eq, cash_balance:eq*0.2, net_notional:eq*0.8});
    benchPoints.push({ts_ms:1730000000000+i*86400000, equity:bench, cash_balance:bench, net_notional:0});
  }
  const totalReturn=(eq-100000)/100000;
  const dr=[]; for(let i=1;i<points.length;i++)dr.push((points[i].equity-points[i-1].equity)/points[i-1].equity);
  const avgDr=dr.reduce((a,b)=>a+b,0)/dr.length;
  const stdDr=Math.sqrt(dr.reduce((s,v)=>s+Math.pow(v-avgDr,2),0)/(dr.length-1));
  const annVol=stdDr*Math.sqrt(252),annRet=Math.pow(1+totalReturn,365/300)-1;
  const sharpe=annVol>0?annRet/annVol:0;
  const downDr=dr.filter(r=>r<0);
  const downVol=downDr.length>0?Math.sqrt(downDr.reduce((s,v)=>s+v*v,0)/downDr.length)*Math.sqrt(252):0;
  const sortino=downVol>0?annRet/downVol:0,calmar=maxDD>0?annRet/maxDD:0;

  // Month data
  const monthData=[]; let mStart=0,mStartIdx=0;
  for(let i=1;i<points.length;i++){
    const dt=new Date(points[i].ts_ms);
    const m=dt.getUTCFullYear()*12+dt.getUTCMonth();
    if(i===1){mStart=m;mStartIdx=0;}
    if(m!==mStart||i===points.length-1){
      monthData.push({period:dt.getUTCFullYear()+'-'+String(dt.getUTCMonth()+1).padStart(2,'0'),return_ratio:points[mStartIdx]?(points[i].equity-points[mStartIdx].equity)/points[mStartIdx].equity:0,trade_count:Math.floor(rand()*10+5)});
      mStart=m;mStartIdx=i;
    }
  }

  // VaR, streaks
  const sorted=[...dr].sort((a,b)=>a-b);
  const varIdx=Math.floor(sorted.length*0.05);
  const var95=sorted[varIdx]||0;
  const cvar95=sorted.slice(0,varIdx+1).reduce((a,b)=>a+b,0)/(varIdx+1);
  let cw=0,cl=0,mw=0,ml=0;
  for(const r of dr){if(r>0){cw++;cl=0;if(cw>mw)mw=cw;}else if(r<0){cl++;cw=0;if(cl>ml)ml=cl;}}

  const tradeCount=Math.floor(300*(cfg.trades/30)*(1+cfg.vol*50));
  const winCount=Math.floor(tradeCount*0.55);
  const dig={algorithm:'sha256_canonical_json',value:''};
  const sum={step_count:300,trade_count:tradeCount,total_return_ratio:totalReturn,final_equity:eq,net_profit:eq-100000,win_rate:winCount/tradeCount,annualized_return:annRet,annualized_volatility:annVol,risk_adjusted:{sharpe_ratio:sharpe,sortino_ratio:sortino,calmar_ratio:calmar,var_95:var95,cvar_95:cvar95},trade_analysis:{profit_factor:dr.filter(r=>r>0).reduce((a,b)=>a+b,0)/Math.max(0.001,Math.abs(dr.filter(r=>r<0).reduce((a,b)=>a+b,0))),avg_win:dr.filter(r=>r>0).reduce((a,b)=>a+b,0)/Math.max(1,dr.filter(r=>r>0).length),avg_loss:Math.abs(dr.filter(r=>r<0).reduce((a,b)=>a+b,0))/Math.max(1,dr.filter(r=>r<0).length),max_consecutive_wins:mw,max_consecutive_losses:ml},drawdown_analysis:{max_drawdown_ratio:maxDD,max_drawdown_duration_days:Math.floor(maxDD*500),avg_drawdown_duration_days:Math.floor(maxDD*200)},benchmark_comparison:{benchmark_total_return:(bench-100000)/100000,alpha:annRet-(bench-100000)/100000,beta:1.2,information_ratio:sharpe*0.8},skewness:dr.reduce((s,v)=>s+Math.pow((v-avgDr)/stdDr,3),0)*dr.length/((dr.length-1)*(dr.length-2)),kurtosis:dr.reduce((s,v)=>s+Math.pow((v-avgDr)/stdDr,4),0)*dr.length/((dr.length-1)*(dr.length-2))-3};
  const exec={initial_cash_balance:100000,taker_fee_bps:cfg.fee,default_slippage_bps:cfg.fee<10?3:8,total_cost_buffer_bps:cfg.fee*2,time_in_force:'Gtc',allow_partial_fills:false,latency_assumption_ms:null};
  const gov={schema_version:'quantpilot/runtime-governance/v1',governance_source:'legacy_default',capability_hash:'sha256:screenshot_test',strategy_version:'v1',parameter_version:'v1',deployment_revision:'v1',capability_api_version:'quantpilot/capabilities/v1',runtime_support_boundary:{runtime_modes:['paper'],execution_module_keys:['builtin.execution.paper']},indicator_kinds:['ma_cross','rsi','macd','momentum','spread','z_score'],attested_at_ms:1730000000000,attestation_signature:'',permission_boundary:{model_version:'quantpilot/permission-boundary/v1',execution_owner_module:'builtin.execution.paper',live_execution_allowed:false,ai_write_policy:'proposal_only',plugin_network_default:'deny',non_execution_order_access:'deny'}};
  const manifest={schema_version:'quantpilot/reproducibility-manifest/v1',manifest_id:'manifest_'+cfg.label,backtest_id:cfg.label,graph_id:'graph_'+cfg.label,compile_id:'compile_'+cfg.label,created_at_ms:1730000000000,protocol_name:'Strategy '+cfg.label,config_hash:'hash_'+cfg.label,account:{equity_estimate:eq,cash_balance:eq*0.2,available_cash_balance:eq*0.18,frozen_cash_balance:eq*0.02,total_leverage:0,total_gross_notional:0,total_net_notional:0,positions:1,open_order_count:0,open_orders:[]},summary:sum,backtest_spec:{schema_version:'quantpilot/backtest-spec/v1',backtest_id:cfg.label,replay_source:'deterministic_mock',requested_at_ms:1730000000000,run_spec:{schema_version:'quantpilot/run-spec/v1',run_mode:'backtest',graph_id:'graph_'+cfg.label,compile_id:'compile_'+cfg.label,runtime_mode:'backtest',protocol_name:'Strategy '+cfg.label,config_hash:'hash_'+cfg.label,datasets:[],execution_assumptions:exec,core_ir_digest:dig},market_data_snapshot:{snapshot_id:'snap_'+cfg.label,replay_source:'deterministic_mock',captured_at_ms:1730000000000,datasets:[],quotes:[],klines:[]}},compile_artifacts:null,governance:gov,output_artifacts:[],backtest_output_digest:dig};

  const dir=DST+'/storage/backtests/'+cfg.label;
  fs.mkdirSync(dir,{recursive:true});
  fs.writeFileSync(dir+'/manifest.json',JSON.stringify(manifest,null,2));
  fs.writeFileSync(dir+'/backtest_output.json',JSON.stringify({mode:'historical_replay',started_at_ms:1730000000000,ended_at_ms:1730000000000+300*86400000,sessions:[],equity_curve:points,benchmark_equity_curve:benchPoints,period_returns:monthData,summary:sum,final_portfolio:{cash_balance:eq*0.2,available_cash_balance:eq*0.18,frozen_cash_balance:eq*0.02,total_leverage:0,total_gross_notional:0,total_net_notional:0,positions:[],open_orders:[],open_order_count:0},debug_values:null},null,2));
  fs.writeFileSync(dir+'/event_log.json',JSON.stringify({schema_version:'v1',artifact_id:'',backtest_id:cfg.label,digest:dig,event_count:tradeCount*5,events:[]}));
  fs.writeFileSync(dir+'/trade_ledger.json',JSON.stringify({schema_version:'v1',artifact_id:'',backtest_id:cfg.label,digest:dig,trades:[],trade_count:tradeCount,summary:null}));
  fs.writeFileSync(dir+'/equity_curve.json',JSON.stringify({schema_version:'v1',artifact_id:'',backtest_id:cfg.label,digest:dig,points:points,point_count:points.length}));
  fs.writeFileSync(dir+'/metrics.json',JSON.stringify({schema_version:'v1',artifact_id:'',backtest_id:cfg.label,digest:dig,summary:sum,event_count:tradeCount*5,session_count:300,started_at_ms:1730000000000,ended_at_ms:1730000000000+300*86400000,final_account:{cash_balance:eq*0.2,available_cash_balance:eq*0.18,frozen_cash_balance:eq*0.02,total_leverage:0,total_gross_notional:0,total_net_notional:0,positions:1,open_order_count:0,open_orders:[]},execution_assumptions:null}));

  console.log(cfg.label+': return='+(totalReturn*100).toFixed(1)+'% sharpe='+sharpe.toFixed(2)+' trades='+tradeCount);
}

// Create bat
fs.writeFileSync(DST+'/启动.bat', '@echo off\r\ncd /d "%~dp0"\r\nset QUANTPILOT_DEV=true\r\necho QuantPilot v1.1.1\r\nstart "qp" quantpilot.exe\r\nchoice /t 8 /d y /n >nul 2>&1\r\nstart http://127.0.0.1:3000\r\necho.\r\necho Server: http://127.0.0.1:3000\r\necho Backtest A: http://127.0.0.1:3000/backtests/sc_a_aggressive\r\necho Backtest B: http://127.0.0.1:3000/backtests/sc_b_conservative\r\necho Compare: http://127.0.0.1:3000/backtests/compare?ids=sc_a_aggressive,sc_b_conservative\r\necho.\r\npause\r\ntaskkill /f /im quantpilot.exe >nul 2>&1\r\n');
console.log('bat created');
console.log('DONE: ' + DST);
