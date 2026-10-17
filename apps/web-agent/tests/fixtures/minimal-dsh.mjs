export default async function (ctx) {
  ctx.tools.register({
    name: "leaf_echo",
    description: "DSH 测试回显",
    parameters: {
      type: "object",
      properties: { msg: { type: "string" } },
      required: ["msg"],
    },
    execute: async (args) => ({ echo: args.msg ?? "" }),
  });
}
