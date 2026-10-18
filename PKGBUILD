# Maintainer: Jakub Ziaja
pkgname=xsakura
pkgver=1.1.0
pkgrel=1
pkgdesc='Pixel-art cherry blossom tree that grows and sheds petals in your terminal'
arch=('any')
url='https://github.com/77nyan/xsakura'
license=('MIT')
depends=('python')
source=('xsakura'
        'xsakura.1'
        'xsakura.bash'
        '_xsakura'
        'xsakura.fish'
        'LICENSE'
        'README.md')
sha256sums=('5aca5ecac50caccf1917b5ce1e79c7276d9d0eb868fcc46cea095e2b2d306270'
            '0d66ff736eeb5da2c152ef211b643f83b718dbea6fa4c4d74ebf2daf910c9f72'
            '0a64243f24aad4ee6cfe9631cb1273cd11e4cd8914c7219ce9df4bec4cddce1e'
            'c762244edc602797f27fd8cd82056e1a900664c42b0bce1cafebf98b8a654887'
            '1ed5afee77b2196d73accf534e8bf48eb95e02a7e22eb59adf594d3b79a787fa'
            '2467a4fb768bd7cb99c1bd762898b068596075447ca5a97e0420dcc32bacff4b'
            '21d6d007a0848c8a646c8a351a963debbacf16f1cef46bb737bcd790913fb60d')

check() {
  cd "$srcdir"
  python ./xsakura --version
  python ./xsakura --print --seed 1 --width 60 --height 20 > /dev/null
}

package() {
  cd "$srcdir"
  install -Dm755 xsakura "$pkgdir/usr/bin/xsakura"
  install -Dm644 xsakura.1 "$pkgdir/usr/share/man/man1/xsakura.1"
  install -Dm644 xsakura.bash "$pkgdir/usr/share/bash-completion/completions/xsakura"
  install -Dm644 _xsakura "$pkgdir/usr/share/zsh/site-functions/_xsakura"
  install -Dm644 xsakura.fish "$pkgdir/usr/share/fish/vendor_completions.d/xsakura.fish"
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
