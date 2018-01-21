extern crate crc;

extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate packed_struct;

#[macro_use]
extern crate packed_struct_codegen;


#[macro_use]
extern crate fsm;
#[macro_use]
extern crate fsm_codegen;  

use packed_struct::prelude::*;

pub mod structs;
pub mod protocol_decoder;
pub mod fifo_decoder;
pub mod message_decoder;
pub mod policy_engine;


fn find_crc_packet(data: &[u8]) -> Result<bool, ()> {
    if data.len() < 6 { return Err(()); }
    let l = data.len();
    let payload = &data[..(l-4)];
    
    let crc = &data[(l-4)..];
    let crc_packet: u32 = **<LsbInteger<_, _, Integer<u32, packed_bits::Bits32>>>::unpack_from_slice(&crc).unwrap();

    use crc::{crc32, Hasher32};

    let c = crc32::checksum_ieee(payload);
    if c == crc_packet {
        Ok(true)
    } else {
        Err(())
    }
}


#[test]
fn bf_packets() {
    let traffic = [
        224, 97,
        71, 44, 145, 1, 8, 44, 209, 2, 8, 44, 177, 4, 8, 44, 65, 6, 8, 201, 183, 153, 250, 224, 97, 73, 
        
        44, 145, 1, 8, 44, 
        
        209, 2, 
    ];

    let min = 6;
    for i in 0..traffic.len() - min {
        for j in min..(traffic.len() - i) {
            let data = &traffic[i..(i + j)];
            let f = find_crc_packet(data);
            if let Ok(true) = f {
                println!("found something");
                println!("data: {:?}", data);
            }
        }
    }
}

#[test]
fn test_dec_fusb302() {
    use structs::*;
    
    let traffic = [97, 71, 44, 145, 1, 8, 44, 209, 2, 8, 44, 177, 4, 8, 44, 65, 6, 8, 201, 183, 153, 250];

    let mut i = 0;
    let h = {
        let i = 0;        
        let h = [traffic[i+1], traffic[i]];
        MessageHeader::unpack_from_slice(&h).unwrap().decode().unwrap()
    };
    println!("header: {:#?}", h);
    i += 2;

    if let DecodedMessageHeader::Data(data) = h {
        println!("data header: {:#?}", data);

        //return;

        let b = data.get_number_of_data_bytes();
        let data_bytes = &traffic[i..i+b];
        println!("data: {:?}", data_bytes);


        if data.message_type == DataMessageType::SourceCapabilities {
            //let data_bytes = [8, 1, 144, 240];

            for data_bytes in data_bytes.chunks(4) {

                println!("data len: {:?}", data_bytes.len());
                let data_bytes: Vec<_> = data_bytes.iter().cloned().rev().collect();
                
                let pdo = PowerDataObject::unpack_from_slice(&data_bytes).unwrap();
                println!("pdo: {:#?}", pdo);

                if pdo.supply_kind == SupplyKind::FixedSupply {
                    let pdo_fixed = PowerDataObjectFixed::unpack_from_slice(&data_bytes).unwrap();
                    println!("pdo fixed: {:#?}", pdo_fixed);
                    println!("Max current: {:.2} A", pdo_fixed.get_maximum_current_milli_amperes() as f32 / 1000.0);
                    println!("Voltage: {:.2} V", pdo_fixed.get_voltage_milli_volts() as f32 / 1000.0);
                }
            }
        }


        let packet_size = 2 + data.get_number_of_data_bytes();
    
        //let l = traffic.len();
        
        let crc = &traffic[packet_size..(packet_size + 4)];
        let crc: u32 = **<LsbInteger<_, _, Integer<u32, packed_bits::Bits32>>>::unpack_from_slice(&crc).unwrap();

        println!("crc from the packet: 0x{:X}", crc);
            

        
        let for_crc = &traffic[..packet_size];
        println!("for crc: {:?}", for_crc);

        use crc::{crc32, Hasher32};

        let c = crc32::checksum_ieee(for_crc);
        //let mut digest = crc32::Digest::new(0xEDB88320);
        //digest.write(crc);
        //let c = digest.sum32();
        //assert_eq!(digest.sum32(), 0xcbf43926);

        println!("computed crc: 0X{:X}", c);
        assert_eq!(crc, c);
    }    

    
    

    //println!("traffic: {:?}", traffic);

}

#[test]
fn test_dec_lecroy() {
    use structs::*;

    let traffic = [
        0x1, 0x6, 0x1, 0x1, 
        0x0, 0xF, 0x0, 0x9, 
        0x1, 0x0, 0x8, 0x0, 
        0xA, 0x4, 0x4, 0xD, 
        0xF, 0x5, 0x3, 0x4 
    ];

    let traffic: Vec<_> = traffic.chunks(2).map(|c| {
        (c[1] as u8) << 4 | (c[0] as u8)
    }).collect();

    let l = traffic.len();
    let crc = &traffic[(l-4)..];
    let crc: u32 = **<LsbInteger<_, _, Integer<u32, packed_bits::Bits32>>>::unpack_from_slice(&crc).unwrap();

    println!("crc from the packet: 0x{:X}", crc);


    let mut i = 0;
    let h = {
        let h = &traffic[i..i+2];
        let h: Vec<_> = h.iter().cloned().rev().collect();
        MessageHeader::unpack_from_slice(&h).unwrap().decode().unwrap()
    };
    //println!("header: {:#?}", h);
    i += 2;

    if let DecodedMessageHeader::Data(data) = h {
        println!("data header: {:#?}", data);

        let b = data.get_number_of_data_bytes();
        let data_bytes = &traffic[i..i+b];
        println!("data: {:?}", data_bytes);


        if data.message_type == DataMessageType::SourceCapabilities {
            //let data_bytes = [8, 1, 144, 240];

            let data_bytes: Vec<_> = data_bytes.iter().cloned().rev().collect();
            
            let pdo = PowerDataObject::unpack_from_slice(&data_bytes).unwrap();
            println!("pdo: {:#?}", pdo);

            if pdo.supply_kind == SupplyKind::FixedSupply {
                let pdo_fixed = PowerDataObjectFixed::unpack_from_slice(&data_bytes).unwrap();
                println!("pdo fixed: {:#?}", pdo_fixed);
                println!("Max current: {:.2} A", pdo_fixed.get_maximum_current_milli_amperes() as f32 / 1000.0);
                println!("Voltage: {:.2} V", pdo_fixed.get_voltage_milli_volts() as f32 / 1000.0);
            }
        }

        let packet_size = 2 + data.get_number_of_data_bytes();
        let for_crc = &traffic[..packet_size];
        println!("for crc: {:?}", for_crc);

        use crc::{crc32, Hasher32};

        let c = crc32::checksum_ieee(for_crc);
        //let mut digest = crc32::Digest::new(0xEDB88320);
        //digest.write(crc);
        //let c = digest.sum32();
        //assert_eq!(digest.sum32(), 0xcbf43926);

        println!("computed crc: 0X{:X}", c);
        assert_eq!(crc, c);
    }    

    
    

    //println!("traffic: {:?}", traffic);
}


