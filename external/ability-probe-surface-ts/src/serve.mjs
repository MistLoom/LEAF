import { serveAbility } from "../../ability-sdk-ts/lib/ability.js";

await serveAbility({
  lanes: async (ctx) => {
    const flags = { middleware: false, pipeline: false, parallel: false, message: false };
    const filter = { class: "ability", name: "lane" };
    await ctx.handle(filter, {
      mode: "middleware",
      async around(ev, next) {
        const out = await next(ev);
        flags.middleware = true;
        return out;
      },
    });
    await ctx.handle(filter, {
      mode: "pipeline",
      async process(ev) {
        flags.pipeline = true;
        return ev;
      },
    });
    await ctx.handle(filter, {
      mode: "parallel",
      async onEvent() {
        flags.parallel = true;
      },
    });
    await ctx.handle(filter, {
      mode: "message",
      async onEvent() {
        flags.message = true;
      },
    });
    await ctx.emit("lane", {});
    for (let i = 0; i < 40 && !flags.message; i++) {
      await ctx.localGet("_drain");
    }
    return { ...flags };
  },
  filter: async (ctx) => {
    const filtered = ctx.withProtocolFilter("fun");
    await filtered.localSet("pf", 1);
    const v = await ctx.localGet("pf");
    return {
      shared_local: v === 1,
      same_instance: filtered.instance === ctx.instance,
    };
  },
  aspect: async (ctx) => {
    ctx.registerAspect({});
    return {};
  },
});
