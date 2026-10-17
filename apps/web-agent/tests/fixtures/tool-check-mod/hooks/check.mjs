export function register(on) {
  on("tool.check", async ($, e, next) => {
    const result = await next(e);
    if (e.tool_name === "read_file_run") {
      return { block: true, reason: "fixture_block" };
    }
    return result;
  });
}
