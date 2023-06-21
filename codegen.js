import  codegen from "@cosmwasm/ts-codegen";

async function main() {
  // NOTE: typing broken?
  await codegen.default({
    contracts: [
      {
        name: "selene",
        dir: "./schema",
      },
    ],
    outPath: "./client/src/contract",

    // options are completely optional ;)
    options: {
      bundle: {
        bundleFile: "index.ts",
        scope: "contracts",
      },
      types: {
        enabled: true,
      },
      client: {
        enabled: true,
      },
      messageComposer: {
        enabled: false,
      },
      msgBuilder: {
        enabled: false,
      },
    },
  });
}

main()
  .then(() => {
    console.log("✨ all done!");
  })
  .catch((e) => {
    console.error(e);
    process.exit(1);
  })
  .finally(() => {
    process.exit(0);
  });
