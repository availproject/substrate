// Required imports
const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');

const keyring = new Keyring({ type: 'sr25519' });

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function main () {
  // Initialise the provider to connect to the local node
  const provider = new WsProvider('ws://127.0.0.1:9944');

  // Create the API and wait until ready
  const api = await ApiPromise.create({ provider });

  // Retrieve the chain & node information information via rpc calls
  const [chain, nodeName, nodeVersion] = await Promise.all([
    api.rpc.system.chain(),
    api.rpc.system.name(),
    api.rpc.system.version()
  ]);

  console.log(`You are connected to chain ${chain} using ${nodeName} v${nodeVersion}\n`);
  console.log(`WARNING: In this script, finalized does not mean transaction is accepted!\n`);

  // Add our Alice dev account
  const alice = keyring.addFromUri('//Alice', { name: 'Alice default' });

  // Log some info
  // console.log(`${alice.meta.name}: has address ${alice.address} with publicKey [${alice.publicKey}]`);
  
  let counter = 1;
  while (true) {
    let data = "sample_data_" + counter++;
    const unsub = await api.tx.templateModule
    .submitData(data)
    .signAndSend(alice, (result) => {
      console.log(`${data} is ${result.status}`);

      // if (result.status.isInBlock) {
      //   console.log(`${data} included at blockHash ${result.status.asInBlock}`);
      // } else 
      if (result.status.isFinalized) {
        // console.log(`${data} finalized at blockHash ${result.status.asFinalized}`);
        unsub();
      }
    });
    await sleep(6000);
  }

  // Subscribe to the new headers on-chain. The callback is fired when new headers
  // are found, the call itself returns a promise with a subscription that can be
  // used to unsubscribe from the newHead subscription
//   const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
//     console.log(`Chain is at block: #${header.number}`);
//   });


}

main().catch(console.error);