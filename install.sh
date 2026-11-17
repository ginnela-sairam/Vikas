#!/bin/sh
set -eu

repo="ginnela-sairam/Vikas"
latest_url=$(curl --proto '=https' --tlsv1.2 -fsSL -o /dev/null -w '%{url_effective}' "https://github.com/$repo/releases/latest")
tag=${latest_url##*/}

case "$tag" in
    v[0-9]*) ;;
    *)
        echo "Could not determine the latest VIKAS release tag." >&2
        exit 1
        ;;
esac
case "$tag" in
    *[!A-Za-z0-9._-]*)
        echo "The latest VIKAS release tag contains unsupported characters." >&2
        exit 1
        ;;
esac

os=$(uname -s)
arch=$(uname -m)
case "$os:$arch" in
    Linux:x86_64) platform="linux-x86_64" ;;
    Darwin:arm64) platform="macos-arm64" ;;
    *)
        echo "Prebuilt VIKAS binaries are currently available for Linux x86_64 and macOS arm64." >&2
        exit 1
        ;;
esac

archive="vikas-$tag-$platform.tar.gz"
download_url="https://github.com/$repo/releases/download/$tag/$archive"
tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM

curl --proto '=https' --tlsv1.2 -fsSL "$download_url" -o "$tmp_dir/$archive"
tar -xzf "$tmp_dir/$archive" -C "$tmp_dir" ./vikas

install_dir="$HOME/.local/bin"
mkdir -p "$install_dir"
install -m 755 "$tmp_dir/vikas" "$install_dir/vikas"
"$install_dir/vikas" version
echo "Installed VIKAS to $install_dir/vikas"
case ":$PATH:" in
    *":$install_dir:"*) ;;
    *) echo "Add $install_dir to your PATH to run vikas from any directory." ;;
esac
