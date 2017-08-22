extern crate futures;
extern crate native_tls;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_tls;
extern crate base64;
extern crate json;
extern crate lettre;
extern crate openssl;
extern crate adi_storage;

use std::io;
use std::net::ToSocketAddrs;
use std::fmt;
use std::error::Error;
use std::env;

use futures::Future;
use native_tls::TlsConnector;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;
use tokio_tls::TlsConnectorExt;
use base64::{encode_config, URL_SAFE};
use json::JsonValue;
use lettre::email::EmailBuilder;
use lettre::transport::smtp;
use lettre::transport::smtp::{SecurityLevel, SmtpTransport, SmtpTransportBuilder};
use lettre::transport::EmailTransport;
use lettre::transport::smtp::authentication::Mechanism;
use lettre::transport::smtp::client::Client;
use lettre::transport::smtp::client::net::NetworkStream;
use lettre::transport::smtp::SMTP_PORT;
use openssl::ssl::{ SslContextBuilder, SslMethod };

pub enum PError {
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

impl fmt::Display for PError {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			PError::Connecting => write!(f, "Error Connecting"),
			PError::Response => write!(f, "Error In Response"),
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

fn https_get(website: &str, path: &str/*, authorization: &str*/) -> String {
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
//	let authorization = encode_config(authorization, URL_SAFE);
	let textreq = format!("\
		GET {} HTTP/1.0\r\n\
		Host: {}\r\n\
		\r\n", /* Authorization: Basic {}\r\n\ */
		path, website/*, authorization*/);
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

fn json_get(object: &JsonValue, key: &str) -> Result<String, PError> {
	let value = json_get_value(object, key)?;

	if let JsonValue::Short(string) = value {
		Ok(string.as_str().to_string())
	} else if let JsonValue::Null = value {
		Ok("Unknown".to_string())
	} else {
		Err(PError::Response)
	}
}

fn json_get_value(object: &JsonValue, key: &str) -> Result<JsonValue, PError> {
	if let JsonValue::Object(ref obj) = *object {
		if let Some(value) = obj.get(key) {
			Ok(value.clone())
		} else {
			Err(PError::Response)
		}
	} else {
		Err(PError::Response)
	}
}

/*pub fn get_phone_details(number: &str) -> Result<PhoneDetails, PError> {
	let number = number.to_string();

	let message = https_get("lookups.twilio.com",
		&format!("/v1/PhoneNumbers/{}?Type=carrier&Type=caller-name",
			number),
		"ACc56ff3886235ff3df8ffd780d8c72bb4:\
			15aa6f51504a1dab62ea0f00e1a307c9");

	let phone_details = if let Ok(details) = json::parse(&message) {
		details
	} else {
		return Err(PError::Response)
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
}*/

/*pub fn get_texting_email(details: PhoneDetails) -> Option<String> {
	let mut email = String::new();

	email.push_str(&details.number);
	email.push('@');
	email.push_str(
		match &details.carrier[..] {
			_ => return None
		}
	);

	Some(email)
}*/

pub fn send_email(string: &str, username: &str, password: &str) {
	let email = EmailBuilder::new()
		.to("jeronlau.5@gmail.com")
		.from("user@localhost")
		.body("Hello World!")
		.subject("Hello")
		.build()
		.unwrap();

	let mut transport = smtp::SmtpTransportBuilder::new(
		("smtp.gmail.com", smtp::SUBMISSION_PORT))
		.expect("Failed to create transport")
   	 	.credentials(username, password)
		.build();

	println!("{:?}", transport.send(email.clone()));
}

/// Get the email address for sending an SMS text to the number.
pub fn find_texting_email(number: &str, username: &str, password: &str) {
	const CARRIERS: [&'static str; 12] = [
		"message.alltel.com", // Alltel
		"txt.att.net", // AT&T
		"cingularme.com", // Cingular
		"myboostmobile.com", // Boost Mobile
		"messaging.nextel.com", // Nextel
		"messaging.sprintpcs.com", // Sprint PCS
		"tmomail.net", // T Mobile
		"email.uscc.net", // US Cellular
		"vmobl.com", // Virgin Mobile USA
		"sms.mycricket.com", // Cricket
		"mymetropcs.com", // MetroPCS
		"vtext.com", // Verizon
	];

	let mut transport = smtp::SmtpTransportBuilder::new(
		("smtp.gmail.com", smtp::SUBMISSION_PORT))
		.expect("Failed to create transport")
   	 	.credentials(username, password)
//		.security_level(SecurityLevel::AlwaysEncrypt)
//		.smtp_utf8(true)
//		.authentication_mechanism(Mechanism::CramMd5)
		.connection_reuse(true).build();

	for i in CARRIERS.iter() {
/*		let mut email_client: Client<NetworkStream> = Client::new();

		email_client.connect(&("vtext.com", 25),
			Some(&SslContextBuilder::new(SslMethod::tls()).unwrap()
				.build())).unwrap();
//		email_client.ehlo("hi").unwrap();
//		email_client.mail("youremail@gmail.com", None).unwrap();
//		email_client.rcpt("mailbox.does.not.exist@webdigiapps.com").unwrap();
		let a = email_client.vrfy("jeronlau.5@gmail.com").unwrap();
		email_client.quit().unwrap();

*/		let email = EmailBuilder::new()
			.to(&format!("{}@{}", number, i)[..])
			.from("user@localhost")
			.body("Welcome To Ploppy Text.  Please verify that you \
				have received this message.")
			.build()
			.unwrap();

		if let Ok(eee) = transport.send(email) {
			if eee.is_positive() {
				println!("Positive {}", i);

				println!("SEVERITY {}", eee.severity());
				println!("DETAIL {}", eee.detail());
				println!("\"{:?}\"", eee.message());

				break;
			} else {
				println!("Negative {}", i);
			}
		} else {
			println!("Not {}", i);
		}
	}
}

pub fn send(list: Vec<String>, message: &str, username: &str, password: &str) {
	let mut transport = smtp::SmtpTransportBuilder::new(
		("smtp.gmail.com", smtp::SUBMISSION_PORT))
		.expect("Failed to create transport")
   	 	.credentials(username, password)
		.connection_reuse(true).build();

	for i in list {
		let email = EmailBuilder::new().to(&i[..])
			.from("user@localhost").body(message).build().unwrap();

		if transport.send(email).is_ok() {
			println!("{}: Succeeded Sent Message", i);
		} else {
			println!("{}: Failed to Send Message", i);
		}
	}
}

fn generate_texting_email(number: String, carrier: String) -> String {
/*	const CARRIERS: [&'static str; _] = [
		"message.alltel.com", // Alltel
		"cingularme.com", // Cingular
		"myboostmobile.com", // Boost Mobile
		"messaging.nextel.com", // Nextel
		"email.uscc.net", // US Cellular
		"vmobl.com", // Virgin Mobile USA
		"sms.mycricket.com", // Cricket
		"mymetropcs.com", // MetroPCS
	];*/

	let domain = match &carrier[..] {
		"Verizon Wireless" => "vtext.com",
		"Sprint Spectrum, L.P." => "messaging.sprintpcs.com",
		"AT&T Wireless" => "txt.att.net",
		"T-Mobile USA, Inc." => "tmomail.net",
		a => panic!("Carrier Unknown: {}", a),
	};

	format!("{}@{}", number, domain)
}

pub fn load_emails(name: &str) -> Vec<String> {
	let home = match env::home_dir() {
		Some(path) => path.display().to_string(),
		None => panic!("Impossible to get your home dir!"),
	};

	let a = String::from_utf8(adi_storage::load(
		format!("{}/.ploppy-text", home))).unwrap();
	let b = a.split("#");
	let mut c = Vec::new();

	for i in b {
		if i.starts_with(name) {
			let mut ii = i.split('\n');

			ii.next();
			for j in ii {
				let mut jj = j.to_string();

				if jj.contains(' ') {
					let l = jj.find(' ').unwrap();
					let mut k = jj.split_off(l);

					k.remove(0); // remove the space

					jj = generate_texting_email(jj, k);
				}

				c.push(jj);
			}

			c.retain(|a| !a.is_empty());

			return c;
		}
	}

	panic!("Couldn't Find Group {}", name);
}
