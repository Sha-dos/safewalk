use std::time::Duration;
use rppal::uart::{Parity, Uart};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use crate::overpass::Point;

pub struct Gps {
    uart: Uart,
}

pub enum Command {
    HotStart,
    WarmStart,
    ColdStart,
    FullColdStart,
    SetPerpetualStandbyMode,
    SetPeriodicMode,
    SetNormalMode,
    SetPeriodicBackupMode,
    SetPeriodicStandbyMode,
    SetPerpetualBackupMode,
    SetAlwaysLocateStandbyMode,
    SetAlwaysLocateBackupMode,
    SetPosFix,
    SetPosFix100ms,
    SetPosFix200ms,
    SetPosFix400ms,
    SetPosFix800ms,
    SetPosFix1s,
    SetPosFix2s,
    SetPosFix4s,
    SetPosFix8s,
    SetPosFix10s,
    SetSyncPpsNmeaOff,
    SetSyncPpsNmeaOn,
    SetNmeaBaudrate,
    SetNmeaBaudrate115200,
    SetNmeaBaudrate57600,
    SetNmeaBaudrate38400,
    SetNmeaBaudrate19200,
    SetNmeaBaudrate14400,
    SetNmeaBaudrate9600,
    SetNmeaBaudrate4800,
    SetReduction,
    SetNmeaOutput,
}

impl Command {
    pub fn as_str(&self) -> &'static str {
        match self {
            Command::HotStart => "$PMTK101",
            Command::WarmStart => "$PMTK102",
            Command::ColdStart => "$PMTK103",
            Command::FullColdStart => "$PMTK104",
            Command::SetPerpetualStandbyMode => "$PMTK161",
            Command::SetPeriodicMode => "$PMTK225",
            Command::SetNormalMode => "$PMTK225,0",
            Command::SetPeriodicBackupMode => "$PMTK225,1,1000,2000",
            Command::SetPeriodicStandbyMode => "$PMTK225,2,1000,2000",
            Command::SetPerpetualBackupMode => "$PMTK225,4",
            Command::SetAlwaysLocateStandbyMode => "$PMTK225,8",
            Command::SetAlwaysLocateBackupMode => "$PMTK225,9",
            Command::SetPosFix => "$PMTK220",
            Command::SetPosFix100ms => "$PMTK220,100",
            Command::SetPosFix200ms => "$PMTK220,200",
            Command::SetPosFix400ms => "$PMTK220,400",
            Command::SetPosFix800ms => "$PMTK220,800",
            Command::SetPosFix1s => "$PMTK220,1000",
            Command::SetPosFix2s => "$PMTK220,2000",
            Command::SetPosFix4s => "$PMTK220,4000",
            Command::SetPosFix8s => "$PMTK220,8000",
            Command::SetPosFix10s => "$PMTK220,10000",
            Command::SetSyncPpsNmeaOff => "$PMTK255,0",
            Command::SetSyncPpsNmeaOn => "$PMTK255,1",
            Command::SetNmeaBaudrate => "$PMTK251",
            Command::SetNmeaBaudrate115200 => "$PMTK251,115200",
            Command::SetNmeaBaudrate57600 => "$PMTK251,57600",
            Command::SetNmeaBaudrate38400 => "$PMTK251,38400",
            Command::SetNmeaBaudrate19200 => "$PMTK251,19200",
            Command::SetNmeaBaudrate14400 => "$PMTK251,14400",
            Command::SetNmeaBaudrate9600 => "$PMTK251,9600",
            Command::SetNmeaBaudrate4800 => "$PMTK251,4800",
            Command::SetReduction => "$PMTK314,-1",
            Command::SetNmeaOutput => "$PMTK314,1,1,1,1,1,1,1,1,0,0,0,0,0,0,0,0,0,1,0",
        }
    }
}

#[derive(Debug)]
pub struct GNRMC {
    pub lon: f64,
    pub lat: f64,
    pub lon_area: u8,
    pub lat_area: u8,
    pub time_h: u8,
    pub time_m: u8,
    pub time_s: u8,
    pub status: u8, // 1:Successful positioning 0：Positioning failed
}

impl Default for GNRMC {
    fn default() -> Self {
        GNRMC {
            lon: 0.0,
            lat: 0.0,
            lon_area: 0,
            lat_area: 0,
            time_h: 0,
            time_m: 0,
            time_s: 0,
            status: 0,
        }
    }
}

