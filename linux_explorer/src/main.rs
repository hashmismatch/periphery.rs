extern crate terminal_termion;
extern crate periphery_flex;
extern crate periphery_linux;
extern crate csv;
extern crate chrono;

use std::time::*;
use std::path::Path;
use std::thread;
use std::sync::{Arc, Mutex};

use chrono::*;

use periphery_flex::core::*;
use periphery_flex::core::bus::logger::*;
use periphery_flex::core::prelude::v1::*;
use periphery_flex::core::terminal_cli::*;
use periphery_flex::core::cli::*;
use periphery_flex::*;
use periphery_linux::*;

fn detect(i2c_busses: &Vec<(PeripheryBusCliState, Logger<LinuxI2CBus<StdSystemApi>>)>, devices: &mut Vec<Box<Device + Send + Sync + 'static>>) {
	devices.clear();
	
	for bus in i2c_busses {
		devices.extend(devices_detect_all(bus.1.clone()));
	}

	for device in devices.iter() {
		println!("Detected device: {}", device.description());
		match device.init_after_detection() {
			Ok(false) => (),
			Ok(true) => {
				println!("Defaults initialized.");
			},
			Err(e) => {
				println!("Failed to initialize to defaults: {:?}", e);
			},
		}
	}

	println!("Detected {} devices.", devices.len());
}

