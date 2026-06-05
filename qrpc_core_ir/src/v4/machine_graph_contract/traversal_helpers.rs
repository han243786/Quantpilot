use super::V4MachineContract;

pub(in crate::v4) fn collect_machine_family<'a>(
    machine: &'a V4MachineContract,
    out: &mut Vec<&'a V4MachineContract>,
) {
    out.push(machine);
    for state in &machine.states {
        if let Some(child_machine) = state.child_machine.as_deref() {
            collect_machine_family(child_machine, out);
        }
    }
}

pub(in crate::v4) fn machine_nested_depth(machine: &V4MachineContract) -> u32 {
    let child_depth = machine
        .states
        .iter()
        .filter_map(|state| state.child_machine.as_deref())
        .map(machine_nested_depth)
        .max()
        .unwrap_or(0);
    1 + child_depth
}
