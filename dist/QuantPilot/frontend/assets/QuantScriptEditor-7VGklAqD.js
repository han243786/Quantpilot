import{a,j as t}from"./react-vendor-CPjvwER2.js";const m=`fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 50)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}

@test {
    name: "场景一：BTC双均线"
    cover: ["P-03"]
}

@step("编译") {
    @compile
    @assert compile.compilable == true
}

@step("回测") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
}
`;function v(){const[r,o]=a.useState(m),[d,l]=a.useState(!1),[s,c]=a.useState(null),[p,i]=a.useState(null),g=a.useCallback(async()=>{l(!0),i(null),c(null);try{const e=await fetch("/api/test/scenario/run",{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({source:r})});if(e.ok)c(await e.json());else{const n=await e.text();i(`HTTP ${e.status}: ${n.slice(0,200)}`)}}catch(e){i(e.message)}finally{l(!1)}},[r]);return t.jsxs("div",{className:"qs-editor-page",style:{padding:"20px",maxWidth:"1200px",margin:"0 auto"},children:[t.jsxs("div",{style:{display:"flex",justifyContent:"space-between",alignItems:"center",marginBottom:"16px"},children:[t.jsx("h1",{style:{margin:0},children:"QuantScript 编辑器"}),t.jsxs("div",{style:{display:"flex",gap:"8px"},children:[t.jsx("button",{className:"ghost-btn",onClick:()=>o(m),disabled:d,children:"重置示例"}),t.jsx("button",{className:"primary-btn",onClick:g,disabled:d,"data-testid":"qs-editor-run",children:d?"运行中...":"▶ 运行测试"})]})]}),t.jsx("textarea",{value:r,onChange:e=>o(e.target.value),"data-testid":"qs-editor-textarea",spellCheck:!1,style:{width:"100%",height:"400px",fontFamily:"Consolas, Monaco, monospace",fontSize:"13px",lineHeight:"1.4",padding:"12px",border:"1px solid var(--ad-border)",borderRadius:"var(--ad-radius-sm)",background:"var(--ad-panel)",color:"var(--ad-text)",resize:"vertical"}}),p&&t.jsx("div",{"data-testid":"qs-editor-error",style:{marginTop:"16px",padding:"12px",background:"var(--ad-error-soft)",border:"1px solid var(--ad-error)",borderRadius:"4px",color:"var(--ad-error)",fontFamily:"monospace"},children:p}),s&&t.jsxs("div",{"data-testid":"qs-editor-report",style:{marginTop:"16px"},children:[t.jsxs("div",{style:{marginBottom:"12px",display:"flex",alignItems:"center",gap:"12px"},children:[t.jsx("h3",{style:{margin:0},children:s.scenario_name}),t.jsxs("span",{style:{padding:"2px 8px",borderRadius:"12px",fontSize:"12px",background:s.failed_count>0?"var(--ad-error)":"var(--ad-success)",color:"white"},children:[s.passed_count,"/",s.steps.length," 通过",s.failed_count>0&&` ${s.failed_count} 失败`,s.skipped_count>0&&` ${s.skipped_count} 跳过`]}),t.jsxs("span",{style:{fontSize:"12px",color:"var(--ad-text-muted)"},children:[s.duration_ms,"ms"]})]}),s.graph_id&&t.jsxs("div",{style:{marginBottom:"12px",fontSize:"12px",color:"var(--ad-text-muted)"},children:["策略已保存: ",t.jsx("code",{children:s.graph_id})]}),t.jsxs("table",{style:{width:"100%",borderCollapse:"collapse",fontSize:"13px"},children:[t.jsx("thead",{children:t.jsxs("tr",{style:{background:"var(--ad-card)"},children:[t.jsx("th",{style:{padding:"8px",textAlign:"left",width:"30px"}}),t.jsx("th",{style:{padding:"8px",textAlign:"left"},children:"步骤"}),t.jsx("th",{style:{padding:"8px",textAlign:"right",width:"70px"},children:"耗时"}),t.jsx("th",{style:{padding:"8px",textAlign:"left"},children:"详情"})]})}),t.jsx("tbody",{children:s.steps.map((e,n)=>{var x,u,h;const y=e.status==="passed"?"✓":e.status==="failed"?"✗":"⊘",f=e.status==="passed"?"var(--ad-success)":e.status==="failed"?"var(--ad-error)":"var(--ad-text-muted)";return t.jsxs("tr",{style:{borderBottom:"1px solid var(--ad-border)"},children:[t.jsx("td",{style:{padding:"8px",color:f,fontWeight:"bold"},children:y}),t.jsx("td",{style:{padding:"8px"},children:e.name}),t.jsxs("td",{style:{padding:"8px",textAlign:"right",color:"var(--ad-text-muted)"},children:[e.duration_ms,"ms"]}),t.jsxs("td",{style:{padding:"8px",fontSize:"12px",color:"var(--ad-text-secondary)",maxWidth:"500px",overflow:"hidden",textOverflow:"ellipsis"},children:[(x=e.message)==null?void 0:x.slice(0,200),e.status==="failed"&&((u=e.message)==null?void 0:u.includes("actual:"))&&t.jsxs("span",{style:{color:"var(--ad-error)",fontWeight:"bold"},children:[" ","[actual: ",((h=e.message.match(/actual:\s*([^)]+)/))==null?void 0:h[1])||"?","]"]})]})]},n)})})]})]}),t.jsxs("div",{style:{marginTop:"24px",fontSize:"11px",color:"var(--ad-text-muted)",borderTop:"1px solid var(--ad-border)",paddingTop:"12px"},children:[t.jsx("strong",{children:"支持指令"}),": @test @step @compile @run @backtest @assert @save_run @modify @wait @compare_backtests @debug"," | ",t.jsx("strong",{children:"快捷键"}),": Tab 缩进, Ctrl+Enter 运行"]})]})}export{v as default};