fn main() {
    use terminal_termion::*;	

	let options = PromptBufferOptions { echo: true, ..Default::default() };

    println!("Periphery.rs Linux device explorer");
		
    let i2c_busses = LinuxI2CBus::detect(StdSystemApi);
	let mut i2c_busses: Vec<_> = i2c_busses.into_iter()
		.map(|bus| Logger::new(bus) )
		.map(|bus| (PeripheryBusCliState::new(&bus).unwrap(), bus))
		.collect();
    println!("Detected {} I2C busses", i2c_busses.len());

	let mut devices = vec![];

	let mut term = TerminalTermion::new();	

	detect(&i2c_busses, &mut devices);
    
	let mut prompt = PromptBuffer::new(options);
	prompt.print_prompt(&mut term);

	#[derive(Copy, Clone, Debug, PartialEq)]
	enum PollingOutput {
		Screen,
		Csv
	}

	// data polling thread
	struct CurrentPolling {
		device: Box<Device + Send + Sync>,
		poller: Box<DataStreamPoller + Send + Sync>,
		output: PollingOutput

	}
	let current_polling: Option<CurrentPolling> = None;
	let mut current_polling = Arc::new(Mutex::new(current_polling));
	let mut polling_thread = {

		let mut current_polling = current_polling.clone();		
		thread::spawn(move|| {
			let mut new_poll = true;
			let mut csv_output = None;
			let mut n = 1;
			let mut started = Instant::now();

			loop {
				let mut sleep_for_ms: u32 = 100;

				if let Ok(mut current_polling) = current_polling.lock() {
					if let Some(ref mut current_polling) = *current_polling {
						let device = &current_polling.device;
						let mut poller = &mut current_polling.poller;

						if new_poll {
							n = 1;
							started = Instant::now();
							let local: DateTime<Local> = Local::now();

							// do something, like open a file
							if current_polling.output == PollingOutput::Csv {
								let file_path = format!("{}_{}.csv", device.id(), local.format("%F %X"));
								let mut wtr = csv::Writer::from_path(&file_path).unwrap();
								let mut header = vec!["num".to_string(), "ms".to_string()];
								for l in &poller.get_info().labels {
									header.push(l.to_string());
								}
								wtr.write_record(&header);
								csv_output = Some(wtr);
								println!("Writing CSV to {}", file_path);
							}
						}
						new_poll = false;
					
						if let Ok(polled) = poller.poll() {
							
							match current_polling.output {
								PollingOutput::Screen => {
									for result in &polled {
										println!("{:?}", result);
										n += 1;
									}
								},
								PollingOutput::Csv => {
									if let Some(ref mut wtr) = csv_output {
										let elapsed = started.elapsed();
										let nanos = elapsed.subsec_nanos() as u64;
										let ms = (1000*1000*1000 * elapsed.as_secs() + nanos)/(1000 * 1000);

										for result in &polled {
											let mut row = vec![n.to_string(), ms.to_string()];
											match result {
												&DataStreamPolled::F32 { ref data } => {
													for v in data {
														row.push(v.to_string());
													}
												},
												&DataStreamPolled::Strings { ref data } => {
													for v in data {
														row.push(v.to_string());
													}
												}
											}
											
											wtr.write_record(&row);
											n += 1;
										}
									}
								}
							}
						}

						sleep_for_ms = poller.get_info().poll_every_ms as u32;
					} else {
						new_poll = true;

						// cleanup						
						if let Some(csv_output) = csv_output.take() {
							// anything?
							println!("Captured {} data points.", n);
						}
					}
				}

				thread::sleep_ms(sleep_for_ms);
			}
		})
	};

	loop {
		match term.read() {
			Ok(key) => {

                let mut exit = false;
				let mut stop_polling = false;

                let prompt_result = prompt.handle_key(key, &mut term, |mut m| {


                    if let Some(mut ctx) = m.command("exit") {
						exit = true;
					}

					if let Some(mut ctx) = m.command("detect") {
						detect(&i2c_busses, &mut devices);
					}

					for bus in &mut i2c_busses {
						periphery_bus_cli(&mut bus.0, &bus.1, m);
						bus.1.logger_cli(m);
					}

					for device in &devices {
						device.execute_cli(m);
					}

					// data streams
					{
						if let Ok(c) = current_polling.lock() {
							if c.is_some() {
								let cmd = format!("data_stream/stop");
								if let Some(mut ctx) = m.command(&cmd) {
									stop_polling = true;
								}
							}
						}


						let mut device_start_polling = None;

						for (i, device) in devices.iter().enumerate() {
							if let Some(data_streams) = device.get_data_streams() {
								for data_stream in data_streams.get_stream_infos() {
									let cmd = format!("data_stream/{}/{}/info", device.id(), data_stream.cli_id);
									if let Some(mut ctx) = m.command(&cmd) {
										write!(ctx.get_terminal(), "{:?}\r\n", data_stream);
									}

									let cmd = format!("data_stream/{}/{}/poll", device.id(), data_stream.cli_id);
									if let Some(mut ctx) = m.command(&cmd) {
										if let Ok(poller) = data_streams.get_poller(data_stream.id) {
											device_start_polling = Some((i, poller, PollingOutput::Screen));
										}
									}

									let cmd = format!("data_stream/{}/{}/csv", device.id(), data_stream.cli_id);
									if let Some(mut ctx) = m.command(&cmd) {
										if let Ok(poller) = data_streams.get_poller(data_stream.id) {
											device_start_polling = Some((i, poller, PollingOutput::Csv));
										}
									}
								}
							}
						}

						if let Some(device_start_polling) = device_start_polling {
							if let Ok(mut current_polling) = current_polling.lock() {
								if current_polling.is_none() {
									let device = devices.swap_remove(device_start_polling.0);
									let poller = device_start_polling.1;
									let info = poller.get_info();

									write!(m.get_terminal(), "Starting polling device {}, data stream {}, polling every {} ms, data labels {:?}!\r\n",
										device.id(),
										info.cli_id,
										info.poll_every_ms,
										info.labels
									);
									
									*current_polling = Some(CurrentPolling {
										device: device,
										poller: poller,
										output: device_start_polling.2
									});
								}
							}
						}

					}

				});
				
                if exit { break; }

				match prompt_result {
					Some(PromptEvent::Break) => {
						if let Ok(c) = current_polling.lock() {
							if c.is_some() {
								stop_polling = true;
							} else {
								break;
							}
						}
					},
					_ => ()
				}

				if stop_polling {
					if let Ok(mut current_polling) = current_polling.lock() {
						if let Some(current_polling) = current_polling.take() {
							println!("Stopped polling device {}!", current_polling.device.id());
							devices.push(current_polling.device);
							continue;
						}
					}
				}



			},
			Err(_) => {
				break;
			}
		}
	}

    println!("");
}
