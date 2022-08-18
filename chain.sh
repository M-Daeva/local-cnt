# script for lauching local juno network

CHAIN_ID="testing"
RPC="http://localhost:26657/"
TXFLAG="--gas-prices 0.1ujunox --gas auto --gas-adjustment 1.3 -y -b block --chain-id $CHAIN_ID --node $RPC"
BINARY="docker exec -i juno-node-1 junod"
DIR=$(pwd)
JUNO_DIR="$DIR/../juno"
DIR_NAME=$(basename "$PWD")
IMAGE_NAME="juno-node-1"
DIR_NAME_SNAKE=$(echo $DIR_NAME | tr '-' '_')
WASM="artifacts/$DIR_NAME_SNAKE.wasm"

waitForChainServe() {
  ADDR="juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y"
  TRIES=0
  echo Waiting for chain serve
  $BINARY query account $ADDR 2> /dev/null
  RESULT=$(echo $?)
  while [ $RESULT = 1 ]
  do
    ((TRIES=$TRIES+1))
    echo -ne "not ready, waiting 1 sec x$TRIES\r"
    sleep 1
    $BINARY query account $ADDR 2> /dev/null
    RESULT=$(echo $?)
  done
  echo
  echo Ready!
}

# build optimized binary if it doesn't exist
if [ ! -f "$WASM" ]; then
  echo "building optimized binary..."
  docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.6
fi

# stop docker container
cd $JUNO_DIR
echo "stopping container..."
docker compose down
# delete docker container
echo "deleting container"
docker rm -f $IMAGE_NAME 2> /dev/null
# build new docker container
echo "starting local network"
STAKE_TOKEN=ujunox UNSAFE_CORS=true docker compose up -d
# move binary to docker container
cd $DIR
docker cp "artifacts/$DIR_NAME_SNAKE.wasm" "$IMAGE_NAME:/$DIR_NAME_SNAKE.wasm"
cd $JUNO_DIR

# wait for chain starting before contract storing
waitForChainServe

# add new users
ALICE_SEED=$(junod keys mnemonic)
echo $ALICE_SEED | $BINARY keys add alice --recover
ALICE_ADDR=$($BINARY keys show alice --address)

BOB_SEED=$(junod keys mnemonic)
echo $BOB_SEED | $BINARY keys add bob --recover
BOB_ADDR=$($BINARY keys show bob --address)

# send them some coins
VALIDATOR_ADDR=$($BINARY keys show validator --address)
$BINARY tx bank send $VALIDATOR_ADDR $ALICE_ADDR "250000000ujunox" --from $VALIDATOR_ADDR --yes --broadcast-mode block --sign-mode direct --chain-id $CHAIN_ID
$BINARY tx bank send $VALIDATOR_ADDR $BOB_ADDR "250000000ujunox" --from $VALIDATOR_ADDR --yes --broadcast-mode block --sign-mode direct --chain-id $CHAIN_ID

CONTRACT_CODE=$($BINARY tx wasm store "/$DIR_NAME_SNAKE.wasm" --from validator $TXFLAG --output json | jq -r '.logs[0].events[-1].attributes[0].value')

#---------- SMART CONTRACT INTERACTION ------------------------

# instantiate smart contract
INIT='{"count":42}'
$BINARY tx wasm instantiate $CONTRACT_CODE "$INIT" --from "alice" --label "my first contract" $TXFLAG --no-admin

# get smart contract address
CONTRACT_ADDRESS=$($BINARY query wasm list-contract-by-code $CONTRACT_CODE --output json | jq -r '.contracts[-1]')

# write data to file
cd $DIR/scripts
R="{\"ALICE_SEED\":\"$ALICE_SEED\",\"BOB_SEED\":\"$BOB_SEED\",\"ALICE_ADDR\":\"$ALICE_ADDR\",\"BOB_ADDR\":\"$BOB_ADDR\",\"CONTRACT_ADDRESS\":\"$CONTRACT_ADDRESS\",\"CONTRACT_CODE\":\"$CONTRACT_CODE\"}"
echo $R > chain_data.json

cd $DIR
