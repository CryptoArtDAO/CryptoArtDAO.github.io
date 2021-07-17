#!/usr/bin/env bash

set -o errexit

# cargo install wasm-snip wasm-gc
# apt-get install -y binaryen wabt
# https://github.com/near/near-sdk-rs/tree/master/minifier
minify() {
  filePath="${1}"
  fileName=$(basename -- "${filePath}")
  dirPath=$(dirname -- "${filePath}")
  tmpPath="${dirPath}/temp-${fileName}"
  outFileName="${fileName%.*}-minified.${fileName##*.}"
  outPath="${dirPath}/${outFileName}"
  wasm-snip \
    --snip-rust-fmt-code \
    --snip-rust-panicking-code \
    --pattern core::num::flt2dec::.* \
    --pattern core::fmt::float::.* \
    --output "${tmpPath}" \
    "${filePath}"
  wasm-gc "${tmpPath}"
  wasm-strip "${tmpPath}"
  wasm-opt -Oz "${tmpPath}" --output "${outPath}"
  rm "${tmpPath}"
  fileSize=$(stat -c "%s" "${filePath}")
  outSize=$(stat -c "%s" "${outPath}")
  echo "Minifying ${fileName} ${fileSize} bytes -> ${outSize} bytes, see ${outFileName}"
}

build() {
  package="${1}"
  cargo build --package "${package}" --target wasm32-unknown-unknown --release
  mkdir -p build
  cp target/wasm32-unknown-unknown/release/*.wasm build/
  # shellcheck disable=SC2002
  cat "src/contract/${package}/${package}.svg" | npx svgo -i - -o - | base64 -w0 | printf 'data:image/svg+xml;base64,%s' "$(cat)" >"build/${package}.icon"
  printf '<html><body><img src="%s"></body></html>' "$(cat "build/${package}.icon")" >"build/${package}.html"
  minify "build/${package}.wasm"
  echo "Build size:"
  du -b build/*"${package}"*
}

build society
