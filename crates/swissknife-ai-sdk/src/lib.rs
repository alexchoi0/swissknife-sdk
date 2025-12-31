mod error;
mod registry;
mod tool;
mod types;

#[cfg(feature = "llm")]
pub mod llm;

#[cfg(feature = "mcp")]
pub mod mcp;

#[cfg(feature = "duckdb")]
pub mod memory;

pub use error::{Error, Result};
pub use registry::ToolRegistry;
pub use tool::{Tool, ToolBuilder};
pub use types::{
    OutputSchema, OutputType, ParameterSchema, ParameterType, ParameterVisibility,
    ToolSpec, ToolOutput, ToolTiming,
};

pub use tool::{
    get_array_param, get_bool_param, get_f64_param, get_i64_param, get_object_param,
    get_required_i64_param, get_required_string_param, get_string_param,
};

pub mod prelude {
    pub use crate::registry::ToolRegistry;
    pub use crate::tool::Tool;
    pub use crate::types::{
        OutputSchema, ParameterSchema, ToolSpec, ToolOutput,
    };
    pub use crate::{Error, Result};
}
