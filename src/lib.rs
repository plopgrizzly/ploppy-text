extern crate futures;
extern crate native_tls;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_tls;
extern crate base64;
extern crate json;

use std::io;
use std::net::ToSocketAddrs;
use std::fmt;

use futures::Future;
use native_tls::TlsConnector;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;
use tokio_tls::TlsConnectorExt;
use base64::{encode_config, URL_SAFE};
use json::JsonValue;

pub enum Error {
	Connecting,
	Response,
}

pub struct PhoneDetails {
	number: String,
	caller_id: String,
	caller_type: String,
	country: String,
	carrier: String,
	is_landline: bool,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Error::Connecting => write!(f, "Error Connecting"),
			Error::Response => write!(f, "Error In Response"),
		}
	}
}

impl PhoneDetails {
	pub fn number(&self) -> &str {
		&self.number
	}

	pub fn caller_id(&self) -> &str {
		&self.caller_id
	}

	pub fn caller_type(&self) -> &str {
		&self.caller_type
	}

	pub fn country(&self) -> &str {
		&self.country
	}

	pub fn carrier(&self) -> &str {
		&self.carrier
	}

	pub fn is_landline(&self) -> bool {
		self.is_landline
	}
}

fn https_get(website: &str, path: &str, authorization: &str) -> String {
	let mut core = Core::new().unwrap();
	let handle = core.handle();
	let addr = format!("{}:443", website).to_socket_addrs().unwrap().next().unwrap();

	let cx = TlsConnector::builder().unwrap().build().unwrap();
	let socket = TcpStream::connect(&addr, &handle);

	let tls_handshake = socket.and_then(|socket| {
		let tls = cx.connect_async(website, socket);
		tls.map_err(|e| {
			io::Error::new(io::ErrorKind::Other, e)
		})
	});
	let authorization = encode_config(authorization, URL_SAFE);
	let textreq = format!("\
		GET {} HTTP/1.0\r\n\
		Host: {}\r\n\
		Authorization: Basic {}\r\n\
		\r\n\
	", path, website, authorization);
	let request = tls_handshake.and_then(|socket| {
		tokio_io::io::write_all(socket, textreq.as_bytes())
	});
	let response = request.and_then(|(socket, _request)| {
		tokio_io::io::read_to_end(socket, Vec::new())
	});

	let (_socket, data) = core.run(response).unwrap();

	let received = String::from_utf8_lossy(&data);
	let mut iterator = received.split("\r\n\r\n");

	iterator.nth(1).unwrap().to_string()
}

fn json_get(object: &JsonValue, key: &str) -> Result<String, Error> {
	let value = json_get_value(object, key)?;

	if let JsonValue::Short(string) = value {
		Ok(string.as_str().to_string())
	} else if let JsonValue::Null = value {
		Ok("Unknown".to_string())
	} else {
		Err(Error::Response)
	}
}

fn json_get_value(object: &JsonValue, key: &str) -> Result<JsonValue, Error> {
	if let JsonValue::Object(ref obj) = *object {
		if let Some(value) = obj.get(key) {
			Ok(value.clone())
		} else {
			Err(Error::Response)
		}
	} else {
		Err(Error::Response)
	}
}

pub fn get_phone_details(number: &str) -> Result<PhoneDetails, Error> {
	let number = number.to_string();

	let message = https_get("lookups.twilio.com",
		&format!("/v1/PhoneNumbers/{}?Type=carrier&Type=caller-name",
			number),
		"ACc56ff3886235ff3df8ffd780d8c72bb4:\
			15aa6f51504a1dab62ea0f00e1a307c9");

	let phone_details = if let Ok(details) = json::parse(&message) {
		details
	} else {
		return Err(Error::Response)
	};

	let country = json_get(&phone_details, "country_code")?;

	let group_caller = json_get_value(&phone_details, "caller_name")?;

	let caller_id = json_get(&group_caller, "caller_name")?;
	let caller_type = json_get(&group_caller, "caller_type")?;

	let group_carrier = json_get_value(&phone_details, "carrier")?;
	let carrier = json_get(&group_carrier, "name")?;
	let is_landline = json_get(&group_carrier, "type")? == "landline";

	Ok(PhoneDetails {
		number, caller_id, caller_type, country, carrier, is_landline,
	})
}

/// Get the email address for sending a text to the number.  If possible, it
/// will return the address for MMS messages, otherwise it'll return the SMS
/// email.
pub fn get_texting_email(details: PhoneDetails) -> Option<String> {
	let mut email = String::new();

	email.push_str(&details.number);
	email.push('@');
	email.push_str(
		match &details.carrier[..] {
			"Verizon Wireless" => "vzwpix.com",
//			Virgin Mobile USA => "vmobl.com",
//			US Cellular => "mms.uscc.net",
//			T-Mobile => "tmomail.net",
//			Sprint PCS (now Sprint Nextel) => "pm.sprint.com",
//			Nextel (now Sprint Nextel) => "messaging.nextel.com",
//			Boost Mobile => "myboostmobile.com",
//			AT&T (formerly Cingular) => "mms.att.net",
//			Alltel => "message.alltel.com",
//			Cricket => "mms.mycricket.com",
//			Metro PCS => "mymetropcs.com",
//			Straight Talk => "mypixmessages.com",
			_ => return None
		}
	);

	Some(email)
}
