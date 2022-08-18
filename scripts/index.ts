import { getData, SigningCosmWasmClient } from "./signer";
const { ADDR, CONTR, getAliceClient } = getData(true);

const l = console.log.bind(console);

async function main() {
  const aliceClient = (await getAliceClient(true)) as SigningCosmWasmClient;
  const gas = {
    amount: [{ denom: "ujunox", amount: "625" }],
    gas: "250000",
  };

  const query = async () => {
    let res = await aliceClient.queryContractSmart(CONTR.ADDR, {
      get_count: {},
    });
    l("\n", res, "\n");
  };

  let res;

  await query();

  res = await aliceClient.execute(
    ADDR.ALICE,
    CONTR.ADDR,
    { increment: {} },
    gas
  );
  l({ attributes: res.logs[0].events[2].attributes }, "\n");

  await query();

  res = await aliceClient.execute(
    ADDR.ALICE,
    CONTR.ADDR,
    { set: { count: 200 } },
    gas
  );
  l({ attributes: res.logs[0].events[2].attributes }, "\n");

  await query();
}

main();
