export function register(on) {
  on("turn.complete", async ($, e, next) => {
    await $.agents.get();
    return next(e);
  });

  on("ui.render", { component: "Sidebar" }, ($, e, next) => next(e));
}
