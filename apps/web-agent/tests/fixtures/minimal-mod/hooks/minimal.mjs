export function register(on) {
  on("session.start", async ($, e, next) => {
    const result = await next(e);
    try {
      await $.session.usage();
    } catch {
      /* 宿主 stub */
    }
    return result;
  });
}
