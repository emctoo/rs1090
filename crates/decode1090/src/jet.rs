#![doc = include_str!("../readme.md")]

use clap::Parser;
use rs1090::decode::adsb::{ADSB, ME};
use rs1090::decode::cpr::{decode_position, AircraftState, Position};
use rs1090::decode::IdentityCode;
use rs1090::prelude::*;
use std::collections::BTreeMap;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Parser)]
#[command(
    name = "jet1090",
    version,
    author = "xoolive",
    about = "Decode Mode S demodulated raw messages"
)]
struct Options {
    /// Address of the demodulating server (beast feed)
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Port of the demodulating server
    #[arg(short, long, default_value = None)]
    port: Option<u16>,

    /// Demodulate data from a RTL-SDR dongle
    #[arg(long, default_value = "false")]
    rtlsdr: bool,

    /// Activate JSON output
    #[arg(short, long, default_value = "false")]
    verbose: bool,

    /// Dump a copy of the received messages as .jsonl
    #[arg(short, long, default_value=None)]
    output: Option<String>,

    /// Reference coordinates for the decoding (e.g.
    //  --latlon LFPG for major airports,
    /// --latlon 43.3,1.35 or --latlon ' -34,18.6' if negative)
    #[arg(long, default_value=None)]
    latlon: Option<Position>,

    /// Display a table in interactive mode (not compatible with verbose)
    #[arg(short, long, default_value = "false")]
    interactive: bool,

    /// How to serve the collected data (todo!())
    #[arg(long, default_value=None)]
    serve: Option<u8>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::parse();

    let mut file = if let Some(output_path) = options.output {
        Some(
            fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(output_path)
                .await?,
        )
    } else {
        None
    };

    let mut reference = options.latlon;
    let mut aircraft: BTreeMap<ICAO, AircraftState> = BTreeMap::new();
    let mut states: BTreeMap<String, StateVectors> = BTreeMap::new();

    let mut rx = if options.rtlsdr {
        #[cfg(not(feature = "rtlsdr"))]
        {
            eprintln!(
                "Not compiled with RTL-SDR support, use the rtlsdr feature"
            );
            std::process::exit(127);
        }
        #[cfg(feature = "rtlsdr")]
        {
            rtlsdr::discover();
            rtlsdr::receiver().await
        }
    } else {
        let server_address =
            format!("{}:{}", options.host, options.port.unwrap());
        radarcape::receiver(server_address).await
    };

