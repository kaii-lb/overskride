use async_channel::Sender;
use bluer::Address;
use crate::message::Message;

pub struct OverskrideProperties {
    pub name: String,
    pub current_adapter: String,
    pub sender: Option<Sender<Message>>,
    pub current_index: i32,
    pub address: Address,
    pub auto_accept_first: bool,
    pub auto_accept_from_trusted: bool,
    pub hide_unknown_devices: bool,
    pub send_files_path: Vec<String>,
    pub displaying_dialog: bool,
    pub pin_code: String,
    pub pass_key: u32,
    pub store_folder: String,
    pub confirm_authorization: bool
}

impl OverskrideProperties {
    pub(crate) fn new() -> Self {
        let empty_string = "".to_string();
        OverskrideProperties {
            name: empty_string.to_string(),
            current_adapter: empty_string.to_string(),
            sender: None,
            current_index: 0,
            address: Address::any(),
            auto_accept_first: true,
            auto_accept_from_trusted: false,
            hide_unknown_devices: true,
            send_files_path: vec![],
            displaying_dialog: false,
            pin_code: empty_string.to_string(),
            pass_key: 0,
            store_folder: empty_string,
            confirm_authorization: false
        }
    }
}