impl GNRMC {
    pub fn google_coordinates(&self) -> Point {
        let lat_raw = self.lat;
        let lon_raw = self.lon;
        let lat_deg = (lat_raw as i32 / 100) as f64;
        let lat_min = lat_raw - (lat_deg * 100.0);
        let mut lat = lat_deg + (lat_min / 60.0);
        if self.lat_area == b'S' {
            lat = -lat;
        }
        let lon_deg = (lon_raw as i32 / 100) as f64;
        let lon_min = lon_raw - (lon_deg * 100.0);
        let mut lon = lon_deg + (lon_min / 60.0);
        if self.lon_area == b'W' {
            lon = -lon;
        }

        Point { lat, lon }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Vector {
    pub rotation: f64, // Radians
    pub length: f64, // Google Coordinate Distance
}

impl Vector {
    pub fn rotate(&self, other: &Vector) -> Vector {
        Vector {
            rotation: self.rotation + other.rotation,
            length: self.length,
        }
    }
}

impl Vector {
    pub fn new(rotation: f64, length: f64) -> Self {
        Self { rotation, length }
    }
}

impl Gps {
    pub fn new() -> Self {
        let mut uart = Uart::with_path("/dev/ttyS0", 9600, Parity::None, 8, 1).unwrap();

        Self {
            uart
        }
    }

    pub async fn init(&mut self) {
        self.send_command(Command::SetNmeaBaudrate115200).await.unwrap();
        sleep(Duration::from_millis(100)).await;

        self.set_baud_rate(115200).unwrap();
        sleep(Duration::from_millis(100)).await;

        self.send_command(Command::SetPosFix400ms).await.unwrap();
        self.send_command(Command::SetNmeaOutput).await.unwrap();
    }

    pub fn set_baud_rate(&mut self, baud_rate: u32) -> Result<()> {
        self.uart.set_baud_rate(baud_rate)?;

        Ok(())
    }

    pub async fn send_command(&mut self, command: Command) -> Result<()> {
        let cmd_str = command.as_str();
        let mut checksum: u8 = 0;
        for byte in cmd_str.as_bytes().iter().skip(1) {
            checksum ^= *byte;
        }

        let full_command = format!("{}*{:02X}\r\n", cmd_str, checksum);
        self.uart.write(full_command.as_bytes())?;

        sleep(Duration::from_millis(200)).await;

        Ok(())
    }

    pub async fn get(&mut self) -> GNRMC {
        let mut buff_t = vec![0u8; 800];
        self.uart.read(&mut buff_t).unwrap();

        println!("{}", String::from_utf8_lossy(&buff_t));

        let mut add = 0;
        let mut gps = GNRMC::default();

        while add < 800 - 71 {
            if buff_t[add] == b'$'
                && buff_t[add + 1] == b'G'
                && (buff_t[add + 2] == b'N' || buff_t[add + 2] == b'P')
                && buff_t[add + 3] == b'R'
                && buff_t[add + 4] == b'M'
                && buff_t[add + 5] == b'C'
            {
                let mut x = 0;
                let mut z = 0;
                while x < 12 {
                    if buff_t[add + z] == b'\0' {
                        break;
                    }
                    if buff_t[add + z] == b',' {
                        x += 1;
                        match x {
                            1 => {
                                let mut time: u32 = 0;
                                let mut i = 0;
                                while buff_t[add + z + i + 1] != b'.' {
                                    if buff_t[add + z + i + 1] == b'\0' {
                                        break;
                                    }
                                    if buff_t[add + z + i + 1] == b',' {
                                        break;
                                    }
                                    time = (buff_t[add + z + i + 1] - b'0') as u32 + time * 10;
                                    i += 1;
                                }
                                gps.time_h = (time / 10000 + 8) as u8;
                                gps.time_m = ((time / 100) % 100) as u8;
                                gps.time_s = (time % 100) as u8;
                                if gps.time_h >= 24 {
                                    gps.time_h -= 24;
                                }
                            }
                            2 => {
                                if buff_t[add + z + 1] == b'A' {
                                    gps.status = 1;
                                } else {
                                    gps.status = 0;
                                }
                            }
                            3 => {
                                let mut latitude: u32 = 0;
                                let mut i = 0;
                                while buff_t[add + z + i + 1] != b',' {
                                    if buff_t[add + z + i + 1] == b'\0' {
                                        break;
                                    }
                                    if buff_t[add + z + i + 1] == b'.' {
                                        i += 1;
                                        continue;
                                    }
                                    latitude = (buff_t[add + z + i + 1] - b'0') as u32 + latitude * 10;
                                    i += 1;
                                }
                                gps.lat = latitude as f64 / 1_000_000.0;
                            }
                            4 => {
                                gps.lat_area = buff_t[add + z + 1];
                            }
                            5 => {
                                let mut longitude: u32 = 0;
                                let mut i = 0;
                                while buff_t[add + z + i + 1] != b',' {
                                    if buff_t[add + z + i + 1] == b'\0' {
                                        break;
                                    }
                                    if buff_t[add + z + i + 1] == b'.' {
                                        i += 1;
                                        continue;
                                    }
                                    longitude = (buff_t[add + z + i + 1] - b'0') as u32 + longitude * 10;
                                    i += 1;
                                }
                                gps.lon = longitude as f64 / 1_000_000.0;
                            }
                            6 => {
                                gps.lon_area = buff_t[add + z + 1];
                            }
                            _ => {}
                        }
                    }
                    z += 1;
                }
                add = 0;
                break;
            }

            if buff_t[add + 5] == b'\0' {
                add = 0;
                break;
            }

            add += 1;

            if add > 800 {
                add = 0;
                break;
            }
        }

        gps
    }

    pub fn calculate_bearing(&self, from: &Point, to: &Point) -> f64 {
        let lat1 = from.lat.to_radians();
        let lat2 = to.lat.to_radians();
        let delta_lon = (to.lon - from.lon).to_radians();

        let y = delta_lon.sin() * lat2.cos();
        let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * delta_lon.cos();

        let bearing = y.atan2(x);
        (bearing.to_degrees() + 360.0) % 360.0
    }
    
    pub async fn get_with_direction(&mut self, previous_position: Option<Point>) -> (GNRMC, Option<f64>) {
        let current_reading = self.get().await;

        if current_reading.status == 1 {
            let current_position = current_reading.google_coordinates();

            if let Some(prev_pos) = previous_position {
                let direction = self.calculate_bearing(&prev_pos, &current_position);
                return (current_reading, Some(direction));
            }
        }

        (current_reading, None)
    }
}
