//#![allow(unused_imports, dead_code)]

use clap::Parser;
use kcs_decoder::*;
use riff_wave::WaveReader;
use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Write;
use std::ops::Deref;
use std::thread;
use std::time;

const MINIMUM_OUTPUT_FILE_SIZE: usize = 10;

fn decode_file(
    input_filename: &str,
    prefix: &str,
    config: &DecoderConfig,
    channel: u8,
    zc_direction: ZeroCrossingDirection,
) -> Result<usize, Box<dyn Error>> {
    let wavereader =
        riff_wave::WaveReader::new(BufReader::new(File::open(input_filename)?)).unwrap();
    let mut zc_detector = ZeroCrossingDetector::new(0.0);
    let mut frq_calculator =
        FrequencyIdentifier::new(zc_direction, wavereader.pcm_format.sample_rate);
    let mut hi_low_identifier = HiLowIdentifier::new(
        config.symbols[0].frequency as u32,
        config.symbols[1].frequency as u32,
        config.frequency_tolerance as u8,
        (config.symbols[1].periods as u8, config.symbols[1].signal),
        (config.symbols[0].periods as u8, config.symbols[0].signal),
    )
    .unwrap();
    let mut decoder = Decoder::new(*config).ok_or(DecoderError::Config)?;

    let mut output_prev_idx: usize = 0;
    let mut output_data: Vec<u8> = Vec::with_capacity(100000);

    let mut files_written: usize = 0;
    let samplerate = wavereader.pcm_format.sample_rate as usize;
    let mut write_vector_to_disk = |idx: usize, data: &mut Vec<u8>| -> Result<(), std::io::Error> {
        if data.len() >= MINIMUM_OUTPUT_FILE_SIZE {
            let filename = format!(
                "{prefix}-ch{channel}-{}-{}.dat",
                numsamples_to_timestring(output_prev_idx, samplerate),
                if zc_direction == ZeroCrossingDirection::Neg {
                    "neg"
                } else {
                    "pos"
                }
            );
            println!("Writing file '{filename}'");
            let mut file = File::create(filename)?;
            file.write_all(data)?;
            files_written += 1;
        }
        data.clear();
        output_prev_idx = idx;
        Ok(())
    };

    WaveReaderIteratorMono::new(wavereader, channel)
        .unwrap()
        .enumerate()
        .filter_map(|val| (zc_detector.process(val)))
        .filter_map(|val| frq_calculator.process(val))
        .filter_map(|val| hi_low_identifier.process(val))
        .chain([(0, SignalCondition::Mark)]) // To make sure we clock out the last data byte
        .map(|(idx, val)| (idx, decoder.process(val)))
        .filter(|(_idx, val)| {
            val.is_ok() || (val.is_err() && val.as_ref().err().unwrap().is_some())
        })
        .for_each(|(idx, val)| {
            match val {
                Err(Some(DecoderError::Parity)) => {
                    eprintln!(
                        "Channel {}: Parity error at {}",
                        channel,
                        numsamples_to_timestring(idx, samplerate)
                    );
                    write_vector_to_disk(idx, &mut output_data).unwrap();
                }
                Err(Some(DecoderError::Signal)) => {
                    //eprintln!("Signal error at sample {idx}");
                    write_vector_to_disk(idx, &mut output_data).unwrap();
                }
                Err(Some(DecoderError::Sync)) => {
                    eprintln!(
                        "Channel {}: Sync error at {}",
                        channel,
                        numsamples_to_timestring(idx, samplerate)
                    );
                    write_vector_to_disk(idx, &mut output_data).unwrap();
                }
                Err(Some(DecoderError::IO(val))) => {
                    eprintln!(
                        "Channel {}: IO error '{val}' at {}",
                        channel,
                        numsamples_to_timestring(idx, samplerate)
                    );
                }
                Err(Some(DecoderError::Other(val))) => {
                    eprintln!(
                        "Channel {}: Error '{val}' at {}",
                        channel,
                        numsamples_to_timestring(idx, samplerate)
                    );
                }
                Ok(val) => {
                    output_data.push(val);
                }
                _ => {}
            };
        });
    write_vector_to_disk(0, &mut output_data)?;
    Ok(files_written)
}

#[derive(Debug)]
struct BoxError<T: Error> {
    pub inner: Box<T>,
}

#[allow(dead_code)]
impl<T: Error> BoxError<T> {
    fn new(val: T) -> Self {
        Self::from(val)
    }
}

impl<T: Error> From<T> for BoxError<T> {
    fn from(value: T) -> Self {
        Self {
            inner: Box::new(value),
        }
    }
}

impl<T: Error + 'static> Error for BoxError<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.inner)
    }
}

impl<T: Error> Display for BoxError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BoxError::display")
    }
}

impl<T: Error> Deref for BoxError<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.inner
    }
}

fn numsamples_to_timestring(samples: usize, samplerate: usize) -> String {
    let seconds = samples / samplerate;
    format!("{:0>2}m{:0>2}s", seconds / 60, seconds % 60)
}

