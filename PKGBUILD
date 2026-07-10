pkgname=wikiquote-fetcher
pkgver=0.1.1
pkgrel=1
pkgdesc="Fetches quotes from Wikiquote for desktop quote overlays"
arch=('x86_64')
license=('GPL-3.0-or-later')
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
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
