<img style="display: block; margin: auto;" src="src/banner/banner.png"/>

A powerful command-line tool for analyzing CSS/SCSS files and finding unused classes in your projects. Clean up your codebase by identifying dangling CSS classes that are safe to remove!
You may consider using the GUI version of this application, [TagFinder](https://github.com/renseck/TagFinder).

## Features

- **Unused CSS Detection**: Find CSS classes defined in stylesheets but not used anywhere in your project
- **Word Search**: Search for specific words/terms and see if they appear only in CSS files
- **Detailed Reports**: Get comprehensive breakdowns by file or detailed analysis
- **Beautiful Output**: Clean, colorized terminal output with progress indicators
- **Fast Scanning**: Efficiently processes large codebases using Rust's performance

## 🚀 Installation

### From Source
```bash
git clone https://github.com/your-username/tag-finder-cli.git
cd tag-finder-cli
cargo build --release
```

The executable will be available at `target/release/tag-finder` (or `tag-finder.exe` on Windows).

## Usage

### Find Unused CSS Classes

Analyze your project to find CSS classes that are defined but never used:

```bash
# Analyze current directory
tag-finder unused-classes

# Analyze specific directory
tag-finder unused-classes --directory ./src
# Or from the tag-finder dirctory
cargo run -q unused-classes -d /path/to/project 

# Get detailed breakdown by file
tag-finder unused-classes --by-file

# Get full detailed report
tag-finder unused-classes --detailed
```

### Find Specific Words

Search for words that appear only in CSS/SCSS files (useful for finding CSS-specific code):

```bash
# Find a specific CSS class
tag-finder find-word --word "navbar-brand"

# Search in specific directory
tag-finder find-word --word "button-primary" --directory ./assets

# Show all matches (not just CSS-only)
tag-finder find-word --word "container" --all
```

## Command Reference

### `unused-classes`

Find CSS classes that are defined but not used in your project.

**Options:**
- `-d, --directory <PATH>` - Directory to analyze (default: current directory)
- `-b, --by-file` - Show detailed breakdown by file
- `--detailed` - Show full detailed report with all unused classes

**Examples:**
```bash
tag-finder unused-classes --directory ./src --by-file
tag-finder unused-classes --detailed
```

### `find-word`

Search for a specific word and determine if it appears only in CSS files.

**Options:**
- `-w, --word <WORD>` - The word to search for (exact match)
- `-d, --directory <PATH>` - Directory to search in (default: current directory)  
- `-a, --all` - Show all matches, not just CSS-only ones

**Examples:**
```bash
tag-finder find-word --word "hero-section" --directory ./styles
tag-finder find-word --word "utility-class" --all
```

## How It Works

The tool follows a systematic workflow to analyze your codebase:

### Component-Level Workflow

1. **File Discovery**
   - [`FileWalker`](src/file_walker.rs) recursively scans the specified directory
   - Identifies CSS/SCSS files using extension filtering
   - Builds a comprehensive file list for analysis

2. **CSS Class Extraction**
   - [`CssParser`](src/css_parser.rs) processes each CSS/SCSS file
   - Uses regex patterns to identify class definitions (`.class-name`)
   - [`TextProcessor`](src/text_processor.rs) handles pattern matching and text analysis
   - Deduplicates classes to avoid counting the same class multiple times per file

3. **Usage Analysis**
   - For each discovered CSS class, [`FileScanner`](src/scanner.rs) searches the entire project
   - Performs exact word matching to find class usage in HTML, JavaScript, templates, etc.
   - Determines if classes appear only in CSS files (indicating potential unused code)

4. **Report Generation**
   - [`UnusedDetector`](src/unused_detector.rs) compiles analysis results
   - Categorizes classes as used vs. unused
   - Provides multiple report formats (summary, by-file, detailed)

5. **Output Formatting**
   - [`utils.rs`](src/utils.rs) handles banner display and formatting
   - [`ProgressReporter`](src/progress_reporter.rs) shows real-time progress
   - Clean, colorized terminal output with emojis and formatting

### Supported File Types

- **CSS Analysis**: `.css`, `.scss`
- **Usage Detection**: All text-based files in your project
- **Smart Filtering**: Ignores binary files and respects common ignore patterns

## Example Output

### Unused Classes Summary
```
📋 UNUSED CSS CLASSES REPORT
==================================================
Total classes analyzed: 156
Unused classes: 23
Used classes: 133
Unused percentage: 14.7%

🗑️  UNUSED CLASSES (first 10):
  .old-button in styles/components.scss (line 45)
  .legacy-header in styles/layout.scss (line 12)
  .unused-utility in styles/utilities.scss (line 78)
  ... and 13 more

Use --detailed for full list or --by-file for file breakdown
```

### Word Search Results
```
Search results for word: 'navbar-toggle'
==================================================
Found in CSS/SCSS files:
  ✓ src/styles/components.scss

🎯 SUCCESS: 'navbar-toggle' appears ONLY in CSS/SCSS files!
This code might be extraneous and safe to remove.
```

## Configuration

The tool works out of the box with sensible defaults, but you can customize behavior:

- **Directory Scope**: Use `--directory` to limit analysis to specific folders
- **Report Verbosity**: Choose between summary, by-file, or detailed reports
- **Search Scope**: Use `--all` flag to see matches in all file types

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Issues & Support

If you encounter any issues or have questions:

1. Check the [Issues](https://github.com/renseck/tag-finder-cli/issues) page
2. Create a new issue with:
   - Your operating system
   - Command you ran
   - Expected vs actual behavior
   - Sample files (if applicable)

## Why Use Tag Finder CLI?

- **Clean Codebase**: Remove dead CSS code and reduce bundle sizes
- **Performance**: Fast Rust-based analysis even for large projects  
- **Accuracy**: Precise word matching prevents false positives
- **Insights**: Detailed reports help understand your CSS usage patterns
- **Developer Friendly**: Simple CLI interface with helpful output

---

Made with ❤️ in Rust. Happy cleaning!