pkgname=wikiquote-fetcher
pkgver=0.1.0
pkgrel=1
pkgdesc="Wikiquote fetcher for Marxist Quote"
arch=('x86_64')
license=('unknown')
depends=('engyls')
makedepends=('cargo')
source=()
sha256sums=()

build() {
  cd "$startdir"
  cargo build --release --locked
}

package() {
  cd "$startdir"
  install -Dm755 target/release/wikiquote-fetcher "$pkgdir/usr/bin/wikiquote-fetcher"
}
