//! Buffer, tab, and panel-focus commands.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceCommand {
    // --- Buffer Management (Document Level) ---
    NextBuffer,
    PrevBuffer,
    CloseBuffer,
    CloseBufferForce,
    SwitchBuffer(usize), // Or by name/id

    // --- Panel / Focus Management (UI Level) ---
    FocusEditorPanel,
    FocusLeftPanel,
    FocusRightPanel,
    ToggleFileTree,
}