    while let Some(tmsg) = rx.recv().await {
        let frame = hex::decode(&tmsg.frame).unwrap();
        if let Ok((_, msg)) = Message::from_bytes((&frame, 0)) {
            let mut msg = TimedMessage {
                timestamp: tmsg.timestamp,
                frame: tmsg.frame.to_string(),
                message: Some(msg),
            };

            if let Some(message) = &mut msg.message {
                match &mut message.df {
                    ExtendedSquitterADSB(adsb) => decode_position(
                        &mut adsb.message,
                        msg.timestamp,
                        &adsb.icao24,
                        &mut aircraft,
                        &mut reference,
                    ),
                    ExtendedSquitterTisB { cf, .. } => decode_position(
                        &mut cf.me,
                        msg.timestamp,
                        &cf.aa,
                        &mut aircraft,
                        &mut reference,
                    ),
                    _ => {}
                }
            };

            update_snapshot(&mut states, &mut msg);
            //dbg!(&states);

            let json = serde_json::to_string(&msg).unwrap();
            if options.verbose {
                println!("{}", json);
            }
            if let Some(file) = &mut file {
                file.write_all(json.as_bytes()).await?;
                file.write_all("\n".as_bytes()).await?;
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
pub struct StateVectors {
    pub cur: Snapshot,
    //pub hist: Vec<TimedMessage>,
}

impl StateVectors {
    fn new(ts: u32) -> StateVectors {
        let cur = Snapshot {
            first: ts,
            last: ts,
            callsign: None,
            squawk: None,
            latitude: None,
            longitude: None,
            altitude: None,
            groundspeed: None,
            vertical_rate: None,
            track: None,
            ias: None,
            mach: None,
            roll: None,
        };
        StateVectors { cur }
    }
}

#[derive(Debug)]
pub struct Snapshot {
    pub first: u32,
    pub last: u32,
    pub callsign: Option<String>,
    pub squawk: Option<IdentityCode>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<u16>,
    pub groundspeed: Option<f64>,
    pub vertical_rate: Option<i16>,
    pub track: Option<f64>,
    pub ias: Option<u16>,
    pub mach: Option<f64>,
    pub roll: Option<f64>,
}

fn icao24(msg: &Message) -> Option<String> {
    match msg.df {
        ShortAirAirSurveillance { ap, .. } => Some(ap.to_string()),
        SurveillanceAltitudeReply { ap, .. } => Some(ap.to_string()),
        SurveillanceIdentityReply { ap, .. } => Some(ap.to_string()),
        AllCallReply { icao, .. } => Some(icao.to_string()),
        LongAirAirSurveillance { ap, .. } => Some(ap.to_string()),
        ExtendedSquitterADSB(ADSB { icao24, .. }) => Some(icao24.to_string()),
        ExtendedSquitterTisB { pi, .. } => Some(pi.to_string()),
        CommBAltitudeReply { ap, .. } => Some(ap.to_string()),
        CommBIdentityReply { ap, .. } => Some(ap.to_string()),
        _ => None,
    }
}

fn update_snapshot(
    states: &mut BTreeMap<String, StateVectors>,
    msg: &mut TimedMessage,
) {
    if let TimedMessage {
        timestamp,
        message: Some(message),
        ..
    } = msg
    {
        if let Some(icao24) = icao24(message) {
            let aircraft = states
                .entry(icao24)
                .or_insert(StateVectors::new(*timestamp as u32));
            aircraft.cur.last = *timestamp as u32;

            match &mut message.df {
                SurveillanceIdentityReply { id, .. } => {
                    aircraft.cur.squawk = Some(*id)
                }
                SurveillanceAltitudeReply { ac, .. } => {
                    aircraft.cur.altitude = Some(ac.0);
                }
                ExtendedSquitterADSB(adsb) => {
                    match &adsb.message {
                        ME::BDS05(bds05) => {
                            aircraft.cur.latitude = bds05.latitude;
                            aircraft.cur.longitude = bds05.longitude;
                            aircraft.cur.altitude = bds05.alt;
                        }
                        ME::BDS06(bds06) => {
                            aircraft.cur.latitude = bds06.latitude;
                            aircraft.cur.longitude = bds06.longitude;
                            aircraft.cur.track = bds06.track;
                            aircraft.cur.groundspeed = bds06.groundspeed;
                        }
                        ME::BDS08(bds08) => {
                            aircraft.cur.callsign =
                                Some(bds08.callsign.to_string())
                        }
                        ME::BDS09(bds09) => {
                            aircraft.cur.vertical_rate = bds09.vertical_rate;
                            //aircraft.cur.groundspeed = bds09.s
                        }
                        _ => {}
                    }
                }
                ExtendedSquitterTisB { cf, .. } => match &cf.me {
                    ME::BDS05(bds05) => {
                        aircraft.cur.latitude = bds05.latitude;
                        aircraft.cur.longitude = bds05.longitude;
                        aircraft.cur.altitude = bds05.alt;
                    }
                    ME::BDS06(bds06) => {
                        aircraft.cur.latitude = bds06.latitude;
                        aircraft.cur.longitude = bds06.longitude;
                        aircraft.cur.track = bds06.track;
                        aircraft.cur.groundspeed = bds06.groundspeed;
                    }
                    ME::BDS08(bds08) => {
                        aircraft.cur.callsign = Some(bds08.callsign.to_string())
                    }
                    _ => {}
                },
                CommBAltitudeReply { bds, .. } => {
                    // Invalidate data if marked as both BDS50 and BDS60
                    if let (Some(_), Some(_)) = (&bds.bds50, &bds.bds60) {
                        bds.bds50 = None;
                        bds.bds60 = None
                    }
                }
                CommBIdentityReply { bds, .. } => {
                    // Invalidate data if marked as both BDS50 and BDS60
                    if let (Some(_), Some(_)) = (&bds.bds50, &bds.bds60) {
                        bds.bds50 = None;
                        bds.bds60 = None
                    }
                    if let Some(bds50) = &bds.bds50 {
                        aircraft.cur.roll = bds50.roll_angle;
                        aircraft.cur.track = bds50.track_angle;
                        aircraft.cur.groundspeed =
                            bds50.groundspeed.map(|x| x as f64);
                    }
                    if let Some(bds60) = &bds.bds60 {
                        aircraft.cur.ias = bds60.indicated_airspeed;
                        aircraft.cur.mach = bds60.mach_number;
                    }
                }

                _ => {}
            };
        }
    }
}