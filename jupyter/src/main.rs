use rmcp::{ServerHandler, ServiceExt, model::ServerInfo, schemars, tool};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{stdin, stdout};
use log::{info, warn};
use std::fs;
use std::path::Path;
use std::process::Command;

// Request structure for notebook path and cell content
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NotebookRequest {
    #[schemars(description = "Absolute or relative path to the .ipynb file, including filename")]
    pub notebook_path: String,
    #[schemars(description = "Content to be added to the cell, can be markdown or code")]
    pub cell_content: String,
}

// Request structure for notebook path only
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NotebookPathRequest {
    #[schemars(description = "Absolute or relative path to the .ipynb file, including filename")]
    pub notebook_path: String,
}

// Simplified notebook structures
#[derive(Debug, Serialize, Deserialize)]
struct Notebook {
    cells: Vec<Cell>,
    metadata: Value,
    nbformat: i32,
    nbformat_minor: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Cell {
    cell_type: String,
    metadata: Value,
    source: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    execution_count: Option<i32>,
    #[serde(default)]
    outputs: Vec<Value>,
}

// Main JupyterTools implementation
#[derive(Debug, Clone)]
pub struct JupyterTools {}

impl JupyterTools {
    pub fn new() -> Self {
        JupyterTools {}
    }

    fn read_notebook(&self, path: &str) -> Option<Notebook> {
        match fs::read_to_string(path) {
            Ok(notebook_content) => {
                match serde_json::from_str(&notebook_content) {
                    Ok(notebook) => Some(notebook),
                    Err(e) => {
                        warn!("Failed to parse notebook {}: {}", path, e);
                        None
                    }
                }
            },
            Err(e) => {
                warn!("Failed to read notebook {}: {}", path, e);
                None
            }
        }
    }

    fn write_notebook(&self, path: &str, notebook: &Notebook) -> bool {
        match serde_json::to_string_pretty(notebook) {
            Ok(notebook_content) => {
                match fs::write(path, notebook_content) {
                    Ok(_) => true,
                    Err(e) => {
                        warn!("Failed to write notebook {}: {}", path, e);
                        false
                    }
                }
            },
            Err(e) => {
                warn!("Failed to serialize notebook {}: {}", path, e);
                false
            }
        }
    }

    fn split_into_lines(content: &str) -> Vec<String> {
        content.lines().map(|line| format!("{}\n", line)).collect()
    }

    fn create_markdown_cell(&self, content: &str) -> Cell {
        Cell {
            cell_type: "markdown".to_string(),
            metadata: json!({}),
            source: Self::split_into_lines(content),
            execution_count: None,
            outputs: vec![],
        }
    }

    fn create_code_cell(&self, content: &str) -> Cell {
        Cell {
            cell_type: "code".to_string(),
            metadata: json!({}),
            source: Self::split_into_lines(content),
            execution_count: None,
            outputs: vec![],
        }
    }

    fn execute_code(&self, code: &str) -> Vec<String> {
        // Simple Python executor - just runs the code and captures output
        let output = match Command::new("python")
            .arg("-c")
            .arg(code)
            .output() {
                Ok(output) => output,
                Err(e) => {
                    warn!("Failed to execute Python code: {}", e);
                    return vec![format!("Error executing code: {}", e)];
                }
            };
        
        let mut results = Vec::new();
        
        if !output.stdout.is_empty() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                results.push(stdout);
            }
        }
        
        if !output.stderr.is_empty() {
            if let Ok(stderr) = String::from_utf8(output.stderr) {
                if !stderr.is_empty() {
                    results.push(format!("Error: {}", stderr));
                }
            }
        }
        
        if results.is_empty() {
            results.push("No output".to_string());
        }
        
        results
    }

    // Check if notebook exists and create empty one if not
    fn ensure_notebook_exists(&self, path: &str) -> bool {
        if !Path::new(path).exists() {
            info!("Notebook does not exist, creating a new one at {}", path);
            
            // Create empty notebook
            let empty_notebook = Notebook {
                cells: Vec::new(),
                metadata: json!({
                    "kernelspec": {
                        "display_name": "Python 3",
                        "language": "python",
                        "name": "python3"
                    },
                    "language_info": {
                        "codemirror_mode": {
                            "name": "ipython",
                            "version": 3
                        },
                        "file_extension": ".py",
                        "mimetype": "text/x-python",
                        "name": "python",
                        "nbconvert_exporter": "python",
                        "pygments_lexer": "ipython3",
                        "version": "3.8.0"
                    }
                }),
                nbformat: 4,
                nbformat_minor: 5,
            };
            
            // Create parent directories if they don't exist
            if let Some(parent) = Path::new(path).parent() {
                if !parent.exists() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        warn!("Failed to create directory {}: {}", parent.display(), e);
                        return false;
                    }
                }
            }
            
            // Write the empty notebook
            return self.write_notebook(path, &empty_notebook);
        }
        
        true
    }
}

// Define tool implementations
#[tool(tool_box)]
impl JupyterTools {
    #[tool(description = "Append a new markdown cell to an existing Jupyter notebook file or create a new notebook if it doesn't exist")]
    async fn add_markdown_cell(&self, #[tool(aggr)] request: NotebookRequest) -> String {
        info!("Adding markdown cell to {} with content length: {}", request.notebook_path, request.cell_content.len());
        
        // Ensure notebook exists
        if !self.ensure_notebook_exists(&request.notebook_path) {
            return format!("Failed to create or access notebook: {}", request.notebook_path);
        }
        
        // Read the notebook
        let mut notebook = match self.read_notebook(&request.notebook_path) {
            Some(nb) => nb,
            None => return format!("Failed to read notebook: {}", request.notebook_path),
        };
        
        // Create and add a markdown cell
        let cell = self.create_markdown_cell(&request.cell_content);
        notebook.cells.push(cell);
        
        // Write the updated notebook
        if !self.write_notebook(&request.notebook_path, &notebook) {
            return format!("Failed to write notebook: {}", request.notebook_path);
        }
        
        format!("Successfully added markdown cell to {}", request.notebook_path)
    }

