CARGO_LOC=$(which cargo)
RUSTC_LOC=$(which rustc)
CARGO_VERSION=$($CARGO_LOC version)
RUSTC_VERSION=$($RUSTC_LOC --version)

echo "Cargo found at $CARGO_LOC"
echo "Rust compiler at $RUSTC_LOC"
echo "Cargo version: $CARGO_VERSION"
echo "Rust compiler version: $RUSTC_VERSION"

BASE_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
echo "Base directory: $BASE_DIR"

/bin/bash -c "cd $BASE_DIR && $CARGO_LOC fmt" > "$BASE_DIR"/build.log 2>&1
/bin/bash -c "cd $BASE_DIR && $CARGO_LOC build" > "$BASE_DIR"/build.log 2>&1
