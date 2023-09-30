use std::collections::HashMap;

pub enum Message {
    #[allow(dead_code)]
    SwitchTrusted(bool),
    SwitchBlocked(bool),
    SwitchActive(bool),
    SwitchName(String, Option<String>),
    SwitchPage(Option<String>, Option<String>),
    RemoveDevice(String),
    AddRow(bluer::Device),
    SwitchAdapterPowered(bool),
    SwitchAdapterTimeout(u32),
    SwitchAdapterDiscoverable(bool),
    SwitchAdapterName(String, String),
    PopulateAdapterExpander(HashMap<String, String>),
    SetRefreshSensitive(bool),
    PopupError(String),
    UpdateListBoxImage(),
    RequestPinCode(bluer::agent::RequestPinCode),
    DisplayPinCode(bluer::agent::DisplayPinCode),
    RequestPassKey(bluer::agent::RequestPasskey),
    DisplayPassKey(bluer::agent::DisplayPasskey),
    RequestConfirmation(bluer::agent::RequestConfirmation),
    RequestAuthorization(bluer::agent::RequestAuthorization),
    AuthorizeService(bluer::agent::AuthorizeService),
} 