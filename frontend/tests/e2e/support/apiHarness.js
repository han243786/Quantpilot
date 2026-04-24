import { expect } from "@playwright/test";

function jsonResponse(body, status = 200) {
  return {
    status,
    contentType: "application/json; charset=utf-8",
    body: JSON.stringify(body)
  };
}

export async function createApiMockHarness(page) {
  const unexpectedRequests = [];
  const expectedPatterns = [];
  let guardEnabled = false;
  let guardRouteInstalled = false;

  function patternToRegex(pattern) {
    if (pattern instanceof RegExp) return pattern;
    const escaped = String(pattern)
      .replace(/[.+^${}()|[\]\\]/g, "\\$&")
      .replace(/\*\*/g, "___DOUBLE_WILDCARD___")
      .replace(/\*/g, "[^/]+")
      .replace(/___DOUBLE_WILDCARD___/g, ".*");
    return new RegExp(`^${escaped}$`);
  }

  function rememberPattern(pattern) {
    expectedPatterns.push(patternToRegex(pattern));
  }

  return {
    async fulfill(pattern, response) {
      rememberPattern(pattern);
      await page.route(pattern, async (route) => {
        await route.fulfill(response);
      });
    },

    async json(pattern, body, status = 200) {
      rememberPattern(pattern);
      await page.route(pattern, async (route) => {
        await route.fulfill(jsonResponse(body, status));
      });
    },

    async text(pattern, body, status = 200, contentType = "text/plain; charset=utf-8") {
      rememberPattern(pattern);
      await page.route(pattern, async (route) => {
        await route.fulfill({
          status,
          contentType,
          body
        });
      });
    },

    async handle(pattern, handler) {
      rememberPattern(pattern);
      await page.route(pattern, handler);
    },

    async installGuard() {
      if (guardEnabled) return;
      guardEnabled = true;
      if (!guardRouteInstalled) {
        guardRouteInstalled = true;
        await page.route("**/api/**", async (route) => {
          const url = route.request().url();
          const isExpected = expectedPatterns.some((pattern) => pattern.test(url));
          if (isExpected) {
            await route.fallback();
            return;
          }

          const pathname = new URL(url).pathname;
          unexpectedRequests.push(`${route.request().method()} ${pathname}`);
          await route.fulfill(jsonResponse({
            error: "unexpected_e2e_request",
            message: `Unexpected unmocked API request: ${pathname}`
          }, 501));
        });
      }
      page.on("request", (request) => {
        const url = request.url();
        if (!url.includes("/api/")) return;
        const isExpected = expectedPatterns.some((pattern) => pattern.test(url));
        if (isExpected) return;

        const pathname = new URL(url).pathname;
        unexpectedRequests.push(`${request.method()} ${pathname}`);
      });
    },

    expectNoUnexpectedApiRequests() {
      expect(
        unexpectedRequests,
        `Unexpected unmocked API requests:\n${unexpectedRequests.join("\n")}`
      ).toEqual([]);
    }
  };
}
