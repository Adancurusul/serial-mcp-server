//! Application-layer serial automation.

mod dsl;
mod error;
mod executor;
mod planner;
mod registry;
mod transport;
mod validation;

pub use dsl::{
    AssemblyDefinition, AssemblyStep, DataRecord, DelayStep, Encoding, ExpectOperation, ExpectStep,
    MacroDefinition, MacroPack, MacroStep, SendStep,
};
pub use error::{AutomationError, AutomationResult};
pub use executor::{MacroExecutor, RunReport, StepReport};
pub use planner::{plan_target, MacroPlan, MacroTarget, PlanStep};
pub use registry::{LoadedMacroPack, MacroList, MacroLoadRecord, MacroRegistry};
pub use transport::{MacroTransport, SerialMacroTransport, SimulatedMacroTransport};
pub use validation::{validate_pack, MacroInventory};

impl From<AutomationError> for crate::error::SerialError {
    fn from(error: AutomationError) -> Self {
        crate::error::SerialError::InvalidConfig(error.to_string())
    }
}
