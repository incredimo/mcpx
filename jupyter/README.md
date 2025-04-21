# Jupyter MCP Server (Rust)

A lightweight Rust implementation of a Model Context Protocol (MCP) server for interacting with Jupyter notebooks directly - no Jupyter server required.

## Overview

This project implements an MCP (Model Context Protocol) server in Rust that works directly with local `.ipynb` files. It doesn't require a running Jupyter server or any complex dependencies - just provide a file path and it works.

## Features

- Create new Jupyter notebooks
- Add markdown cells to Jupyter notebooks
- Add code cells to Jupyter notebooks
- Add and execute code cells with output capture
- Read notebook contents

## Requirements

- Rust (cargo)
- Python (for code execution only)

## Usage

```bash
# Build and run the server
cargo run

# Set environment variables (optional for logging level)
export RUST_LOG=info
```

## How It Works

This server handles notebooks by:

1. Directly reading/writing `.ipynb` files as JSON
2. Manipulating the notebook structure in memory
3. For code execution, it spawns a Python process to run the code and captures the output
4. All results are saved back to the notebook file

## MCP Tools Available

1. `create_notebook`: Creates a new empty Jupyter notebook
2. `read_notebook_content`: Reads a notebook file and returns its contents
3. `add_markdown_cell`: Adds a markdown cell to a notebook
4. `add_code_cell`: Adds a code cell to a notebook (without execution)
5. `add_execute_code_cell`: Adds a code cell, executes it, and saves the output

## Example Usage

When using this MCP server, you'll provide the notebook path with each request:

```json
{
  "notebook_path": "path/to/your/notebook.ipynb",
  "cell_content": "# This is a markdown heading"
}
```

## Building

```bash
cargo build --release
```

## License

BSD 3-Clause License
