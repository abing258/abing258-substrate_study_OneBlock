import { ApiPromise,WsProvider,Keyring } from "@polkadot/api";

/// Don't use localhost
/// Don't use wss it meas ssl for websocket!
const WS_ADDRESS = "ws://127.0.0.1:9944"

const connectSubstrate = async ()=> {
    const wsProvider = new WsProvider(WS_ADDRESS);
    const api = await ApiPromise.create({ provider: wsProvider,types:{}})
    await api.isReady;
    console.log("connection to substrate is OK.")
    return api;
}
const getConst = async (api:ApiPromise)=> {
    return api.consts.balances.existentialDeposit.toHuman();    
}

const getFreeBalance = async (api:ApiPromise,address: string) => {
    const account = await api.query.system.account(address);
    return account
}

const printBalance = async (api:ApiPromise) => {
    const keyring = new Keyring({type:'sr25519'})
    const alice = keyring.addFromUri('//Alice')
    const bob = keyring.addFromUri('//Bob')
    console.log("alice balance is ",await getFreeBalance(api,alice.address))
    console.log("bob balance is ",await getFreeBalance(api,bob.address))

}

const subscribeAliceAndBob = async (api:ApiPromise) => {
    const keyring = new Keyring({type:'sr25519'})
    const alice = keyring.addFromUri('//Alice')
    await api.query.system.account(alice.address, (aliceAcc: { data: { free: any; }; }) => {
        console.log("Subscribe to Alice account")
        const aliceFreeSub = aliceAcc.data.free;
        console.log(`Alice Account (sub): ${aliceFreeSub}`)
    })
}
const subscribeEvents = async (api:ApiPromise) => {
    // Subscribe to system events via storage
  api.query.system.events((events: any[]) => {
    console.log(`\nReceived ${events.length} events:`);

    // Loop through the Vec<EventRecord>
    events.forEach((record) => {
      // Extract the phase, event and the event types
      const { event, phase } = record;
      const types = event.typeDef;

      // Show what we are busy with
      console.log(`\t${event.section}:${event.method}:: (phase=${phase})`);
      console.log(`\t\t${event.meta.documentation}`);

      // Loop through each of the parameters, displaying the type and data
      event.data.forEach((data: { toString: () => any; }, index: string | number) => {
        console.log(`\t\t\t${types[index].type}: ${data.toString()}`);
      });
    });
  });
}
const sleep = async (time:number) => {
    return new Promise(resolve => setTimeout(resolve, time));
}

const main = async ()=> {
    const api = await connectSubstrate();
    const t = await getConst(api)
    const d = await printBalance(api)
    console.log("connection executed~")
    await subscribeAliceAndBob(api)
    await subscribeEvents(api)
    await sleep(600000)

    console.log("successfuly exit")
}
main().then(()=> {
    console.log("connection succefully!")
    process.exit(0)
}).catch((err)=> {
    console.log("connection failed: " + err)
    process.exit(1)
})