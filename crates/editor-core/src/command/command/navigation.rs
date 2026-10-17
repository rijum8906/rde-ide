/// Tab and buffer navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationCommand {
    NextTab,
    PrevTab,
    SelectTab(usize),
    CloseTab,
    CloseAllTabs,
    NextBuffer,
    PrevBuffer,
    FocusEditor,
    FocusLeftPanel,
    FocusRightPanel,
    FocusNextPane,
}
