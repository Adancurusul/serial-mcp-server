use std::collections::HashSet;

use serde::Serialize;

use super::{AssemblyStep, AutomationError, AutomationResult, MacroPack, MacroStep, SendStep};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MacroInventory {
    pub schema_version: String,
    pub name: String,
    pub macros: Vec<String>,
    pub assemblies: Vec<String>,
}

pub fn validate_pack(pack: &MacroPack) -> AutomationResult<MacroInventory> {
    require_non_empty("name", &pack.name)?;
    if pack.schema_version != "0.3" {
        return Err(AutomationError::validation(
            "schema_version",
            "expected schema version 0.3",
        ));
    }
    if pack.macros.is_empty() {
        return Err(AutomationError::validation(
            "macros",
            "at least one macro is required",
        ));
    }

    let mut macro_names = HashSet::new();
    for (macro_index, macro_definition) in pack.macros.iter().enumerate() {
        let field = format!("macros[{macro_index}].name");
        require_non_empty(&field, &macro_definition.name)?;
        if !macro_names.insert(macro_definition.name.as_str()) {
            return Err(AutomationError::validation(
                field,
                format!("duplicate macro name {}", macro_definition.name),
            ));
        }
        if macro_definition.steps.is_empty() {
            return Err(AutomationError::validation(
                format!("macros[{macro_index}].steps"),
                "macro steps must not be empty",
            ));
        }
        validate_macro_steps(macro_index, &macro_definition.steps)?;
    }

    let mut assembly_names = HashSet::new();
    for (assembly_index, assembly) in pack.assemblies.iter().enumerate() {
        let field = format!("assemblies[{assembly_index}].name");
        require_non_empty(&field, &assembly.name)?;
        if macro_names.contains(assembly.name.as_str()) {
            return Err(AutomationError::validation(
                field,
                format!("assembly name {} conflicts with a macro", assembly.name),
            ));
        }
        if !assembly_names.insert(assembly.name.as_str()) {
            return Err(AutomationError::validation(
                field,
                format!("duplicate assembly name {}", assembly.name),
            ));
        }
        if assembly.steps.is_empty() {
            return Err(AutomationError::validation(
                format!("assemblies[{assembly_index}].steps"),
                "assembly steps must not be empty",
            ));
        }
        validate_assembly_steps(assembly_index, &assembly.steps, &macro_names)?;
    }

    Ok(MacroInventory {
        schema_version: pack.schema_version.clone(),
        name: pack.name.clone(),
        macros: pack.macros.iter().map(|item| item.name.clone()).collect(),
        assemblies: pack
            .assemblies
            .iter()
            .map(|item| item.name.clone())
            .collect(),
    })
}

fn validate_macro_steps(macro_index: usize, steps: &[MacroStep]) -> AutomationResult<()> {
    for (step_index, step) in steps.iter().enumerate() {
        match step {
            MacroStep::Send(step) => validate_send_step(macro_index, step_index, step)?,
            MacroStep::Delay(step) => {
                if step.ms == 0 {
                    return Err(AutomationError::validation(
                        format!("macros[{macro_index}].steps[{step_index}].ms"),
                        "delay must be greater than zero",
                    ));
                }
            }
            MacroStep::Expect(step) => {
                let field = format!("macros[{macro_index}].steps[{step_index}].data");
                step.encoding.decode(&step.data, &field)?;
                if step.timeout_ms == 0 {
                    return Err(AutomationError::validation(
                        format!("macros[{macro_index}].steps[{step_index}].timeout_ms"),
                        "timeout must be greater than zero",
                    ));
                }
                if step.idle_ms == 0 {
                    return Err(AutomationError::validation(
                        format!("macros[{macro_index}].steps[{step_index}].idle_ms"),
                        "idle boundary must be greater than zero",
                    ));
                }
                if step.max_bytes == 0 {
                    return Err(AutomationError::validation(
                        format!("macros[{macro_index}].steps[{step_index}].max_bytes"),
                        "max bytes must be greater than zero",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_send_step(
    macro_index: usize,
    step_index: usize,
    step: &SendStep,
) -> AutomationResult<()> {
    let field = format!("macros[{macro_index}].steps[{step_index}].data");
    step.encoding.decode(&step.data, &field)?;
    Ok(())
}

fn validate_assembly_steps(
    assembly_index: usize,
    steps: &[AssemblyStep],
    macro_names: &HashSet<&str>,
) -> AutomationResult<()> {
    for (step_index, step) in steps.iter().enumerate() {
        let AssemblyStep::Macro(call) = step;
        if !macro_names.contains(call.name.as_str()) {
            return Err(AutomationError::validation(
                format!("assemblies[{assembly_index}].steps[{step_index}].name"),
                format!("unknown macro reference {}", call.name),
            ));
        }
    }
    Ok(())
}

fn require_non_empty(field: &str, value: &str) -> AutomationResult<()> {
    if value.trim().is_empty() {
        return Err(AutomationError::validation(
            field,
            "value must not be empty",
        ));
    }
    Ok(())
}
