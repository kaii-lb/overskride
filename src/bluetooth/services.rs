use phf::phf_map;
use bluer::Uuid;

const SERVICES: phf::Map<&'static str, &'static str> = phf_map! {
	"00001203-0000-1000-8000-00805f9b34fb" => "Generic Audio",
	"00001108-0000-1000-8000-00805f9b34fb" => "Hands Free Profile",
	"0000111e-0000-1000-8000-00805f9b34fb" => "Hands Free Profile",
	"00001112-0000-1000-8000-00805f9b34fb" => "Hands Free Profile Audio Gateway",
	"0000111f-0000-1000-8000-00805f9b34fb" => "Hands Free Profile Audio Gateway",
	"0000110d-0000-1000-8000-00805f9b34fb" => "Advanced Audio",
	"0000110a-0000-1000-8000-00805f9b34fb" => "A2DP Source",
	"0000110b-0000-1000-8000-00805f9b34fb" => "A2DP Sink",
	"0000110e-0000-1000-8000-00805f9b34fb" => "Audio / Video Remote Control",
	"0000110c-0000-1000-8000-00805f9b34fb" => "Audio / Video Remote Control Target",
	"00001115-0000-1000-8000-00805f9b34fb" => "Personal Area Networking User",
	"00001116-0000-1000-8000-00805f9b34fb" => "Network Access Point",
	"00001117-0000-1000-8000-00805f9b34fb" => "Group ad-hoc Network",
	"0000000f-0000-1000-8000-00805f9b34fb" => "Bluetooth Network Encapsulation Protocol",
	"00002a50-0000-1000-8000-00805f9b34fb" => "Part Number and Product ID",
	"0000180a-0000-1000-8000-00805f9b34fb" => "Device Information",
	"00001801-0000-1000-8000-00805f9b34fb" => "Generic Attribute Profile",
	"00001802-0000-1000-8000-00805f9b34fb" => "Immediate Alert",
	"00001803-0000-1000-8000-00805f9b34fb" => "Link Loss",
	"00001804-0000-1000-8000-00805f9b34fb" => "Transmit Power",
	"0000112D-0000-1000-8000-00805f9b34fb" => "SIM Access Profile",
	"0000180d-0000-1000-8000-00805f9b34fb" => "Heart Rate",
	"00002a37-0000-1000-8000-00805f9b34fb" => "Heart Rate Measurement", 
	"00002a38-0000-1000-8000-00805f9b34fb" => "Body Sensor Location",
	"00002a39-0000-1000-8000-00805f9b34fb" => "Heart Rate Control Point",
	"00001809-0000-1000-8000-00805f9b34fb" => "Health Thermometer",
	"00002a1c-0000-1000-8000-00805f9b34fb" => "Temperature Measurement",
	"00002a1d-0000-1000-8000-00805f9b34fb" => "Temperature Type",
	"00002a1e-0000-1000-8000-00805f9b34fb" => "Intermediate Temperature",
	"00002a21-0000-1000-8000-00805f9b34fb" => "Measurement Interval",
	"00001816-0000-1000-8000-00805f9b34fb" => "Cycling Speed and Cadence",
	"00002a5b-0000-1000-8000-00805f9b34fb" => "Cycling Speed and Cadence Measurement",
	"00002a5c-0000-1000-8000-00805f9b34fb" => "Cycling Speed and Cadence Feature",
	"00002a5d-0000-1000-8000-00805f9b34fb" => "Sensor Location",
	"00002a55-0000-1000-8000-00805f9b34fb" => "Speed and Cadence Control Point",
	"00000003-0000-1000-8000-00805f9b34fb" => "Serial port transport protocol (rfcomm)",
	"00001400-0000-1000-8000-00805f9b34fb" => "Health Device",
	"00001401-0000-1000-8000-00805f9b34fb" => "Health Device Source",
	"00001402-0000-1000-8000-00805f9b34fb" => "Health Device Sink",
	"00001124-0000-1000-8000-00805f9b34fb" => "Human Interface Device",
	"00001103-0000-1000-8000-00805f9b34fb" => "Dial-Up Networking Gateway",
	"00001800-0000-1000-8000-00805f9b34fb" => "Generic Access Profile",
	"00001200-0000-1000-8000-00805f9b34fb" => "Plug and Play",
	"00001101-0000-1000-8000-00805f9b34fb" => "Serial Port",
	"00001104-0000-1000-8000-00805f9b34fb" => "Obex Sync",
	"00001105-0000-1000-8000-00805f9b34fb" => "Obex Object Push Profile",
	"00001106-0000-1000-8000-00805f9b34fb" => "Obex File Transfer Protocol",
	"0000112e-0000-1000-8000-00805f9b34fb" => "Phone Book Client Equipment",
	"0000112f-0000-1000-8000-00805f9b34fb" => "Phone Book Server Equipment",
	"00001130-0000-1000-8000-00805f9b34fb" => "Phone Book Access",
	"00001132-0000-1000-8000-00805f9b34fb" => "Message Access Service",
	"00001133-0000-1000-8000-00805f9b34fb" => "Message Notification Service",
	"00001134-0000-1000-8000-00805f9b34fb" => "Message Access Profile",
};

pub fn get_name_from_service(service: Uuid) -> Result<String, bluer::Error> {
	let uuid_slice = service.to_string();

	let name = SERVICES.get(uuid_slice.as_str());
	if let Some(service_name) = name {
		Ok(service_name.to_string() + "Profile")
	}
	else {
		Err(bluer::Error { kind: bluer::ErrorKind::Failed, message: "Failed to get name from UUID".to_string() })
	}
}