fn parse_command_line_arguments(
    args: Args,
) -> Result<(DecoderConfig, String, String), Box<dyn Error>> {
    let mut config = DecoderConfig::get_preset(&args.preset);

    config.parity = args.parity.unwrap_or(config.parity);
    config.num_databits = usize::from(args.num_databits.unwrap_or(config.num_databits as u8));
    config.startbits = (
        usize::from(args.num_startbits.unwrap_or(config.startbits.0 as u8)),
        args.startbit.unwrap_or(config.startbits.1),
    );
    config.stopbits = (
        usize::from(args.num_stopbits.unwrap_or(config.stopbits.0 as u8)),
        args.stopbit.unwrap_or(config.stopbits.1),
    );
    config.channels = args.channel;
    if args.baud_rate.is_some() {
        config.symbols = [
            Symbol {
                frequency: args.baud_rate.unwrap() as usize,
                periods: 1,
                signal: SignalCondition::Space,
            },
            Symbol {
                frequency: 2 * args.baud_rate.unwrap() as usize,
                periods: 2,
                signal: SignalCondition::Mark,
            },
        ];
    }

    if !args.inputfile.contains(".wav") {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            "Input file must be .wav",
        )));
    }
    Ok((
        config,
        args.inputfile.clone(),
        args.prefix
            .unwrap_or(args.inputfile[0usize..args.inputfile.find(".wav").unwrap()].to_string()),
    ))
}

#[derive(Parser, Debug)]
#[command(author = "Martin Sørensen", version, long_about)]
/// A decoder for the Kansas City Standard 'KCS' tape format.
///
/// This program will take a .wav file as the input and generate raw binary files as the input file is being decoded.
/// When an error is found in the stream, the state machine is reset and a new file will be started so all generated files can be assumed to be without detectable errors.
///
/// The NASCOM preset is the only one that has been tested so far.
struct Args {
    /// Input .wav file (PCM format only)
    inputfile: String,

    /// Optional output file prefix, default will use the name from the input file
    #[arg(long)]
    prefix: Option<String>,

    /// Channel to process if inputfile is multi-channel.
    /// For a stereo track, left will be 0 and right will be 1. 'All' will process all channels. (All|0|1|..)
    #[arg(short, long, default_value_t = Channels::All)]
    channel: Channels,

    /// Base config. Use the options below to adjust the preset. (Standard|NASCOM|Acorn|MSX1200|MSX2400)
    #[arg(short, long, default_value_t = Preset::Std)]
    preset: Preset,

    /// Baud rate
    #[arg(long)]
    baud_rate: Option<u16>,

    /// Number of start bits (1|2)
    #[arg(long)]
    num_startbits: Option<u8>,

    /// Number of data bits (7|8)
    #[arg(long)]
    num_databits: Option<u8>,

    /// Number of stop bits (1|2)
    #[arg(long)]
    num_stopbits: Option<u8>,

    /// Parity (None|Even|Odd|Space|Mark)
    #[arg(long)]
    parity: Option<Parity>,

    /// Start bit (Mark|Space)
    #[arg(long)]
    startbit: Option<SignalCondition>,

    /// Stop bit (Mark|Space)
    #[arg(long)]
    stopbit: Option<SignalCondition>,
}

fn main() -> Result<(), Box<dyn Error>> {
    //let myboxerror2: Result<DecoderState, _> = Err(DecoderError::Parity);
    //let _fg = myboxerror2?;

    let start = time::Instant::now();

    let config = parse_command_line_arguments(Args::parse()).expect("Parsing config");
    let pcm_format = WaveReader::new(File::open(config.1.clone())?)
        .unwrap()
        .pcm_format;
    let channelbounds = match config.0.channels {
        Channels::All => 0..pcm_format.num_channels as u8,
        Channels::Specific(ch) => ch..ch + 1,
    };

    println!(
        "Processing '{}', using output file prefix '{}'.\nActive decoder config:\n{}\n",
        config.1, config.2, config.0
    );
    let mut threadpool = vec![];
    for i in channelbounds {
        let (config1, filename, prefix) = config.clone();
        threadpool.push(thread::spawn(move || -> Result<usize, io::Error> {
            decode_file(&filename, &prefix, &config1, i, ZeroCrossingDirection::Neg).or(Err(
                io::Error::new(io::ErrorKind::Other, "Error reported during decoding"),
            ))
        }));
        let (config2, filename, prefix) = config.clone();
        threadpool.push(thread::spawn(move || -> Result<usize, io::Error> {
            decode_file(&filename, &prefix, &config2, i, ZeroCrossingDirection::Pos).or(Err(
                io::Error::new(io::ErrorKind::Other, "Error reported during decoding"),
            ))
        }));
    }
    let mut files_written: usize = 0;
    for handle in threadpool {
        files_written += handle.join().unwrap().unwrap_or(0);
    }
    println!(
        "Completed in {:.2} seconds, {files_written} files produced.",
        start.elapsed().as_secs_f32()
    );
    Ok(())
}
