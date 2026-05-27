import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import StrategyConfigCockpit from "./StrategyConfigCockpit";
import { apiClient } from "../api/client";

vi.mock("../api/client", async (importOriginal) => {
  const actual = await importOriginal();
  return {
    ...actual,
    apiClient: {
      ...actual.apiClient,
      post: vi.fn()
    }
  };
});

describe("StrategyConfigCockpit", () => {
  beforeEach(() => {
    apiClient.post.mockReset();
    window.localStorage.clear();
    Object.defineProperty(URL, "createObjectURL", {
      configurable: true,
      value: vi.fn(() => "blob:strategy-config")
    });
    Object.defineProperty(URL, "revokeObjectURL", {
      configurable: true,
      value: vi.fn()
    });
  });

  it("renders preflight status and keeps real-funds execution closed", async () => {
    apiClient.post.mockResolvedValue({
      schema_version: "quantpilot/v4-strategy-config-preflight/v1",
      artifact: {
        artifact_id: "artifact_alpha",
        strategy_id: "alpha_strategy",
        artifact_digest: "0123456789abcdef0123456789abcdef",
        runtime_boundary: {
          mode_label: "PaperSimulated",
          live_execution_allowed: false,
          execution_capability_sources: ["runtime_simulated"],
          rejection_reasons: []
        },
        config_domains: [
          {
            domain_id: "state_machine",
            lifecycle: "implemented",
            readiness: "ready",
            source_refs: [
              {
                source_kind: "v4_graph",
                source_id: "machine_alpha",
                digest: "sha256:state_machine_digest_1"
              }
            ],
            findings: []
          },
          {
            domain_id: "risk",
            lifecycle: "implemented",
            readiness: "restricted",
            source_refs: [
              {
                source_kind: "v4_graph",
                source_id: "risk_plane_alpha",
                digest: "sha256:risk_digest_1"
              }
            ],
            findings: [
              {
                severity: "warning",
                code: "risk_guard_missing",
                message: "Risk guard needs review"
              }
            ],
            primary_action: "preflight"
          }
        ],
        evidence_anchors: [
          {
            anchor_type: "compile",
            anchor_id: "compile_1",
            digest: "compile_hash_1",
            summary: "runtime compile"
          },
          {
            anchor_type: "proposal",
            anchor_id: "ai_proposal_1",
            digest: "sha256:after_digest_1",
            summary: "risk"
          }
        ],
        proposal_bindings: [
          {
            proposal_id: "ai_proposal_1",
            target_domain: "risk",
            before_digest: "sha256:before_digest_1",
            after_digest: "sha256:after_digest_1",
            evidence_anchor_ids: ["compile_1"],
            sandbox_status: "passed",
            approval_status: "static_check_passed"
          }
        ]
      },
      decision: "ready",
      can_compile: true,
      can_paper_simulated: true,
      can_backtest: true,
      can_paper_actual_demo: false,
      can_live_execution: true,
      findings: []
    });

    const graph = {
      metadata: {
        graph_id: "alpha_strategy",
        version: "draft",
        updated_at: 1_700_000_000_000
      },
      nodes: [],
      edges: []
    };

    render(
      <StrategyConfigCockpit
        graph={graph}
        runtime={{
          runId: "run_1",
          runKind: "PaperSimulated",
          aiProposalState: {
            proposals: [
              {
                ai_proposal_id: "ai_proposal_1",
                status: "static_check_passed",
                config_domain_binding: {
                  target_domain: "risk",
                  before_digest: "sha256:before_digest_1",
                  after_digest: "sha256:after_digest_1",
                  evidence_anchor_ids: ["compile_1"]
                },
                sandbox_status: "passed"
              }
            ]
          }
        }}
        compileSummary={{ config_hash: "compile_hash_1", last_compile_id: "compile_1" }}
      />
    );

    await waitFor(() =>
      expect(apiClient.post).toHaveBeenCalledWith(
        "/v1/strategy-config/preflight",
        expect.objectContaining({
          strategy_id: "alpha_strategy",
          runtime_mode: "PaperSimulated",
          capability_snapshot_hash: expect.stringMatching(/^(sha256:[0-9a-f]{64}|safe-fallback)$/),
          capability_source: "frontend_snapshot",
          proposal_bindings: [
            expect.objectContaining({
              proposal_id: "ai_proposal_1",
              target_domain: "risk",
              before_digest: "sha256:before_digest_1",
              after_digest: "sha256:after_digest_1",
              approval_status: "static_check_passed"
            })
          ],
          evidence_anchors: expect.arrayContaining([
            expect.objectContaining({ anchor_type: "compile", anchor_id: "compile_1" }),
            expect.objectContaining({ anchor_type: "proposal", anchor_id: "ai_proposal_1" })
          ]),
          required_execution_capability_sources: ["runtime_simulated"]
        })
      )
    );

    expect(await screen.findByText("可继续")).toBeInTheDocument();
    expect(screen.getAllByText("状态机").length).toBeGreaterThan(0);
    expect(screen.getAllByText("已就绪").length).toBeGreaterThan(0);
    expect(screen.getByText("未开放")).toBeInTheDocument();
    expect(screen.getByTestId("strategy-config-domain-rail")).toHaveTextContent("状态机");
    expect(screen.getByTestId("strategy-config-domain-panel")).toHaveTextContent("machine_alpha");
    fireEvent.click(screen.getByTestId("strategy-config-domain-risk"));
    expect(screen.getByTestId("strategy-config-domain-panel")).toHaveTextContent("Risk Plane");
    expect(screen.getByTestId("strategy-config-domain-panel")).toHaveTextContent("运行前核验");
    expect(screen.getByTestId("strategy-config-domain-sources")).toHaveTextContent("risk_plane_alpha");
    expect(screen.getByTestId("strategy-config-domain-findings")).toHaveTextContent("risk_guard_missing");
    expect(screen.getByTestId("strategy-config-evidence-anchors")).toHaveTextContent("证据锚点");
    expect(screen.getByTestId("strategy-config-evidence-anchors")).toHaveTextContent("ai_proposal_1");
    expect(screen.getByTestId("strategy-config-proposal-bindings")).toHaveTextContent("AI 提案绑定");
    expect(screen.getByTestId("strategy-config-proposal-bindings")).toHaveTextContent("Risk Plane");
    expect(screen.getByTestId("strategy-config-proposal-bindings")).toHaveTextContent("已通过 / 静态检查通过");
    expect(screen.queryByText("已开放")).not.toBeInTheDocument();
    const clickSpy = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => {});
    fireEvent.click(screen.getByTestId("strategy-config-export-artifact"));
    expect(URL.createObjectURL).toHaveBeenCalledTimes(1);
    expect(clickSpy).toHaveBeenCalledTimes(1);
    clickSpy.mockRestore();
  });

  it("renders domain-level diff against the previous strategy config artifact", async () => {
    const previousArtifact = {
      schema_version: "quantpilot/v4-strategy-config-artifact/v1",
      artifact_id: "artifact_previous",
      artifact_digest: "sha256:previous",
      config_domains: [],
      runtime_boundary: {
        mode_label: "PaperSimulated"
      }
    };
    window.localStorage.setItem(
      "quantpilot.strategy_config.last_artifact.alpha_strategy",
      JSON.stringify(previousArtifact)
    );
    apiClient.post.mockImplementation((path) => {
      if (path === "/v1/strategy-config/preflight") {
        return Promise.resolve({
          schema_version: "quantpilot/v4-strategy-config-preflight/v1",
          artifact: {
            artifact_id: "artifact_current",
            artifact_digest: "sha256:current",
            runtime_boundary: {
              mode_label: "PaperSimulated",
              live_execution_allowed: false,
              execution_capability_sources: ["runtime_simulated"],
              rejection_reasons: []
            },
            config_domains: [
              {
                domain_id: "risk",
                lifecycle: "implemented",
                readiness: "restricted",
                source_refs: [],
                findings: []
              }
            ]
          },
          decision: "restricted",
          can_compile: true,
          can_paper_simulated: true,
          can_backtest: true,
          can_paper_actual_demo: false,
          can_live_execution: false,
          findings: []
        });
      }
      if (path === "/v1/strategy-config/diff") {
        return Promise.resolve({
          schema_version: "quantpilot/v4-strategy-config-diff/v1",
          left_artifact_id: "artifact_previous",
          right_artifact_id: "artifact_current",
          source_digest_changes: [{ field: "qs_digest" }],
          domain_changes: [
            {
              domain_id: "risk",
              lifecycle_changed: false,
              readiness_changed: true,
              source_refs_changed: false,
              findings_changed: true
            }
          ],
          runtime_boundary_changed: false,
          changed: true
        });
      }
      return Promise.reject(new Error(`unexpected path ${path}`));
    });

    render(
      <StrategyConfigCockpit
        graph={{
          metadata: {
            graph_id: "alpha_strategy",
            version: "draft",
            updated_at: 1_700_000_000_001
          },
          nodes: [],
          edges: []
        }}
        runtime={{ runId: "run_2", runKind: "PaperSimulated" }}
        compileSummary={{ config_hash: "compile_hash_2", last_compile_id: "compile_2" }}
      />
    );

    expect(await screen.findByTestId("strategy-config-diff")).toHaveTextContent("配置差异");
    expect(screen.getByTestId("strategy-config-diff")).toHaveTextContent("Risk Plane");
    expect(screen.getByTestId("strategy-config-diff")).toHaveTextContent("就绪状态 / 诊断");
    expect(apiClient.post).toHaveBeenCalledWith(
      "/v1/strategy-config/diff",
      expect.objectContaining({
        left: previousArtifact,
        right: expect.objectContaining({ artifact_id: "artifact_current" })
      })
    );
  });
});
