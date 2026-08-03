# Maintainer: efebaykaraa <efebaykaraa@users.noreply.github.com>

pkgname=wikiquote-fetcher
pkgver=1.0.2
pkgrel=1
pkgdesc="Fetch quotes from Wikiquote, translate them, and manage reusable quote pools"
arch=('x86_64')
url="https://github.com/efebaykaraa/wikiquote-fetcher"
license=('GPL-3.0-or-later')
depends=('gcc-libs' 'glibc')
makedepends=('cargo')
_commit=b0b23b16199d2097b1893a5506e4a27e12debd04
source=("$pkgname-$pkgver.tar.gz::$url/archive/$_commit.tar.gz")
sha256sums=('97666f15b1d176852cd860356be15a9ac4768b45e6689c7ad8e8dfc2f98603a8')

_cargo_environment() {
  export CARGO_HOME="$srcdir/cargo-home"
  export CARGO_TARGET_DIR=target
  unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
  export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/usr/bin/gcc
  export CC=gcc
}

prepare() {
  cd "$pkgname-$_commit"
  _cargo_environment
  cargo fetch --locked
}

build() {
  cd "$pkgname-$_commit"
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
  cd "$pkgname-$_commit"
  _cargo_environment
  cargo test --frozen
}

package() {
  cd "$pkgname-$_commit"
  install -Dm755 target/release/wikiquote-fetcher \
    "$pkgdir/usr/bin/wikiquote-fetcher"
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
