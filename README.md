# templatex

A simple yet powerful template manager for Tectonic LaTeX projects.

`templatex` streamlines the creation of new Tectonic projects by scaffolding them from predefined templates. It features an interactive terminal UI for template selection and uses the Tera templating engine for dynamic content generation.

## Features

-   **Interactive Template Selection**: A clean TUI for picking templates if multiple are available.
-   **Dynamic Templating**: Uses the [Tera](https://keats.github.io/tera/) engine to substitute variables in your template files.
-   **Flexible Configuration**: Configure template sources globally via TOML files or environment variables.
-   **Automatic `Tectonic.toml` Generation**: Creates a basic `Tectonic.toml` for your new project.
-   **Customizable Template Sources**: Keep your templates organized in one or more directories.

## Installation

Install `templatex` using Cargo:

```sh
cargo install templatex
```

## Usage

To create a new project, run:

```sh
templatex <project_name> [OPTIONS]
```

This will launch an interactive prompt to select a template and fill in its variables. The new project will be created in a directory named `<project_name>`.

### Command-Line Arguments & Options

-   `<NAME>`: The name of the new project directory to be created. (Required)

-   `-t, --template-dir <TEMPLATE_DIR>`:
    Specify a single directory to search for templates, ignoring configured sources.

-   `-o, --out-dir <OUT_DIR>`:
    Set the output directory for the new project. Defaults to `<NAME>`.

-   `-s, --silent`:
    Suppress all logging output.

-   `-v, --verbose`:
    Enable verbose (DEBUG level) logging.

-   `--very-verbose`:
    Enable maximum verbosity (TRACE level) logging.

## Configuration

`templatex` can be configured to look for templates in one or more directories.

### Config File

You can create a configuration file to define your template source directories. The location is platform-specific:

-   Linux: `~/.config/templatex/config/`
-   macOS: `~/Library/Application Support/com.jayanaxhf.templatex/config/`
-   Windows: `C:\Users\<user>\AppData\Roaming\jayanaxhf\templatex\config\`

Create a TOML file in this directory (e.g., `settings.toml`) with the following format:

```toml
# ~/.config/templatex/config/settings.toml

# A list of directories where your templates are stored.
source_dirs = [
    "/path/to/my/templates",
    "~/Documents/latex-templates",
]
```

### Environment Variables

Configuration can also be managed via environment variables.

-   `TEMPLATEX_CONFIG`: Override the default configuration directory path.
-   `TEMPLATEX_SOURCE_DIRS__0`: Set the first source directory. Use `__1`, `__2`, etc., for additional directories.

Example:

```sh
export TEMPLATEX_SOURCE_DIRS__0="/path/to/my/templates"
export TEMPLATEX_SOURCE_DIRS__1="~/other-templates"
templatex my-new-project
```

## Creating Templates

A template is simply a directory containing the files for your Tectonic project.

### Directory Structure

```
my-templates/
└── basic-article/
    ├── templatex.toml  (optional)
    └── src/
        ├── main.tex
        └── preamble.tex
```

### `templatex.toml` (Optional)

You can add a `templatex.toml` file to the root of your template directory to provide a custom name and description for the TUI picker.

```toml
# my-templates/basic-article/templatex.toml
name = "Basic Article"
description = "A simple template for a standard article."
```

If this file is not present, the directory name (`basic-article`) will be used as the template's name.

### Template Variables

Files inside the `src/` directory are processed by the Tera templating engine. You can use `{{ variable_name }}` syntax to define placeholders that `templatex` will prompt you to fill in.

Example `main.tex`:

```latex
% src/main.tex
\documentclass{article}

\input{preamble.tex}

\title{ {{ title }} }
\author{ {{ author }} }
\date{ {{ date }} }

\begin{document}

\maketitle

\section{Introduction}
Hello, world!

\end{document}
```

When you use this template, `templatex` will ask you for values for `title`, `author`, and `date`.

## Logging

Log files are stored in a platform-specific data directory:

-   Linux: `~/.local/share/templatex/`
-   macOS: `~/Library/Application Support/com.jayanaxhf.templatex/`
-   Windows: `C:\Users\<user>\AppData\Local\jayanaxhf\templatex\data\`

The log level can be controlled with the `TEMPLATEX_LOG_LEVEL` environment variable (e.g., `export TEMPLATEX_LOG_LEVEL=debug`).
