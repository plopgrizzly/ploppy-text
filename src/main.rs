extern crate ploppy_text;

use std::env;
use std::panic;

fn get(number: &str, email: &str, password: &str) {
/*	let phone_details = match ploppy_text::get_phone_details(number) {
		Ok(details) => details,
		Err(error) => panic!("Error: {}", error),
	};

	println!("Phone #: {}", phone_details.number());
	println!("Caller ID: {}", phone_details.caller_id());
	println!("Caller Type: {}", phone_details.caller_type());
	println!("Country: {}", phone_details.country());
	println!("Carrier: {}", phone_details.carrier());
	println!("Is Landline: {}", phone_details.is_landline());*/

//	println!("Email: {:?}", ploppy_text::get_texting_email(phone_details))
	println!("find_texting_email: {:?}",
		ploppy_text::find_texting_email(number, email, password));
}

fn members(group: &str) {
	let list = ploppy_text::load_emails(group);

	println!("{:?}", list);
}

fn send(group: &str, message: &str, email: &str, password: &str) {
	let list = ploppy_text::load_emails(group);

	ploppy_text::send(list, message, email, password);
}

fn help() {
	println!("ploppy-text get PHONENUMBER EMAIL PASSWORD");
	println!("ploppy-text send GROUP MESSAGE EMAIL PASSWORD");
	println!("ploppy-text members GROUP");
}

fn main() {
	let args : Vec<String> = env::args().collect();

	let result = panic::catch_unwind(|| {
		match &args[1][..] {
			"members" => members(&args[2]),
			"send" => send(&args[2], &args[3], &args[4], &args[5]),
			"get" => get(&args[2], &args[3], &args[4]),
			"help" => help(),
			_ => println!("try: `ploppy-text help`"),
		}
	});

	if result.is_err() {
		println!("\ntry: `ploppy-text help`");
	}
}
