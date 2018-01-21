use prelude::v1::*;
use base::*;

pub fn print_buspirate_help<F: FnMut(&str) -> ()>(line_printer: F) {
	let mut l = line_printer;
	l("[      Chip select enable.");
	l("{      Chip select enable, show the read SPI byte after every write.");
	l("]      Chip select disable.");
	l("}      Chip select disable.");
	l("r      Read one byte by sending dummy byte (0xFF).");
	l("       Use \"r:1...255\" for bulk reads.");
	l("0b     Write this binary value. Format is 0b00000000.");
	l("       Partial bytes are also fine: 0b1001.");
	l("0x     Write this hex value. Format is 0x01. Partial bytes are fine: 0xA.");
	l("       A-F can be lower or upper case.");
	l("0-255  Write this decimal value.");
	l(",      Value delimiter. Use a comma or a space to separate numbers.");
}

