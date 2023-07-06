fn main() {

	let my_string = String::from("hello world");

	let word = first_word(&my_string[..]);

	println!("{}", word);
}

fn first_word(s: &str) -> &str {
	let bytes = s.as_bytes();

	for (i, &item) in bytes.iter().enmuerate() {
		//b 代表一个字节 即65
		if item == b' ' {

			return &s[0..i]
		}
	}
	&s[..]
}
