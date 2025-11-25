use crate::overpass::Point;
use anyhow::Result;
use rppal::uart::{Parity, Uart};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use log::info;
use tokio::time::sleep;

pub struct Gps {
    uart: Uart,
    buffer: Vec<u8>,
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

#[derive(Debug, Copy, Clone)]
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
    pub length: f64,   // Google Coordinate Distance
}

impl Vector {
    pub fn rotate(&self, rotation: f64) -> Vector {
        Vector {
            rotation: self.rotation + rotation,
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
            uart,
            buffer: Vec::new(),
        }
    }

    pub async fn init(&mut self) {
        // sleep(Duration::from_millis(250)).await;
        // self.send_command(Command::SetNmeaBaudrate115200)
        //     .await
        //     .unwrap();
        // sleep(Duration::from_millis(250)).await;
        //
        // self.set_baud_rate(115200).unwrap();
        // sleep(Duration::from_millis(100)).await;

        self.send_command(Command::SetPosFix100ms).await.unwrap();
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
        let mut attempt = 0;
        const MAX_ATTEMPTS: u32 = 5;

        loop {
            attempt += 1;

            let mut buff_t = vec![0u8; 800];
            let bytes_read = self.uart.read(&mut buff_t).unwrap_or(0);

            if bytes_read > 0 {
                self.buffer.extend_from_slice(&buff_t[..bytes_read]);
            }

            // println!("{}", String::from_utf8_lossy(&self.buffer));

            let mut pos = 0;
            while pos < self.buffer.len() {
                if self.buffer[pos] == b'$' {
                    let mut end = pos + 1;
                    while end < self.buffer.len()
                        && self.buffer[end] != b'\r'
                        && self.buffer[end] != b'\n'
                        && self.buffer[end] != b'$'
                    {
                        end += 1;
                    }

                    if end < self.buffer.len()
                        && (self.buffer[end] == b'\r' || self.buffer[end] == b'\n')
                    {
                        let sentence_bytes = &self.buffer[pos..end];
                        let sentence_str = String::from_utf8_lossy(sentence_bytes);

                        if sentence_bytes.len() > 6
                            && sentence_bytes[1] == b'G'
                            && (sentence_bytes[2] == b'N' || sentence_bytes[2] == b'P')
                            && sentence_bytes[3] == b'R'
                            && sentence_bytes[4] == b'M'
                            && sentence_bytes[5] == b'C'
                        {
                            let parts: Vec<&str> = sentence_str.split(',').collect();

                            let mut gps = GNRMC::default();

                            if parts.len() >= 3 {
                                if parts.len() > 1 && parts[1].len() >= 6 {
                                    let time_str = &parts[1][..6];
                                    if let Ok(time) = time_str.parse::<u32>() {
                                        gps.time_h = ((time / 10000 + 8) % 24) as u8;
                                        gps.time_m = ((time / 100) % 100) as u8;
                                        gps.time_s = (time % 100) as u8;
                                    }
                                }

                                let status_str = if parts.len() > 2 { parts[2].trim() } else { "" };
                                gps.status = if status_str == "A" { 1 } else { 0 };

                                if gps.status == 1 {
                                    if parts.len() > 3 && !parts[3].is_empty() {
                                        if let Ok(lat) = parts[3].parse::<f64>() {
                                            gps.lat = lat;
                                        }
                                    }

                                    if parts.len() > 4 && !parts[4].is_empty() {
                                        gps.lat_area = parts[4].as_bytes()[0];
                                    }

                                    if parts.len() > 5 && !parts[5].is_empty() {
                                        if let Ok(lon) = parts[5].parse::<f64>() {
                                            gps.lon = lon;
                                        }
                                    }

                                    if parts.len() > 6 && !parts[6].is_empty() {
                                        gps.lon_area = parts[6].as_bytes()[0];
                                    }
                                    info!("GPS FIX: lat={:.6}, lon={:.6}", gps.lat, gps.lon);
                                }
                            }

                            let clear_until = if end + 1 < self.buffer.len() {
                                end + 1
                            } else {
                                end
                            };
                            self.buffer.drain(0..clear_until);

                            return gps;
                        } else {
                            let clear_until = if end + 1 < self.buffer.len() {
                                end + 1
                            } else {
                                end
                            };
                            self.buffer.drain(0..clear_until);
                        }
                    } else {
                        break;
                    }
                } else {
                    pos += 1;
                }
            }

            if attempt >= MAX_ATTEMPTS {
                if self.buffer.len() > 2000 {
                    self.buffer.clear();
                }
                return GNRMC::default();
            }

            sleep(Duration::from_millis(10)).await;
        }
    }

    // Calculate bearing in radians to determine direction based off movement from 2 points
    pub fn calculate_bearing(from: &Point, to: &Point) -> f64 {
        let x = to.lon - from.lon;
        let y = to.lat - from.lat;
        let bearing = y.atan2(x);

        // println!("Bearing calc: from({:.6}, {:.6}) to ({:.6}, {:.6})", from.lat, from.lon, to.lat, to.lon);
        // println!("  dx={:.8}, dy={:.8}, bearing={:.4} rad ({:.1}°)", x, y, bearing, bearing.to_degrees());

        bearing
    }

    pub async fn get_with_direction(
        &mut self,
        previous_position: Option<Point>,
    ) -> (GNRMC, Option<f64>) {
        let current_reading = self.get().await;

        if current_reading.status == 1 {
            let current_position = current_reading.google_coordinates();

            if let Some(prev_pos) = previous_position {
                let direction = Self::calculate_bearing(&prev_pos, &current_position);
                return (current_reading, Some(direction));
            }
        }

        (current_reading, None)
    }
}

pub struct GpsSimulator {
    starting_point: Point,
    ending_point: Point,
    current_point: Point,
}

impl GpsSimulator {
    pub fn new(starting_point: Point, ending_point: Point) -> Self {
        Self {
            starting_point,
            ending_point,
            current_point: starting_point,
        }
    }

    pub fn get(&mut self) -> Option<Point> {
        let step_size = 0.00001;

        let mut next_lat = self.current_point.lat;
        let mut next_lon = self.current_point.lon;

        // println!("{} {}", (next_lat - self.ending_point.lat).abs(), (next_lon - self.ending_point.lon).abs());

        if (next_lat - self.ending_point.lat).abs() > step_size
            || (next_lon - self.ending_point.lon).abs() > step_size
        {
            if next_lat < self.ending_point.lat {
                next_lat += step_size;
            } else if next_lat > self.ending_point.lat {
                next_lat -= step_size;
            }

            if next_lon < self.ending_point.lon {
                next_lon += step_size;
            } else if next_lon > self.ending_point.lon {
                next_lon -= step_size;
            }

            self.current_point = Point {
                lat: next_lat,
                lon: next_lon,
            };

            Some(self.current_point)
        } else {
            None
        }
    }

    pub fn get_with_direction(
        &mut self,
        previous_position: Option<Point>,
    ) -> (Option<Point>, Option<f64>) {
        let current_position = self.get();

        if current_position.is_none() {
            return (None, None);
        }

        let current_pos = current_position.unwrap();

        if let Some(prev_pos) = previous_position {
            let direction = Gps::calculate_bearing(&prev_pos, &current_pos);
            return (Some(current_pos), Some(direction));
        }

        (Some(current_pos), None)
    }
}
