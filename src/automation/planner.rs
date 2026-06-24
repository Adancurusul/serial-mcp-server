use serde::Serialize;

use super::{
    validate_pack, AssemblyStep, AutomationError, AutomationResult, Encoding, ExpectOperation,
    MacroPack, MacroStep,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MacroTarget {
    Macro(String),
    Assembly(String),
}

impl MacroTarget {
    pub fn macro_named(name: impl Into<String>) -> Self {
        Self::Macro(name.into())
    }

    pub fn assembly_named(name: impl Into<String>) -> Self {
        Self::Assembly(name.into())
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Macro(_) => "macro",
            Self::Assembly(_) => "assembly",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Macro(name) | Self::Assembly(name) => name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MacroPlan {
    pub pack_name: String,
    pub target_kind: String,
    pub target_name: String,
    pub steps: Vec<PlanStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlanStep {
    Send {
        source_macro: String,
        data: String,
        encoding: Encoding,
    },
    Delay {
        source_macro: String,
        ms: u64,
    },
    Expect {
        source_macro: String,
        op: ExpectOperation,
        data: String,
        encoding: Encoding,
        timeout_ms: u64,
        idle_ms: u64,
        max_bytes: usize,
        trim: bool,
    },
}

pub fn plan_target(pack: &MacroPack, target: MacroTarget) -> AutomationResult<MacroPlan> {
    validate_pack(pack)?;
    let steps = match &target {
        MacroTarget::Macro(name) => plan_macro(pack, name)?,
        MacroTarget::Assembly(name) => plan_assembly(pack, name)?,
    };

    Ok(MacroPlan {
        pack_name: pack.name.clone(),
        target_kind: target.kind().to_string(),
        target_name: target.name().to_string(),
        steps,
    })
}

fn plan_macro(pack: &MacroPack, name: &str) -> AutomationResult<Vec<PlanStep>> {
    let macro_definition = pack
        .macros
        .iter()
        .find(|item| item.name == name)
        .ok_or_else(|| AutomationError::UnknownMacro(name.to_string()))?;
    Ok(expand_macro_steps(
        &macro_definition.name,
        &macro_definition.steps,
    ))
}

fn plan_assembly(pack: &MacroPack, name: &str) -> AutomationResult<Vec<PlanStep>> {
    let assembly = pack
        .assemblies
        .iter()
        .find(|item| item.name == name)
        .ok_or_else(|| AutomationError::UnknownAssembly(name.to_string()))?;
    let mut steps = Vec::new();
    for step in &assembly.steps {
        let AssemblyStep::Macro(call) = step;
        steps.extend(plan_macro(pack, &call.name)?);
    }
    Ok(steps)
}

fn expand_macro_steps(source_macro: &str, steps: &[MacroStep]) -> Vec<PlanStep> {
    steps
        .iter()
        .map(|step| match step {
            MacroStep::Send(send) => PlanStep::Send {
                source_macro: source_macro.to_string(),
                data: send.data.clone(),
                encoding: send.encoding,
            },
            MacroStep::Delay(delay) => PlanStep::Delay {
                source_macro: source_macro.to_string(),
                ms: delay.ms,
            },
            MacroStep::Expect(expect) => PlanStep::Expect {
                source_macro: source_macro.to_string(),
                op: expect.op,
                data: expect.data.clone(),
                encoding: expect.encoding,
                timeout_ms: expect.timeout_ms,
                idle_ms: expect.idle_ms,
                max_bytes: expect.max_bytes,
                trim: expect.trim,
            },
        })
        .collect()
}
