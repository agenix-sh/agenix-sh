# AGEniX Website

Documentation website for the AGEniX project, built with [Docusaurus](https://docusaurus.io/).

## Development

```bash
cd website
npm install
npm start
```

This starts a local development server at `http://localhost:3000`.

## Build

```bash
npm run build
```

Generates static content into the `build` directory.

## Deployment

The website is configured to deploy to GitHub Pages. To deploy:

1. Enable GitHub Pages in repository settings
2. Set source to "GitHub Actions"
3. Push changes to main branch - the deploy workflow will run automatically

## Configuration

- `docusaurus.config.ts` - Main configuration
- `sidebars.ts` - Documentation sidebar structure
- `../docs/` - Documentation content (parent directory)
- `src/pages/` - Custom pages (landing page, etc.)

## Customization

- Logo: `static/img/logo.svg`
- Favicon: `static/img/favicon.ico`
- CSS: `src/css/custom.css`
- Homepage: `src/pages/index.tsx`
- Features: `src/components/HomepageFeatures/index.tsx`
