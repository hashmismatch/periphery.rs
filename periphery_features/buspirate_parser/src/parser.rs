use nom::{digit, hex_digit, space, IResult};

use std::str::from_utf8;

fn is_binary(chr: u8) -> bool {
	chr == ('0' as u8) || chr == ('1' as u8)
}

named!(binary, take_while!(is_binary));

named!(u8_dec < u8 >,
	map_res!(map_res!(digit, from_utf8), |v: &str| v.parse::<u8>() )
);

named!(u8_hex < u8 >,
	do_parse!(
		tag!("0x") >>
		v: map_res!(map_res!(hex_digit, from_utf8), |v| <u8>::from_str_radix(v, 16)) >>
		(v)
	)
);

named!(u8_bin < u8 >,
	do_parse!(
		tag!("0b") >>
		v: map_res!(map_res!(binary, from_utf8), |v| <u8>::from_str_radix(v, 2)) >>
		(v)
	)
);

named!(u8_any < u8 >,
	alt!(u8_bin | u8_hex | u8_dec)
	);


named!(u8_any_with_ws < u8 >,
	ws!(u8_any)
);

named!(u8_array < Vec<u8> >,
		do_parse!(
            l: separated_list_complete!(tag!(","), u8_any_with_ws) >>
            eof!() >>

            (l)
        )
	);


#[derive(Copy, Clone, Debug)]
pub enum ParserError {
    Invalid
}

pub fn parse_u8_array(input: &str) -> Result<Vec<u8>, ParserError> {
	match u8_array(input.trim().as_bytes()) {
		IResult::Done(i, o) => {
			Ok(o)
		},
		IResult::Error(e) => {
			Err(ParserError::Invalid)
		},
		_ => {
			Err(ParserError::Invalid)
		}
	}
}


#[test]
#[cfg(test)]
fn test_u8_parser() {
	
	{		
		let n = u8_hex(&b"0x00"[..]).unwrap();
		assert_eq!(0, n.1);

		let n = u8_hex(&b"0x05"[..]).unwrap();
		assert_eq!(5, n.1);

		let n = u8_hex(&b"0xFF"[..]).unwrap();
		assert_eq!(255, n.1);

		let n = u8_hex(&b"0xf"[..]).unwrap();
		assert_eq!(15, n.1);

        assert!(u8_hex(&b"0xgg"[..]).is_err());
        assert!(u8_hex(&b"0y00"[..]).is_err());
        assert!(u8_hex(&b"0x260"[..]).is_err());
	}

	
	{
		let n = u8_dec(&b"255"[..]).unwrap();
		assert_eq!(255, n.1);

		let n = u8_dec(&b"0255"[..]).unwrap();
		assert_eq!(255, n.1);

		let n = u8_dec(&b"0"[..]).unwrap();
		assert_eq!(0, n.1);
	}

	
	{
		let n = u8_bin(&b"0b11"[..]).unwrap();
		assert_eq!(3, n.1);

		let n = u8_bin(&b"0b10000000"[..]).unwrap();
		assert_eq!(128, n.1);
	}

	{
		let n = u8_any_with_ws(&b" 100 "[..]).unwrap();
		assert_eq!(100, n.1);

		let n = u8_any_with_ws(&b" 		0xFF  "[..]).unwrap();
		assert_eq!(255, n.1);

        assert!(u8_any_with_ws(&b"test"[..]).is_err());
	}

	
	{
		let r = u8_array(&b"0xFF, 0b1,   3, 4"[..]);
		assert_eq!(&[255, 1, 3, 4], r.unwrap().1.as_slice());
		
		let r = u8_array(&b"0xFF"[..]).unwrap();
		assert_eq!(&[255], r.1.as_slice());

        let r = u8_array(&b"0xFF, 0b1, test,  3, 4"[..]);
		assert!(r.is_err());

        let r = u8_array(&b"1 2 3 4"[..]);
		assert!(r.is_err());
	}
	

	
    /*
	{
		let r = read_operation(&b"r"[..]).unwrap().1;
		assert_eq!(1, r);

		let r = read_operation(&b"r:2"[..]).unwrap().1;
		assert_eq!(2, r);

		let r = read_operation(&b"r:255"[..]).unwrap().1;
		assert_eq!(255, r);
	}

	
    
	{
		let s = "  [ 0x8F, 255 1, 0b11 r:2 rr ]  ";
		let o = [BusOperation::ChipSelect, BusOperation::WriteBytes(vec![143, 255]), BusOperation::WriteBytes(vec![1, 3]), BusOperation::ReadBytes(2), BusOperation::ReadBytes(1), BusOperation::ReadBytes(1), BusOperation::ChipDeselect];

		let p = transaction(s.as_bytes()).unwrap().1;
		assert_eq!(&o, p.as_slice());
	}
    */

  {
    let s = " 10";
    let p = parse_u8_array(&s).unwrap();
    assert_eq!(&[10], p.as_slice());

    assert_eq!(&[1, 2, 3], parse_u8_array("0b1,    0x2,3").unwrap().as_slice());

    assert!(parse_u8_array(&"1, test").is_err());
    assert!(parse_u8_array(&"1, 2, 3, 0y1").is_err());
  }
}







#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BusOperation {
	ChipSelect,
	ChipDeselect,
	
	ChipSelectDebug,
	ChipDeselectDebug,

	WriteBytes(Vec<u8>),
	ReadBytes(u8)
}


