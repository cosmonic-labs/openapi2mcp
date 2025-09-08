use crate::mcp_server::MCPServer;

pub fn generate_typescript_code<F>(_mcp_server: &MCPServer, _file_code: F)
where
    F: Fn(FileCode),
{
}

pub struct FileCode {
    pub path: String,
    pub code: String,
}
