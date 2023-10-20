use std::collections::HashMap;

pub enum Message {
    #[allow(dead_code)]
    /// Changes the trusted switch's active to `bool`
    SwitchTrusted(bool),
    /// Changes the blocked switch's active to `bool`
    SwitchBlocked(bool),
    /// Changes the connected switch's active to `bool`
    SwitchActive(bool),
    /// Changes the connected swtich's spinner spinning state to `bool`
    SwitchActiveSpinner(bool),
    /// Changes the active devices's name to [alias](String) if no [old_alias](Option<String>) is provided, otherwise it looks for a matching row
    SwitchName(String, Option<String>),
    /// Changes the supplied device's (`name: String`) RSSI to the supplied value (`rssi: i32`)
    SwitchRssi(String, i32),
    /// Moves between pages, ie changes the values of the rows and icons to `page: Option<String>` and `icon: Option<String>`
    SwitchPage(Option<String>, Option<String>),
    /// Removes the device matching the supplied name (`name: String`)
    RemoveDevice(String, bluer::Address),
    /// Adds a new device from the properties of [device](bluer::Device)
    AddRow(bluer::Device),
    /// Changes the adapter's powered state to `bool`
    SwitchAdapterPowered(bool),
    /// Changes the adapter's discoverable timeout to [timeout](u32)
    SwitchAdapterTimeout(u32),
    /// Changes the adapter's discoverable state to `bool`
    SwitchAdapterDiscoverable(bool),
    /// Changes the adapter's name to [alias](String), taking in [old_alias](String) for reference
    SwitchAdapterName(String, String),
    /// Adds all currently discovered adapters from [adapters_lut](HashMap) to the adapter list in the settings
    PopulateAdapterExpander(HashMap<String, String>),
    /// Displays an error (or a message) of [message](String) with a [priority](adw::ToastPriority) as a [toast](adw::Toast)
    PopupError(String, adw::ToastPriority),
    /// Checks if there are devices and changes the "no bluetooth devices found" image accordingly
    UpdateListBoxImage(),
    /// Requests a pairing pincode using [request](bluer::agent::RequestPinCode) as input
    RequestPinCode(bluer::agent::RequestPinCode),
    /// Displays a pairing pincode using [request](bluer::agent::DisplayPinCode) as input
    DisplayPinCode(bluer::agent::DisplayPinCode),
    /// Requests a pairing passkey using [request](bluer::agent::RequestPasskey) as input
    RequestPassKey(bluer::agent::RequestPasskey),
    /// Displays a pairing passkey using [request](bluer::agent::RequestPasskey) as input
    DisplayPassKey(bluer::agent::DisplayPasskey),
    /// Requests pairing confirmation using [request](bluer::agent::RequestConfirmation) as input
    RequestConfirmation(bluer::agent::RequestConfirmation),
    /// Requests pairing authorization using [request](bluer::agent::RequestAuthorization) as input
    RequestAuthorization(bluer::agent::RequestAuthorization),
    /// Requests service authorization using [request](bluer::agent::AuthorizeService) as input
    AuthorizeService(bluer::agent::AuthorizeService),
    /// Goes the the settings page or the last device depending on `bool`
    GoToBluetoothSettings(bool),
    /// Gets a `yes/no` answer from a dialog 
    /// ### Arguments
    /// * `title` - a short [String](String) descibing the request
    /// * `subtitle` - a [String](String) describing the request in more detail
    /// * `confirm name` - a [String](String) for the name of the confirmation option
    /// * `response type` - a [Response Type](adw::ResponseAppearance) detailing if the response is destructive, suggested, etc
    RequestYesNo(String, String, String, adw::ResponseAppearance),
    /// Invalidates the device list's sorting, forcing it to resort the devices according to various factors
    InvalidateSort(),
    /// Forcefully refreshes the device list (needs more work)
    RefreshDevicesList(),
    /// Starts a new transfer, displaying a progress bar popover with the filename
    /// ### Arguments
    /// * `trasnfer` - a [String](String) containing the transfer object 
    /// * `filename` - a [String](String) ...which is the filename
    /// * `percent` - a [f32](f32) the starting completion percent (like 45 **not** 0.45)
    /// * `current mb` - a [f32](f32) the current transferred megabytes
    /// * `filesize` - a [f32](f32) the total filesize in megabytes
    /// * `outbound` - a [bool](bool) indicating if the transfer is sending or receiving
    StartTransfer(String, String, f32, f32, f32, bool),
    /// Updates the transfer's progression based on:
    /// ### Arguments
    /// * `transfer` - a [String](String) containing the transfer object
    /// * `filename` - a [String](String) containing the file name
    /// * `current mb` - a [String](String) the current transferred megabytes
    /// * `status` - a [String](String) which is the current status of the transfer (ie: complete, error, active...)
    UpdateTransfer(String, String, f32, String),
    /// Removes a transfer via the supplied [transfer](String) object and the [filename](String) incase of multiple files in same transfer
    RemoveTransfer(String, String),
    /// Gets the path of a selected file or folder, based on [filetype](gtk::FileChooserAction)
    GetFile(gtk::FileChooserAction),
    /// Sets the sensitive state of the send file row, aka if it is interactable
    SwitchSendFileActive(bool),
    /// Sets the new [file storage location](String), doing some checks along the way
    SetFileStorageLocation(String),
	/// Changes whether the current device has obex capabilites or not
	SwitchHasObexService(bool),
} 
