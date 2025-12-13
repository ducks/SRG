# SRG - Static Resume Generator

Build HTML and PDF resumes from JOBL files.

## Installation

```bash
cargo install --path .
```

## Usage

```bash
srg --input resume.jobl --out dist --template minimal
```

### Options

- `-i, --input <FILE>` - Input JOBL file (required)
- `-o, --out <DIR>` - Output directory (default: dist)
- `-t, --template <NAME>` - Template name (default: minimal)

### Examples

Build from a JOBL file:

```bash
srg --input resume.jobl
```

Specify custom output directory:

```bash
srg --input resume.jobl --out public
```

## Output

SRG generates:
- `index.html` - Styled HTML resume
- `resume.pdf` - PDF version (placeholder for now)

## Templates

### minimal

Clean, professional single-page layout with:
- Contact information header
- Skills organized by category
- Work experience with highlights
- Projects with links
- Education with details

CSS is optimized for both screen and print.

## Requirements

Requires a valid JOBL file. See the [JOBL
repository](https://github.com/ducks/jobl) for format specification.

## License

MIT
