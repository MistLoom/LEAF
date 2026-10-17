export function register(on) {
  on("tool.check", async ($, e, next) => next(e));

  on("tool.call", async ($, e, next) => {
    await $.session.usage();
    return next(e);
  });

  on("session.start", async ($, e, next) => {
    await $.session.cwd();
    return next(e);
  });
}
