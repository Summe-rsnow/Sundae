// The Rust root tested from TypeScript: `nsh test` runs this file in a
// headless runner it builds with the crate, and `open(".")` starts the root
// (`nsh.json` "root").
import { expect, open, test } from "nsh/test";

test("opens Sundae shell, About dialog, and Config view", async () => {
  const app = await open(".", { width: 1080, height: 720, platform: "windows" });
  expect(app.texts()).toContain("Sundae");
  expect(app.texts()).toContain("v1.2.0");

  await app.press("label:关于 Sundae");
  expect(app.texts()).toContain("版本更新");
  expect(app.texts()).toContain("v1.2.0");
  await app.press("关闭");

  await app.press("label:选项配置");
  expect(app.texts()).toContain("Scoop 选项配置");
});
