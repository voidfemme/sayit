pkgname=sayit
pkgver=0.1.0
pkgrel=1 
pkgdesc="A text-to-speech CLI utility"
arch=('x86_64')
url="https://github.com/voidfemme/sayit"
license=('MIT')
depends=('openssl' 'alsa-lib')
makedepends=('cargo' 'git')
source=("$pkgname-$pkgver.tar.gz::https://github.com/voidfemme/sayit/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$srcdir/$pkgname-$pkgver"
  cargo build --release
}

package() {
  cd "$srcdir/$pkgname-$pkgver"
  install -Dm755 "target/release/sayit" "$pkgdir/usr/bin/sayit"
  install -Dm644 "README.md" "$pdgdir/usr/share/doc/$pkgname/README.md"
}
