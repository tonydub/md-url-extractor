# md-url-extractor

A tool for extracting, cleaning, and exporting URLs from Markdown files.

## Features

- **Extract URLs**: Recursively scan directories for Markdown files and extract all URLs
- **URL Cleaning**:
  - Remove tracking parameters (utm_source, fbclid, gclid, etc.)
  - Normalize YouTube URLs to canonical format (`https://www.youtube.com/watch?v=VIDEO_ID`)
- **Filtering Options**:
  - Filter by domain (e.g., only github.com URLs)
  - Filter by protocol (http, https, ftp, etc.)
- **Multiple Output Formats**:
  - Plain text output to terminal (stdout)
  - Text file with one URL per line
  - CSV file with URLs, source files, and link text
  - HTML bookmarks file importable into browsers

## Installation

### From Source

```bash
git clone https://github.com/tonydub/md-url-extractor.git
cd md-url-extractor
cargo build --release
```

The compiled binary will be available at `target/release/md-url-extractor`.

### Using Cargo

```bash
cargo install md-url-extractor
```

## Usage

```
md-url-extractor [OPTIONS] <input_dir>
```

### Arguments

- `<input_dir>`: Directory containing Markdown files to scan

### Options

- `-f, --format <FORMAT>`: Output format [default: stdout] [possible values: stdout, text, csv, html]
- `-o, --output <FILE>`: Output file path (required for text, csv, html formats)
- `--domain <DOMAIN>`: Filter URLs by domain
- `--protocol <PROTOCOL>`: Filter URLs by protocol [default: http https] [possible values: http, https, ftp, file, mailto]
- `-h, --help`: Show help information
- `-V, --version`: Show version information

## Examples

### Basic Usage

Extract URLs from Markdown files and print to console:

```bash
md-url-extractor ~/Documents/notes/
```

### Filter URLs by Domain

Extract only GitHub links:

```bash
md-url-extractor ~/Documents/notes/ --domain github.com
```

### Export to CSV

Extract URLs and save to CSV with source information:

```bash
md-url-extractor ~/Documents/notes/ --format csv --output links.csv
```

### Export to HTML Bookmarks

Create a bookmarks file that can be imported into browsers:

```bash
md-url-extractor ~/Documents/notes/ --format html --output bookmarks.html
```

### Filter by Protocol

Extract only HTTPS links:

```bash
md-url-extractor ~/Documents/notes/ --protocol https
```

## URL Cleaning

### Tracking Parameter Removal

The following parameters are automatically removed from URLs:
- `utm_*` parameters (utm_source, utm_medium, utm_campaign, etc.)
- Facebook click identifier (`fbclid`)
- Google click identifier (`gclid`)
- Microsoft click identifier (`msclkid`)
- Generic tracking parameters (`ref`, `source`)

Example:
- From: `https://example.com/article?id=123&utm_source=twitter&utm_medium=social`
- To: `https://example.com/article?id=123`

### YouTube URL Normalization

All YouTube URL formats are converted to the standard format:

Examples:
- `https://youtu.be/dQw4w9WgXcQ` → `https://www.youtube.com/watch?v=dQw4w9WgXcQ`
- `https://youtube.com/embed/dQw4w9WgXcQ` → `https://www.youtube.com/watch?v=dQw4w9WgXcQ`
- `https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=120&feature=recommended` → `https://www.youtube.com/watch?v=dQw4w9WgXcQ`

## Architecture

This tool is built with a clean architecture approach:

- **Domain Layer**: Core business logic for URL processing and cleaning
- **Application Layer**: Orchestrates the extraction and processing workflow
- **Infrastructure Layer**: Handles CLI arguments, file I/O, and output formatting

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/your-feature-name`)
3. Commit your changes (`git commit -m 'Add some feature'`)
4. Push to the branch (`git push origin feature/your-feature-name`)
5. Open a Pull Request

### Future Improvements

- Support for additional URL cleaning rules
- Option to check if links are valid
- Metadata extraction from link destinations
- Support for additional output formats

## License

This project is licensed under the MIT License - see the LICENSE file for details.
