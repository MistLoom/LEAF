export function register(on) {
  on("session.start", async ($, event, next) => {
    const result = await next(event);
    const id = event?.session_id;
    if (typeof id === "string" && id) {
      await $.session.append_user({ id, text: "cc-hook-live" });
    }
    return result;
  });
}
