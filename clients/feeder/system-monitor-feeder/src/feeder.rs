extern crate reqwest;
extern crate serde;
extern crate systemstat;

use reqwest::StatusCode;
use std::thread;
use std::time::Duration;

use systemstat::{System, Platform};

pub fn start_feeder(vsshost:  &String, token: &String) {
    info!("Try authorizing to {} ", vsshost);
    let client = reqwest::blocking::Client::new();
    let request_url = format!(
        "{server}/vss/api/v1/authorize?token={jwttoken}",
        server = vsshost,
        jwttoken = token
    );

    //This is setup. Still fine to crash
    let response =  client.post(&request_url).body("").send().unwrap().text().unwrap();
    
    //reqwest::blocking::post(&request_url).unwrap().text().unwrap();
    debug!("Authorize: respsonse {}", response);

    let v: serde_json::Value = serde_json::from_str(&response).unwrap();

    debug!("Respond-id: {}",  v["requestId"].as_str().unwrap_or("None"));

    let sys = System::new();
    let h2=vsshost.clone();
    let feeder_thread = thread::spawn(move || feeder_loop(&client, h2, &sys) );
    feeder_thread.join().unwrap();
}


pub fn push_value(target_path: &str, value: &String, client: &reqwest::blocking::Client, vsshost: &String ) {
    let request_url = format!(
        "{server}/vss/api/v1/signals/{vsspath}?value={val}",
        val = value,
        server = vsshost,
        vsspath = target_path
    );
    let response =  match client.put(&request_url).body("").send() {
        Ok(resp) => resp,
        Err(x) => {
            warn!("Error setting {}: {}", target_path, x);
            return;
        }
    };

    if  response.status() != StatusCode::OK {
        warn!("Error processing request for {}: {} ", target_path, response.text().unwrap_or("No response".to_string()));
        return;
    }
}

pub fn feeder_loop(client : &reqwest::blocking::Client, vsshost:  String, sys : &System) {
    loop {
        thread::sleep(Duration::from_secs(5));

        debug!("Thread alive");

        //Uptime
        let uptime=match sys.uptime() {
            Ok(uptime) => uptime.as_secs(),
            Err(_x) => 0
        };
        push_value("Vehicle.Private.Systemstats.Uptime", &uptime.to_string(), client, &vsshost);

        //CPU load
        let cpuload = match sys.cpu_load_aggregate() {
            Ok(cpu)=> {
                println!("\nMeasuring CPU load...");
                thread::sleep(Duration::from_millis(100));
                let l=match cpu.done() {
                    Ok(cpu) => (cpu.user+cpu.nice+cpu.system+cpu.interrupt)*100.0,
                    Err(x) =>  {
                        warn!("Error measuring CPU load: {}", x);
                        999.0
                    }
                };
                l
            }
            Err(x) => {
                warn!("Error getting CPU load: {}",x);
                999.0
            }
        };
        push_value("Vehicle.Private.Systemstats.Cpuload", &cpuload.to_string(), client, &vsshost);
        
 /*   
        match sys.networks() {
            Ok(netifs) => {
                println!("\nNetworks:");
                for netif in netifs.values() {
                    println!("{} ({:?})", netif.name, netif.addrs);
                }
            }
            Err(x) => println!("\nNetworks: error: {}", x)
        }
    
        match sys.networks() {
            Ok(netifs) => {
                println!("\nNetwork interface statistics:");
                for netif in netifs.values() {
                    println!("{} statistics: ({:?})", netif.name, sys.network_stats(&netif.name));
                }
            }
            Err(x) => println!("\nNetworks: error: {}", x)
        }
    
    
        match sys.memory() {
            Ok(mem) => println!("\nMemory: {} used / {} ({} bytes) total ({:?})", saturating_sub_bytes(mem.total, mem.free), mem.total, mem.total.as_u64(), mem.platform_memory),
            Err(x) => println!("\nMemory: error: {}", x)
        }
    
        match sys.load_average() {
            Ok(loadavg) => println!("\nLoad average: {} {} {}", loadavg.one, loadavg.five, loadavg.fifteen),
            Err(x) => println!("\nLoad average: error: {}", x)
        }
    
        match sys.uptime() {
            Ok(uptime) => println!("\nUptime: {:?}", uptime),
            Err(x) => println!("\nUptime: error: {}", x)
        }
    
    
        match sys.cpu_load_aggregate() {
            Ok(cpu)=> {
                println!("\nMeasuring CPU load...");
                thread::sleep(Duration::from_secs(1));
                let cpu = cpu.done().unwrap();
                println!("CPU load: {}% user, {}% nice, {}% system, {}% intr, {}% idle ",
                    cpu.user * 100.0, cpu.nice * 100.0, cpu.system * 100.0, cpu.interrupt * 100.0, cpu.idle * 100.0);
            },
            Err(x) => println!("\nCPU load: error: {}", x)
        }
    
        match sys.cpu_temp() {
            Ok(cpu_temp) => println!("\nCPU temp: {}", cpu_temp),
            Err(x) => println!("\nCPU temp: {}", x)
        }
    
        */
    }
}