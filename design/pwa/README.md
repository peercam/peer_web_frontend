# PWA Asset Sources

SVG sources for the PWA icon set, splash screens, and manifest screenshots.

These are **design-time only** — they are not served at runtime. The runtime PNGs live in `public/img/pwa/`.

## Regenerating PNGs

From the repo root (macOS — uses built-in `qlmanage` + `sips`):

```sh
gen() {
  local src="$1"; local out="$2"; local w="$3"; local h="$4"
  local maxdim=$(( w > h ? w : h ))
  qlmanage -t -s "$maxdim" -o /tmp "$src" >/dev/null 2>&1
  sips -z "$h" "$w" "/tmp/$(basename "$src").png" --out "$out" >/dev/null
  rm -f "/tmp/$(basename "$src").png"
}

cd design/pwa
gen _src-icon.svg               ../../public/img/pwa/icon-192.png                  192 192
gen _src-icon.svg               ../../public/img/pwa/icon-512.png                  512 512
gen _src-icon-maskable.svg      ../../public/img/pwa/icon-192-maskable.png         192 192
gen _src-icon-maskable.svg      ../../public/img/pwa/icon-512-maskable.png         512 512
gen _src-icon.svg               ../../public/img/pwa/apple-touch-icon-180.png      180 180
gen _src-screenshot-desktop.svg ../../public/img/pwa/screenshot-dashboard-desktop.png 1920 1080
gen _src-screenshot-mobile.svg  ../../public/img/pwa/screenshot-dashboard-mobile.png   390  844

cd splash
gen _src-2048-2732.svg ../../../public/img/pwa/splash/apple-splash-2048-2732.png 2048 2732
gen _src-1290-2796.svg ../../../public/img/pwa/splash/apple-splash-1290-2796.png 1290 2796
gen _src-1170-2532.svg ../../../public/img/pwa/splash/apple-splash-1170-2532.png 1170 2532
```

The monochrome icon (`icon-monochrome.svg`) ships as SVG — no conversion needed.

Validate maskable variants at [maskable.app](https://maskable.app) before committing. The logo must stay within the inner 80% safe zone so platform-specific masking (circles, squircles, rounded rects) never crops it.
