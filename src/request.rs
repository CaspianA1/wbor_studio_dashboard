use crate::utility_types::generic_result::GenericResult;

fn check_request_failure<T: std::fmt::Display + std::cmp::PartialEq>
	(value_name: &str, url: &str, expected: T, gotten: T) -> GenericResult<()> {

	if expected != gotten {
		return Err(
			format!("Response {} for URL '{}' was not '{}', but '{}'",
			value_name, url, expected, gotten).into()
		)
	}

	Ok(())
}

pub fn build_url(base_url: &str, path_params: Vec<String>,
	query_params: Vec<(&str, String)>) -> GenericResult<String> {

	let mut url = Vec::new();

	let mut add_str_to_url =
		|s: &str| url.append(&mut s.to_string().into_bytes());

	//////////

	add_str_to_url(base_url);

	for path_param in path_params {
		add_str_to_url(&format!("/{}", path_param));
	}

	for (index, query_param) in query_params.iter().enumerate() {
		let separator = if index == 0 {'?'} else {'&'};
		let query = format!("{}{}={}", separator, query_param.0, query_param.1);
		add_str_to_url(&query);
	}

	Ok(String::from_utf8(url)?)
}

pub fn get_with_maybe_header(url: &str, maybe_header: Option<(&str, &str)>) -> GenericResult<minreq::Response> {
	let mut request = minreq::get(url);

	if let Some(header) = maybe_header {
		request = request.with_header(header.0, header.1);
	}

	let response = request.send()?;

	check_request_failure("status code", url, 200, response.status_code)?;
	check_request_failure("reason phrase", url, "OK", &response.reason_phrase)?;

	Ok(response)
}

pub fn get(url: &str) -> GenericResult<minreq::Response> {
	get_with_maybe_header(url, None)
}
