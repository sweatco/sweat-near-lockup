const nearApi = require("near-api-js");
const BN = require("bn.js");
const fs = require("fs");
const { exec } = require("child_process");
const readline = require("node:readline");

const CONTRACT_ADDRESS = "v2.lockup.sweat.testnet";
const CREDENTIALS_DIR = ".near-credentials";
const TRANSACTIONS_LOG_FILENAME = "transactions.log";
const ACCOUND_IDS_FILE_NAME = "file.txt";
const BATCH_SIZE = 3000;

const myKeyStore = new nearApi.keyStores.UnencryptedFileSystemKeyStore(
  getKeystorePath()
);

function getKeystorePath() {
  const homedir = require("os").homedir();
  return require("path").join(homedir, CREDENTIALS_DIR);
}

async function connect() {
  const connectionConfig = {
    networkId: "testnet",
    keyStore: myKeyStore,
    nodeUrl: "https://rpc.testnet.near.org",
    walletUrl: "https://wallet.testnet.near.org",
    helperUrl: "https://helper.testnet.near.org",
    explorerUrl: "https://explorer.testnet.near.org"
  };

  const connection = await nearApi.connect(connectionConfig);
  return connection.account(CONTRACT_ADDRESS);
}

async function seize(account, ids) {
  return await account.functionCall({
    contractId: CONTRACT_ADDRESS,
    methodName: "seize",
    args: {
      account_ids: ids
    },
    gas: new BN("300000000000000")
  });
}

async function seizeSafely(account, ids) {
  try {
    const contractResponse = await seize(account, ids);

    fs.appendFileSync(
      TRANSACTIONS_LOG_FILENAME,
      `${contractResponse.transaction.hash}\n`
    );
  } catch (e) {
    console.log("Error occurred:", e);
    console.log("Retry...");

    await seizeSafely(account, ids);
  }
}

function hasMoreData() {
  return fs.statSync(ACCOUND_IDS_FILE_NAME).size > 0;
}

async function readFirstNLines(filename, numLines) {
  const fileStream = fs.createReadStream(filename);
  const rl = readline.createInterface({
    input: fileStream,
    crlfDelay: Infinity
  });

  let lineNumber = 0;
  let lines = [];

  for await (const line of rl) {
    lineNumber += 1;
    lines.push(line);

    if (lineNumber === numLines) {
      fileStream.destroy();
      break;
    }
  }

  return lines;
}

async function run() {
  const account = await connect();

  while (hasMoreData()) {
    let ids = await readFirstNLines(ACCOUND_IDS_FILE_NAME, BATCH_SIZE);
    
    console.log(`Running seize for batch ${ids[0]}..${ids[ids.length - 1]}`);
    
    await seizeSafely(account, ids);
    await exec(`sed -i.bu '1,${BATCH_SIZE}d' ${ACCOUND_IDS_FILE_NAME}`);
  }
}

run();
