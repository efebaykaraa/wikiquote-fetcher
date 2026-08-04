# Maintainer: efebaykaraa <efebaykaraa@users.noreply.github.com>

pkgname=wikiquote-fetcher
pkgver=1.1.0
pkgrel=1
pkgdesc="Fetch quotes from Wikiquote, translate them, and manage reusable quote pools"
arch=('x86_64')
url="https://github.com/efebaykaraa/wikiquote-fetcher"
license=('GPL-3.0-or-later')
depends=('gcc-libs' 'glibc')
makedepends=('cargo')
_so_asset=libwikiquote_fetcher-x86_64-linux.so
source=(
  "$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz"
  "$_so_asset::$url/releases/download/v$pkgver/$_so_asset"
)
sha256sums=(
  '564d18df3480c08ef477a2b653952a9dcde58266e8f7ce6436326fbc1ebd0911'
  '5d5d43d9075801006cb99bd9eed4efcde64e87c736b4989c57e129c0ae824647'
)

_cargo_environment() {
  export CARGO_HOME="$srcdir/cargo-home"
  export CARGO_TARGET_DIR=target
  unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
  export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/gcc
  export CC=gcc
}

prepare() {
  cd "$pkgname-$pkgver"
  _cargo_environment
  cargo fetch --locked
}

build() {
  cd "$pkgname-$pkgver"
  _cargo_environment
  cargo build --frozen --release --lib

  local ring_out
  local ring_rlib
  ring_out=$(find target/release/build -path '*/ring-*/out' -type d -print -quit)
  ring_rlib=$(find target/release/deps -name 'libring-*.rlib' -print -quit)
  if [[ -n $ring_out && -n $ring_rlib && -f $ring_out/libring_core_0_17_14_.a ]]; then
    (cd "$ring_out" && ar x libring_core_0_17_14_.a)
    ar r "$ring_rlib" "$ring_out"/*.o
    rm -f "$ring_out"/*.o
  fi

  cargo build --frozen --release --bin wikiquote-fetcher
}

check() {
  cd "$pkgname-$pkgver"
  _cargo_environment
  cargo test --frozen
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 target/release/wikiquote-fetcher \
    "$pkgdir/usr/bin/wikiquote-fetcher"
  install -Dm755 "$srcdir/$_so_asset" \
    "$pkgdir/usr/lib/libwikiquote_fetcher.so.$pkgver"
  ln -s "libwikiquote_fetcher.so.$pkgver" \
    "$pkgdir/usr/lib/libwikiquote_fetcher.so"
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
