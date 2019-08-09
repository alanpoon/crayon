SCRIPT_DIR=$(dirname "$0")
BASE_DIR=$SCRIPT_DIR/../../

set -e
cargo build --example $1 --target wasm32-unknown-unknown
rm -rf $SCRIPT_DIR/dist
mkdir $SCRIPT_DIR/dist
mkdir $SCRIPT_DIR/dist/intermediate
cp $BASE_DIR/target/wasm32-unknown-unknown/debug/examples/$1.wasm $SCRIPT_DIR/dist/intermediate/native.wasm
wasm-bindgen $SCRIPT_DIR/dist/intermediate/native.wasm --out-dir $SCRIPT_DIR/dist
mkdir $SCRIPT_DIR/dist/examples
cp -r $SCRIPT_DIR/../../examples/resources $SCRIPT_DIR/dist/examples
cd $SCRIPT_DIR

npm run serve