    #[tool(description = "Append a new code cell to an existing Jupyter notebook file or create a new notebook if it doesn't exist")]
    async fn add_code_cell(&self, #[tool(aggr)] request: NotebookRequest) -> String {
        info!("Adding code cell to {} with content length: {}", request.notebook_path, request.cell_content.len());
        
        // Ensure notebook exists
        if !self.ensure_notebook_exists(&request.notebook_path) {
            return format!("Failed to create or access notebook: {}", request.notebook_path);
        }
        
        // Read the notebook
        let mut notebook = match self.read_notebook(&request.notebook_path) {
            Some(nb) => nb,
            None => return format!("Failed to read notebook: {}", request.notebook_path),
        };
        
        // Create and add a code cell
        let cell = self.create_code_cell(&request.cell_content);
        notebook.cells.push(cell);
        
        // Write the updated notebook
        if !self.write_notebook(&request.notebook_path, &notebook) {
            return format!("Failed to write notebook: {}", request.notebook_path);
        }
        
        format!("Successfully added code cell to {}", request.notebook_path)
    }

    #[tool(description = "Append a new code cell to a Jupyter notebook, execute the code, and save the outputs to the notebook")]
    async fn add_execute_code_cell(&self, #[tool(aggr)] request: NotebookRequest) -> String {
        info!("Adding and executing code cell in {} with content length: {}", 
              request.notebook_path, request.cell_content.len());
        
        // Ensure notebook exists
        if !self.ensure_notebook_exists(&request.notebook_path) {
            return format!("Failed to create or access notebook: {}", request.notebook_path);
        }
        
        // Read the notebook
        let mut notebook = match self.read_notebook(&request.notebook_path) {
            Some(nb) => nb,
            None => return format!("Failed to read notebook: {}", request.notebook_path),
        };
        
        // Execute the code
        let outputs = self.execute_code(&request.cell_content);
        
        // Create a new cell with outputs
        let mut cell = self.create_code_cell(&request.cell_content);
        cell.execution_count = Some(1); // Simple counter
        
        // Add formatted outputs to the cell
        for output_text in &outputs {
            let output_value = json!({
                "output_type": "stream",
                "name": "stdout",
                "text": output_text
            });
            cell.outputs.push(output_value);
        }
        
        // Add the cell to the notebook
        notebook.cells.push(cell);
        
        // Write the updated notebook
        if !self.write_notebook(&request.notebook_path, &notebook) {
            return format!("Failed to write notebook: {}", request.notebook_path);
        }
        
        // Return outputs as a joined string
        if outputs.is_empty() {
            "Code executed successfully, but produced no output".to_string()
        } else {
            // Serialize the outputs to JSON for proper formatting
            match serde_json::to_string_pretty(&outputs) {
                Ok(json_outputs) => format!("Code executed successfully. Output:\n{}", json_outputs),
                Err(_) => format!("Code executed successfully. Output:\n{}", outputs.join("\n\n"))
            }
        }
    }

    #[tool(description = "Read the contents of an existing Jupyter notebook file and return it as a JSON-formatted string")]
    async fn read_notebook_content(&self, #[tool(aggr)] request: NotebookPathRequest) -> String {
        info!("Reading notebook: {}", request.notebook_path);
        
        if !Path::new(&request.notebook_path).exists() {
            return format!("Error: Notebook not found at path: {}", request.notebook_path);
        }
        
        match fs::read_to_string(&request.notebook_path) {
            Ok(content) => format!("Successfully read notebook. Content:\n{}", content),
            Err(e) => format!("Error reading notebook: {}", e),
        }
    }

    #[tool(description = "Create a new empty Jupyter notebook file at the specified path")]
    async fn create_notebook(&self, #[tool(aggr)] request: NotebookPathRequest) -> String {
        info!("Creating new notebook: {}", request.notebook_path);
        
        // Check if notebook already exists
        if Path::new(&request.notebook_path).exists() {
            return format!("Error: Notebook already exists at path: {}", request.notebook_path);
        }
        
        // Create an empty notebook
        if self.ensure_notebook_exists(&request.notebook_path) {
            format!("Successfully created new notebook at path: {}", request.notebook_path)
        } else {
            format!("Error: Failed to create notebook at path: {}", request.notebook_path)
        }
    }
}

// Register tools with RMCP
#[tool(tool_box)]
impl ServerHandler for JupyterTools {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A Jupyter notebook MCP server that works directly with .ipynb files".into()),
            ..Default::default()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();
    
    info!("Starting Jupyter MCP Server (File Mode)");
    
    // Create our tools
    let tools = JupyterTools::new();
    
    // Set up the transport for stdio communication
    let transport = (stdin(), stdout());
    
    // Start the server
    info!("Starting MCP server");
    let server = tools.serve(transport).await?;
    
    // Wait for server completion
    info!("Server initialized, waiting for shutdown");
    let reason = server.waiting().await?;
    info!("Server shutdown: {:?}", reason);
    
    Ok(())
}
