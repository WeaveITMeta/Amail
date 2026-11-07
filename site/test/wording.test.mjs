import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { FOOTER_CLAIM, PLATFORM_LINE, TAGLINE, WAITLIST_NOTICE } from "../copy.mjs";
import { buildWaitlistPayload } from "../waitlist.mjs";

const read = (path) => readFileSync(new URL(path, import.meta.url), "utf8");

const indexHtml = read("../index.html");
const mcpHtml = read("../mcp/index.html");
const readme = read("../../README.md");
const docsIndex = read("../../docs/index.md");
const start = read("../../START.md");
const appJs = read("../app.js");

const publicCopies = [
  ["site/index.html", indexHtml],
  ["site/mcp/index.html", mcpHtml],
  ["README.md", readme],
  ["docs/index.md", docsIndex],
  ["START.md", start],
];

test("share card and footer stay exact", () => {
  assert.match(indexHtml, /<title>A robot for every inbox on your domain\.<\/title>/);
  assert.match(indexHtml, /property="og:title" content="A robot for every inbox on your domain\."/);
  assert.match(indexHtml, /name="twitter:title" content="A robot for every inbox on your domain\."/);
  assert.match(indexHtml, /<h1>A robot for every inbox on your domain\.<\/h1>/);
  assert.equal(indexHtml.includes("Amail. A robot"), false);
  assert.equal(indexHtml.includes("Amail: a robot"), false);
  assert.equal([...indexHtml.matchAll(new RegExp(FOOTER_CLAIM, "g"))].length, 1);
  assert.equal([...mcpHtml.matchAll(new RegExp(FOOTER_CLAIM, "g"))].length, 1);
  assert.equal(TAGLINE, "A robot for every inbox on your domain.");
});

test("platform wording is universal and does not claim binaries or test runs", () => {
  for (const [name, text] of publicCopies) {
    assert.ok(text.includes(PLATFORM_LINE), `${name} missing platform line`);
    assert.equal(/linux-only|only linux|linux only/i.test(text), false, name);
    assert.equal(/grok bot only|only grok|exclusively grok|exclusive to grok/i.test(text), false, name);
    assert.equal(/prebuilt installer/i.test(text), false, name);
    assert.equal(/tested binary|platform binaries|tested on windows|tested on macos|tested on linux/i.test(text), false, name);
  }
  assert.match(PLATFORM_LINE, /Windows, macOS, and Linux/);
  assert.match(PLATFORM_LINE, /NOVA, Grok Bot, and any compatible client/);
});

test("waitlist notice separates the invitation from marketing and avoids invented contacts", () => {
  assert.equal([...indexHtml.matchAll(new RegExp(WAITLIST_NOTICE.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "g"))].length, 2);
  assert.match(WAITLIST_NOTICE, /early-access invitation/);
  assert.match(WAITLIST_NOTICE, /Role and company are sent only if you fill them in/);
  assert.match(WAITLIST_NOTICE, /not a separate product newsletter/);
  assert.match(WAITLIST_NOTICE, /does not opt you into one/);
  assert.match(WAITLIST_NOTICE, /Reply to the invitation/);
  assert.match(WAITLIST_NOTICE, /does not set a retention date/);
  assert.equal(/mailto:|privacy@|unsubscribe@/i.test(indexHtml), false);
  assert.equal(/mailto:|privacy@|unsubscribe@/i.test(read("../waitlist.mjs")), false);
  assert.equal(/never sold|Nothing else/i.test(indexHtml), false);
});

test("signup payload is invitation-only even if a caller asks for marketing", () => {
  const payload = buildWaitlistPayload({
    email: "  person@example.com ",
    role: " support ",
    company: "",
    website: "",
    source: "hero",
    marketing_opt_in: true,
  });
  assert.deepEqual(payload, {
    email: "person@example.com",
    role: "support",
    company: "",
    website: "",
    source: "hero",
    purpose: "early_access_invitation",
    marketing_opt_in: false,
  });
  assert.match(appJs, /buildWaitlistPayload/);
  assert.equal(/\/api\/waitlist/.test(appJs), true);
  assert.equal(appJs.includes("marketing_opt_in: true"), false);
});

test("human review stays a configurable gate", () => {
  assert.match(mcpHtml, /checkpoint you configure/);
  assert.match(mcpHtml, /A gate you can set/);
  assert.match(mcpHtml, /Leave the gate unset and that step completes with the rest of the pipeline/);
});
