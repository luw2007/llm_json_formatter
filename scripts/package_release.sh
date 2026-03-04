#!/usr/bin/env bash
set -euo pipefail

version="${1:-}"
if [[ -z "${version}" ]]; then
  version="$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys;print(json.load(sys.stdin)["packages"][0]["version"])')"
fi

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${root_dir}"

dist_dir="${root_dir}/dist"
rm -rf "${dist_dir}"
mkdir -p "${dist_dir}"

package_tar() {
  local target="$1"
  local bin_path="$2"

  local out_name="jf-v${version}-${target}.tar.gz"
  local tmp_dir
  tmp_dir="$(mktemp -d "${dist_dir}/tmp.XXXXXX")"
  cp "${bin_path}" "${tmp_dir}/jf"
  (cd "${tmp_dir}" && tar -czf "${dist_dir}/${out_name}" "jf")
  rm -rf "${tmp_dir}"
}

package_zip() {
  local target="$1"
  local bin_path="$2"

  local out_name="jf-v${version}-${target}.zip"
  local tmp_dir
  tmp_dir="$(mktemp -d "${dist_dir}/tmp.XXXXXX")"
  cp "${bin_path}" "${tmp_dir}/jf.exe"
  (cd "${tmp_dir}" && zip -qr "${dist_dir}/${out_name}" "jf.exe")
  rm -rf "${tmp_dir}"
}

host_target="$(rustc -vV | awk -F': ' '/^host: /{print $2}')"
if [[ -f "target/release/jf" && ! -f "target/${host_target}/release/jf" ]]; then
  package_tar "${host_target}" "target/release/jf"
fi

if [[ -f "target/release/jf.exe" && ! -f "target/${host_target}/release/jf.exe" ]]; then
  package_zip "${host_target}" "target/release/jf.exe"
fi

if [[ -f "target/aarch64-apple-darwin/release/jf" ]]; then
  package_tar "aarch64-apple-darwin" "target/aarch64-apple-darwin/release/jf"
fi

if [[ -f "target/x86_64-apple-darwin/release/jf" ]]; then
  package_tar "x86_64-apple-darwin" "target/x86_64-apple-darwin/release/jf"
fi

if [[ -f "target/aarch64-unknown-linux-gnu/release/jf" ]]; then
  package_tar "aarch64-unknown-linux-gnu" "target/aarch64-unknown-linux-gnu/release/jf"
fi

if [[ -f "target/x86_64-unknown-linux-gnu/release/jf" ]]; then
  package_tar "x86_64-unknown-linux-gnu" "target/x86_64-unknown-linux-gnu/release/jf"
fi

if [[ -f "target/x86_64-pc-windows-msvc/release/jf.exe" ]]; then
  package_zip "x86_64-pc-windows-msvc" "target/x86_64-pc-windows-msvc/release/jf.exe"
fi

shopt -s nullglob
artifacts=(
  "${dist_dir}/jf-v${version}-"*.tar.gz
  "${dist_dir}/jf-v${version}-"*.zip
)

if [[ ${#artifacts[@]} -eq 0 ]]; then
  echo "no artifacts found in ${dist_dir}; build binaries first" >&2
  exit 1
fi

(cd "${dist_dir}" && ls -1)
(cd "${dist_dir}" && shasum -a 256 jf-v"${version}"-*.tar.gz jf-v"${version}"-*.zip > SHA256SUMS)
