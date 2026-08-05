pub mod codebase;
pub mod memory;
pub mod file_ops;
pub mod shell_git;
pub mod refactor;
pub mod docs;
pub mod plugins;
pub mod system;
pub mod prompt_audit;

use serde_json::Value;
use tokio::runtime::Runtime;

pub fn list_all_tools() -> Vec<Value> {
    let mut tools = Vec::new();
    tools.extend(codebase::list_schemas());
    tools.extend(memory::list_schemas());
    tools.extend(file_ops::list_schemas());
    tools.extend(shell_git::list_schemas());
    tools.extend(refactor::list_schemas());
    tools.extend(docs::list_schemas());
    tools.extend(plugins::list_schemas());
    tools.extend(system::list_schemas());
    tools.extend(prompt_audit::list_schemas());
    tools
}

pub fn dispatch_tool_call(name: &str, params: &Value, rt: &Runtime) -> Option<String> {
    if let Some(res) = codebase::handle_call(name, params, rt) {
        return Some(res);
    }
    if let Some(res) = memory::handle_call(name, params, rt) {
        return Some(res);
    }
    if let Some(res) = file_ops::handle_call(name, params) {
        return Some(res);
    }
    if let Some(res) = shell_git::handle_call(name, params) {
        return Some(res);
    }
    if let Some(res) = refactor::handle_call(name, params, rt) {
        return Some(res);
    }
    if let Some(res) = docs::handle_call(name, params, rt) {
        return Some(res);
    }
    if let Some(res) = plugins::handle_call(name, params, rt) {
        return Some(res);
    }
    if let Some(res) = system::handle_call(name, params, rt) {
        return Some(res);
    }
    if let Some(res) = prompt_audit::handle_call(name, params, rt) {
        return Some(res);
    }
    None
}
