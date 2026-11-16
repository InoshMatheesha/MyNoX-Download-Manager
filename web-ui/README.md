# MyNoX website

The download page for MyNoX Download Manager: plain HTML, CSS and JavaScript, no build step.
The download buttons read the latest release from the GitHub API, so a new release needs no
change here.

## Preview

```
python -m http.server 8000 --directory web-ui
```

Then open http://127.0.0.1:8000.

## Deploy on Vercel

1. Import the GitHub repository in Vercel.
2. Set **Root Directory** to `web-ui`.
3. Framework preset: **Other**. No build command, no output directory.

`vercel.json` sets clean URLs and the caching and security headers.

## Pictures

`assets/` holds the logo and the screenshots (light and dark). Replace them with new ones from
`docs/images/` after a design change.
