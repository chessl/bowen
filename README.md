# bowen

`bowen` is a fork of [Zola](https://github.com/getzola/zola), starting its own release line at
version 0.1.0.

It remains a fast static site generator in a single binary with everything built-in.

For inherited Zola behavior, see the [Zola Documentation](https://www.getzola.org/documentation/getting-started/overview/)
or the [docs/content](docs/content) folder in this repository.

This fork is derived from Zola 0.22.1 and adds Typst-backed math rendering support.
Project-specific behavior should be documented here as `bowen` diverges from upstream.

## Features

- [Single binary](https://www.getzola.org/documentation/getting-started/cli-usage/)
- [Syntax highlighting](https://www.getzola.org/documentation/content/syntax-highlighting/)
- Typst-backed math rendering
- [Sass compilation](https://www.getzola.org/documentation/content/sass/)
- Assets co-location
- [Multilingual site support](https://www.getzola.org/documentation/content/multilingual/) (Basic currently)
- [Image processing](https://www.getzola.org/documentation/content/image-processing/)
- [Themes](https://www.getzola.org/documentation/themes/overview/)
- [Shortcodes](https://www.getzola.org/documentation/content/shortcodes/)
- [Internal links](https://www.getzola.org/documentation/content/linking/)
- [External link checker](https://www.getzola.org/documentation/getting-started/cli-usage/#check)
- [Table of contents automatic generation](https://www.getzola.org/documentation/content/table-of-contents/)
- Automatic header anchors
- [Aliases](https://www.getzola.org/documentation/content/page/#front-matter)
- [Pagination](https://www.getzola.org/documentation/templates/pagination/)
- [Custom taxonomies](https://www.getzola.org/documentation/templates/taxonomies/)
- [Search with no servers or any third parties involved](https://www.getzola.org/documentation/content/search/)
- [Live reload](https://www.getzola.org/documentation/getting-started/cli-usage/#serve)
- Deploy on many platforms easily: [Netlify](https://www.getzola.org/documentation/deployment/netlify/), [Vercel](https://www.getzola.org/documentation/deployment/vercel/), [Cloudflare Pages](https://www.getzola.org/documentation/deployment/cloudflare-pages/), etc

## License

`bowen` contains code under multiple licenses.

Code derived from Zola is licensed under the EUPL-1.2. Code that existed in Zola prior to commit
3c9131db0d203640b6d5619ca1f75ce1e0d49d8f remains licensed under the MIT License, including in
later versions of this project.

Additional `bowen` modifications are licensed under the EUPL-1.2 unless noted otherwise.

See LICENSE and LICENSE-MIT for details.
