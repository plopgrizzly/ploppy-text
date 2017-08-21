extern crate ploppy_text;

use std::env;
use std::panic;

fn get(number: &str) {
	let phone_details = match ploppy_text::get_phone_details(number) {
		Ok(details) => details,
		Err(error) => panic!("Error: {}", error),
	};

	println!("Phone #: {}", phone_details.number());
	println!("Caller ID: {}", phone_details.caller_id());
	println!("Caller Type: {}", phone_details.caller_type());
	println!("Country: {}", phone_details.country());
	println!("Carrier: {}", phone_details.carrier());
	println!("Is Landline: {}", phone_details.is_landline());

	println!("Email: {:?}", ploppy_text::get_texting_email(phone_details))
}

fn main() {
	let args : Vec<String> = env::args().collect();

	let result = panic::catch_unwind(|| {
		match &args[1][..] {
			"get" => get(&args[2]),
			"help" => println!("ploppy_text get PHONENUMBER"),
			_ => println!("try: `ploppy_text help`"),
		}
	});

	if result.is_err() {
		println!("\ntry: `ploppy_text help`");
	}
}
