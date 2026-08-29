# Maintainer: Aki <contact@omarchy.org>
pkgname=boombox-tui
pkgver=2.4.0
pkgrel=1
pkgdesc="A retro cyberpunk cassette boombox music player and radio explorer for local Hi-Res audio, YouTube streams, and global radio stations."
arch=('any')
url="https://github.com/dannie203/tui-radio"
license=('GPL-3.0-or-later')
depends=('nodejs>=20' 'mpv' 'yt-dlp' 'ffmpeg')
optdepends=(
  'libnotify: Desktop notifications when tracks change or recordings finish'
  'python-gobject: Enhanced MPRIS2 and system tray integration'
)
provides=('boombox' 'boombox-tui' 'hiphop-radio')
conflicts=('hiphop-radio-git' 'tui-radio-git')
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

package() {
  cd "$srcdir/tui-radio-$pkgver" 2>/dev/null || cd "$srcdir/$pkgname-$pkgver" 2>/dev/null || cd "$srcdir"

  install -d "$pkgdir/usr/lib/$pkgname"
  cp -r * "$pkgdir/usr/lib/$pkgname/"

  # Install binary wrappers
  install -d "$pkgdir/usr/bin"
  ln -s "/usr/lib/$pkgname/bin/index.js" "$pkgdir/usr/bin/boombox"
  ln -s "/usr/lib/$pkgname/bin/index.js" "$pkgdir/usr/bin/boombox-tui"
  ln -s "/usr/lib/$pkgname/bin/toggle.js" "$pkgdir/usr/bin/boombox-toggle"
  chmod +x "$pkgdir/usr/lib/$pkgname/bin/index.js"
  chmod +x "$pkgdir/usr/lib/$pkgname/bin/toggle.js"

  # Install desktop entry and icons
  install -Dm644 "assets/boombox.desktop" "$pkgdir/usr/share/applications/boombox.desktop"
  install -Dm644 "assets/icons/hicolor/scalable/apps/boombox.svg" "$pkgdir/usr/share/icons/hicolor/scalable/apps/boombox.svg"
  for size in 16 32 48 64 128 256 512; do
    if [ -f "assets/icons/hicolor/${size}x${size}/apps/boombox.png" ]; then
      install -Dm644 "assets/icons/hicolor/${size}x${size}/apps/boombox.png" "$pkgdir/usr/share/icons/hicolor/${size}x${size}/apps/boombox.png"
    fi
  done
}
