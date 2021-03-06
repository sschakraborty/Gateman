CARGO_LOC=$(which cargo)
RUSTC_LOC=$(which rustc)
CARGO_VERSION=$($CARGO_LOC version)
RUSTC_VERSION=$($RUSTC_LOC --version)

BUILD_MODE="debug"
CARGO_BUILD_FLAG=""
if [ -n "$1" ]; then
  if [ "$1" == "release" ]; then
    BUILD_MODE="release"
  fi
fi
if [ "$BUILD_MODE" == "release" ]; then
  CARGO_BUILD_FLAG="--release"
fi

echo "Cargo found at $CARGO_LOC"
echo "Rust compiler at $RUSTC_LOC"
echo "Cargo version: $CARGO_VERSION"
echo "Rust compiler version: $RUSTC_VERSION"
echo "Build mode: $BUILD_MODE"

BASE_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
echo "Base directory: $BASE_DIR"

/bin/bash -c "cd $BASE_DIR && $CARGO_LOC fmt" > "$BASE_DIR"/build.log 2>&1
/bin/bash -c "cd $BASE_DIR && $CARGO_LOC build $CARGO_BUILD_FLAG" > "$BASE_DIR"/build.log 2>&1

rm -rf "$BASE_DIR"/build
mkdir -p "$BASE_DIR"/build/resources/definitions
mkdir -p "$BASE_DIR"/build/resources/certs/proxy
cp -vf "$BASE_DIR"/target/"$BUILD_MODE"/Gateman "$BASE_DIR"/build/
chmod go-rwx "$BASE_DIR"/build/Gateman
chmod ug+rx "$BASE_DIR"/build/Gateman
cp -rvf "$BASE_DIR"/resources/definitions/api_def "$BASE_DIR"/build/resources/definitions/
cp -rvf "$BASE_DIR"/resources/definitions/origin_def "$BASE_DIR"/build/resources/definitions/
cp -rvf "$BASE_DIR"/resources/config "$BASE_DIR"/build/resources/config
cp -rvf "$BASE_DIR"/resources/scripts/cert/* "$BASE_DIR"/build/resources/certs/proxy

if [ "$BUILD_MODE" == "release" ]; then
  rm -rf "$BASE_DIR"/target
fi
