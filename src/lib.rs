//#![allow(unused_imports, dead_code)]

use core::fmt::Debug;
use core::ops::Range;
use riff_wave::WaveReader;
use std::error::Error;
use std::fmt::Display;
use std::io::{Read, Seek};

const MAX_NUM_STARTBITS: usize = 2;
const MIN_NUM_STARTBITS: usize = 1;
const MAX_NUM_STOPBITS: usize = 2;
const MIN_NUM_STOPBITS: usize = 1;
const MAX_NUM_DATABITS: usize = 8;
const MIN_NUM_DATABITS: usize = 7;

pub struct WaveReaderIteratorMono<T: Read + Seek> {
    reader: riff_wave::WaveReader<T>,
}

pub struct WaveReaderIteratorStereo<T: Read + Seek> {
    reader: riff_wave::WaveReader<T>,
}

fn read_sample(reader: &mut WaveReader<impl Read + Seek>) -> Result<f32, impl Error> {
    match reader.pcm_format.bits_per_sample {
        8 => reader
            .read_sample_u8()
            .and_then(|val| Ok((val as i16 - i16::pow(2, 7)) as f32))
            .or_else(|_| Err(DecoderError::Signal)),
        16 => reader
            .read_sample_i16()
            .and_then(|val| Ok(val as f32))
            .or_else(|_| Err(DecoderError::Signal)),
        24 => reader
            .read_sample_i24()
            .and_then(|val| Ok(val as f32))
            .or_else(|_| Err(DecoderError::Signal)),
        32 => reader
            .read_sample_i32()
            .and_then(|val| Ok(val as f32))
            .or_else(|_| Err(DecoderError::Signal)),
        _ => Err(DecoderError::Signal),
    }
}

impl<T: Read + Seek> WaveReaderIteratorMono<T> {
    pub fn new(mut reader: WaveReader<T>, first_channel_idx: u8) -> Result<Self, impl Error> {
        if first_channel_idx as u16 >= reader.pcm_format.num_channels {
            Err(DecoderError::Signal)
        } else {
            for _ in 0..first_channel_idx {
                if read_sample(&mut reader).is_err() {
                    return Err(DecoderError::Signal);
                }
            }
            Ok(Self { reader })
        }
    }

    fn read_sample_mono_f32(&mut self) -> Result<<Self as IntoIterator>::Item, impl Error> {
        match read_sample(&mut self.reader) {
            Ok(val) => Ok(val as f32),
            Err(e) => Err(e),
        }
        .and_then(|val| {
            for _ in 1..self.reader.pcm_format.num_channels {
                read_sample(&mut self.reader)?;
            }
            Ok(val)
        })
    }
}

impl<T: Read + Seek> WaveReaderIteratorStereo<T> {
    pub fn new(reader: WaveReader<T>) -> Result<Self, impl Error> {
        match reader.pcm_format.num_channels {
            2 => Ok(Self { reader }),
            _ => Err(DecoderError::Signal),
        }
    }

    fn read_sample_stereo_f32(&mut self) -> Result<<Self as IntoIterator>::Item, impl Error> {
        match (read_sample(&mut self.reader), read_sample(&mut self.reader)) {
            (Ok(lval), Ok(rval)) => Ok([lval as f32, rval as f32]),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        }
    }
}

impl<T: Read + Seek> Iterator for WaveReaderIteratorMono<T> {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.read_sample_mono_f32().ok()?;
        let scale = u32::pow(2, (self.reader.pcm_format.bits_per_sample - 1) as u32) as Self::Item;
        Some(sample / scale)
    }
}

impl<T: Read + Seek> Iterator for WaveReaderIteratorStereo<T> {
    type Item = [f32; 2];
    fn next(&mut self) -> Option<Self::Item> {
        let samples = self.read_sample_stereo_f32().ok()?;
        let scale = u32::pow(2, (self.reader.pcm_format.bits_per_sample - 1) as u32) as f32;
        Some(samples.map(|val| (val / scale) as f32))
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Eq)]
pub enum Channels {
    Specific(u8),
    All,
}

impl Display for Channels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parity: String = match self {
            Channels::Specific(ch) => u8::to_string(ch),
            Channels::All => "All".to_string(),
        };
        write!(f, "{}", parity)
    }
}

impl From<&str> for Channels {
    fn from(value: &str) -> Self {
        match value.to_uppercase().chars().nth(0).or(Some('A')).unwrap() {
            'A' => Channels::All,
            val => match val.to_digit(10) {
                Some(x) if x < 256 => Channels::Specific(x as u8),
                _ => Channels::All,
            },
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Eq)]
pub enum Preset {
    Std,
    NASCOM,
    Acorn,
    MSX1200,
    MSX2400,
}

impl Display for Preset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parity: &str = match self {
            Preset::Std => "Standard",
            Preset::NASCOM => "NASCOM",
            Preset::Acorn => "Acorn",
            Preset::MSX1200 => "MSX1200",
            Preset::MSX2400 => "MSX2400",
        };
        write!(f, "{}", parity)
    }
}

impl From<&str> for Preset {
    fn from(value: &str) -> Self {
        match value.to_uppercase().chars().nth(0).or(Some('S')).unwrap() {
            'S' => Preset::Std,
            'N' => Preset::NASCOM,
            'A' => Preset::Acorn,
            'M' if value.contains("2400") => Preset::MSX2400,
            'M' => Preset::MSX1200,
            _ => Preset::NASCOM,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Eq)]
pub enum Parity {
    NONE,
    EVEN,
    ODD,
    MARK,
    SPACE,
}

impl Display for Parity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parity: &str = match self {
            Parity::EVEN => "Even",
            Parity::ODD => "Odd",
            Parity::MARK => "Mark",
            Parity::NONE => "None",
            Parity::SPACE => "Space",
        };
        write!(f, "{}", parity)
    }
}

impl From<&str> for Parity {
    fn from(value: &str) -> Self {
        match value.to_uppercase().chars().nth(0).or(Some('N')).unwrap() {
            'E' => Parity::EVEN,
            'O' => Parity::ODD,
            'M' => Parity::MARK,
            'N' => Parity::NONE,
            'S' => Parity::SPACE,
            _ => Parity::NONE,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Eq)]
pub enum ZeroCrossingDirection {
    Pos,
    Neg,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ZeroCrossingDetector {
    last_sample: f32,
    hysteresis: f32,
}

/// Zero crossing detector
///
/// ```
/// ```
impl ZeroCrossingDetector {
    pub fn new(hysteresis: f32) -> Self {
        ZeroCrossingDetector {
            last_sample: 0.0,
            hysteresis: hysteresis.abs(),
        }
    }

    pub fn process(&mut self, input: (usize, f32)) -> Option<(usize, ZeroCrossingDirection)> {
        let (last_sample_index, sample) = input;
        if sample.abs() < self.hysteresis {
            return None;
        }
        let last_sample = self.last_sample;
        self.last_sample = sample;
        if (sample >= 0.0) != (last_sample >= 0.0) {
            match sample >= 0.0 {
                true => Some((last_sample_index, ZeroCrossingDirection::Pos)),
                false => Some((last_sample_index, ZeroCrossingDirection::Neg)),
            }
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct FrequencyIdentifier {
    start_direction: ZeroCrossingDirection,
    last_sample_idx: Option<usize>,
    sample_frequency: f32,
}

impl FrequencyIdentifier {
    pub fn new(start_direction: ZeroCrossingDirection, sample_frequency: u32) -> Self {
        FrequencyIdentifier {
            start_direction,
            last_sample_idx: None,
            sample_frequency: sample_frequency as f32,
        }
    }

    pub fn process(&mut self, input: (usize, ZeroCrossingDirection)) -> Option<(usize, f32)> {
        let (sample_index, direction) = input;
        if direction != self.start_direction {
            return None;
        }

        let idx = self.last_sample_idx;
        self.last_sample_idx = Some(sample_index);
        idx.and_then(|idx| {
            Some((
                sample_index,
                self.sample_frequency / ((sample_index - idx) as f32),
            ))
        })
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Eq)]
pub enum SignalCondition {
    Space,
    Mark,
    Error,
}

impl Display for SignalCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parity: &str = match self {
            SignalCondition::Error => "Error",
            SignalCondition::Mark => "Mark",
            SignalCondition::Space => "Space",
        };
        write!(f, "{}", parity)
    }
}

impl From<&str> for SignalCondition {
    fn from(value: &str) -> Self {
        match value.to_uppercase().chars().nth(0).or(Some('N')).unwrap() {
            'M' => SignalCondition::Mark,
            'S' => SignalCondition::Space,
            _ => SignalCondition::Error,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct HiLowIdentifier {
    high_level_bounds: Range<f32>,
    low_level_bounds: Range<f32>,
    low_symbol: (u8, SignalCondition),
    high_symbol: (u8, SignalCondition),
    bitcount: u8,
}

impl HiLowIdentifier {
    pub fn new(
        frequency_high: u32,
        frequency_low: u32,
        tolerance_percent: u8,
        low_symbol: (u8, SignalCondition),
        high_symbol: (u8, SignalCondition),
    ) -> Option<Self> {
        let high_f32 = frequency_high as f32;
        let low_f32 = frequency_low as f32;
        let tolerance = tolerance_percent as f32 / 100.0;
        let high_level_bounds = high_f32 - high_f32 * tolerance..high_f32 + high_f32 * tolerance;
        let low_level_bounds = low_f32 - low_f32 * tolerance..low_f32 + low_f32 * tolerance;

        if high_level_bounds.contains(&low_f32)
            || low_level_bounds.contains(&high_f32)
            || tolerance_percent > 50
            || low_symbol.1 == high_symbol.1
        {
            return None;
        }

        Some(Self {
            high_level_bounds,
            low_level_bounds,
            low_symbol,
            high_symbol,
            bitcount: 0,
        })
    }

    pub fn process(&mut self, input: (usize, f32)) -> Option<(usize, SignalCondition)> {
        let (sample_index, frequency) = input;
        self.bitcount += 1;

        if self.high_level_bounds.contains(&frequency) {
            if self.bitcount < self.high_symbol.0 {
                None
            } else {
                self.bitcount = 0;
                Some((sample_index, self.high_symbol.1))
            }
        } else if self.low_level_bounds.contains(&frequency) {
            if self.bitcount < self.low_symbol.0 {
                None
            } else {
                self.bitcount = 0;
                Some((sample_index, self.low_symbol.1))
            }
        } else {
            Some((sample_index, SignalCondition::Error))
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum DecoderState {
    WaitForStartBit(DecoderStateStartBit),
    WaitForDataBit(DecoderStateDataBit),
    WaitForParity(DecoderStateParity),
    WaitForStopBit(DecoderStateStopBit),
    DataOut(u8),
}

impl Eq for DecoderState {}
impl PartialEq for DecoderState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::WaitForStartBit(_), Self::WaitForStartBit(_)) => true,
            (Self::WaitForDataBit(val), Self::WaitForDataBit(val2)) if *val == *val2 => true,
            (Self::WaitForParity(_), Self::WaitForParity(_)) => true,
            (Self::WaitForStopBit(_), Self::WaitForStopBit(_)) => true,
            (Self::DataOut(val), Self::DataOut(val2)) if *val == *val2 => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DecoderError {
    Sync,
    Parity,
    Signal,
    Config,
    IO(String),
    Other(String),
}

impl PartialEq for DecoderError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DecoderError::Config, DecoderError::Config) => true,
            (DecoderError::Sync, DecoderError::Sync) => true,
            (DecoderError::Parity, DecoderError::Parity) => true,
            (DecoderError::Signal, DecoderError::Signal) => true,
            (DecoderError::IO(val), DecoderError::IO(val2)) if *val == *val2 => true,
            (DecoderError::Other(val), DecoderError::Other(val2)) if *val == *val2 => true,
            _ => false,
        }
    }
}

impl From<std::io::Error> for DecoderError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value.to_string())
    }
}

impl Display for DecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DecoderError::IO(val) => val.clone(),
                DecoderError::Sync => "sync".to_string(),
                DecoderError::Parity => "parity".to_string(),
                DecoderError::Signal => "signal".to_string(),
                DecoderError::Config => "config".to_string(),
                DecoderError::Other(val) => val.to_string(),
            }
        )
    }
}

impl Error for DecoderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct DecoderStateStartBit {
    config: DecoderConfig,
    output: [bool; MAX_NUM_STARTBITS],
    idx: usize,
}

impl DecoderStateStartBit {
    fn new(config: DecoderConfig) -> Self {
        DecoderStateStartBit {
            config,
            output: [false; MAX_NUM_STARTBITS],
            idx: 0,
        }
    }

    fn process(&mut self, level: SignalCondition) -> Result<DecoderState, DecoderError> {
        if level == self.config.startbits.1 {
            self.idx += 1;
            if self.idx >= self.config.startbits.0 {
                Ok(DecoderState::WaitForDataBit(DecoderStateDataBit::new(
                    self.config,
                )))
            } else {
                Ok(DecoderState::WaitForStartBit(*self))
            }
        } else if level == SignalCondition::Error {
            // Propagate error
            Err(DecoderError::Signal)
        } else if self.idx > 0 {
            // We expected a 2nd start bit here but we did not get it
            self.idx = 0;
            Err(DecoderError::Sync)
        } else {
            Ok(DecoderState::WaitForStartBit(*self))
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct DecoderStateDataBit {
    config: DecoderConfig,
    data: [bool; MAX_NUM_DATABITS],
    idx: usize,
}

impl<'a> DecoderStateDataBit {
    fn new(config: DecoderConfig) -> Self {
        DecoderStateDataBit {
            config,
            data: [false; MAX_NUM_DATABITS],
            idx: 0,
        }
    }

    fn process(&mut self, level: SignalCondition) -> Result<DecoderState, DecoderError> {
        self.data[self.idx] = match level {
            SignalCondition::Space => Ok(false),
            SignalCondition::Mark => Ok(true),
            _ => Err(DecoderError::Signal),
        }?;
        self.idx += 1;
        if self.idx >= self.config.num_databits {
            if self.config.parity != Parity::NONE {
                Ok(DecoderState::WaitForParity(DecoderStateParity::new(
                    self.config,
                    &self.data,
                )))
            } else {
                Ok(DecoderState::WaitForStopBit(DecoderStateStopBit::new(
                    self.config,
                    &self.data,
                )))
            }
        } else {
            Ok(DecoderState::WaitForDataBit(*self))
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct DecoderStateParity {
    config: DecoderConfig,
    data: [bool; MAX_NUM_DATABITS],
}

impl DecoderStateParity {
    fn new(config: DecoderConfig, data: &[bool; MAX_NUM_DATABITS]) -> Self {
        DecoderStateParity {
            config,
            data: *data,
        }
    }

    fn process(&mut self, level: SignalCondition) -> Result<DecoderState, DecoderError> {
        let mut marks_count_is_even = bool_vec_to_u8(&self.data).count_ones() % 2 == 0;
        if level == SignalCondition::Space {
            marks_count_is_even = !marks_count_is_even;
        }

        let parity_valid = match self.config.parity {
            Parity::EVEN if marks_count_is_even => true,
            Parity::ODD if !marks_count_is_even => true,
            Parity::MARK if level == SignalCondition::Mark => true,
            Parity::SPACE if level == SignalCondition::Space => true,
            Parity::NONE => {
                panic!("Parity state should never become active when parity is set to none");
            }
            _ => false,
        };
        if parity_valid {
            Ok(DecoderState::WaitForStopBit(DecoderStateStopBit::new(
                self.config,
                &self.data,
            )))
        } else {
            Err(DecoderError::Parity)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct DecoderStateStopBit {
    config: DecoderConfig,
    data: [bool; MAX_NUM_DATABITS],
    num_stopbits_received: usize,
}

impl DecoderStateStopBit {
    fn new(config: DecoderConfig, data: &[bool; MAX_NUM_DATABITS]) -> Self {
        DecoderStateStopBit {
            config,
            data: *data,
            num_stopbits_received: 0,
        }
    }

    fn process(&mut self, level: SignalCondition) -> Result<DecoderState, DecoderError> {
        if level == self.config.stopbits.1 {
            self.num_stopbits_received += 1;
            if self.num_stopbits_received < self.config.stopbits.0 {
                Ok(DecoderState::WaitForStopBit(*self))
            } else {
                Ok(DecoderState::DataOut(bool_vec_to_u8(&self.data)))
            }
        } else if level == SignalCondition::Error {
            Err(DecoderError::Signal)
        } else {
            Err(DecoderError::Sync)
        }
    }
}

fn bool_vec_to_u8(data: &[bool; MAX_NUM_DATABITS]) -> u8 {
    let mut out = 0u8;
    for i in 0..MAX_NUM_DATABITS {
        if data[i] {
            out += 1u8 << i;
        }
    }
    out
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Symbol {
    pub frequency: usize,
    pub periods: usize,
    pub signal: SignalCondition,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DecoderConfig {
    pub startbits: (usize, SignalCondition),
    pub num_databits: usize,
    pub parity: Parity,
    pub stopbits: (usize, SignalCondition),
    pub channels: Channels,
    pub symbols: [Symbol; 2],
    pub frequency_tolerance: usize,
}

impl DecoderConfig {
    pub fn get_preset(preset: &Preset) -> DecoderConfig {
        match preset {
            Preset::Std => Self {
                startbits: (1, SignalCondition::Space),
                num_databits: 8,
                parity: Parity::NONE,
                stopbits: (2, SignalCondition::Mark),
                channels: Channels::All,
                symbols: [
                    Symbol {
                        frequency: 1200,
                        periods: 4,
                        signal: SignalCondition::Space,
                    },
                    Symbol {
                        frequency: 2400,
                        periods: 8,
                        signal: SignalCondition::Mark,
                    },
                ],
                frequency_tolerance: 10,
            },
            Preset::NASCOM | Preset::Acorn => Self {
                startbits: (1, SignalCondition::Space),
                num_databits: 8,
                parity: Parity::NONE,
                stopbits: (1, SignalCondition::Mark),
                channels: Channels::All,
                symbols: [
                    Symbol {
                        frequency: 1200,
                        periods: 1,
                        signal: SignalCondition::Space,
                    },
                    Symbol {
                        frequency: 2400,
                        periods: 2,
                        signal: SignalCondition::Mark,
                    },
                ],
                frequency_tolerance: 10,
            },
            Preset::MSX1200 => Self {
                startbits: (1, SignalCondition::Space),
                num_databits: 8,
                parity: Parity::NONE,
                stopbits: (2, SignalCondition::Mark),
                channels: Channels::All,
                symbols: [
                    Symbol {
                        frequency: 1200,
                        periods: 1,
                        signal: SignalCondition::Space,
                    },
                    Symbol {
                        frequency: 2400,
                        periods: 2,
                        signal: SignalCondition::Mark,
                    },
                ],
                frequency_tolerance: 10,
            },
            Preset::MSX2400 => Self {
                startbits: (1, SignalCondition::Space),
                num_databits: 8,
                parity: Parity::NONE,
                stopbits: (2, SignalCondition::Mark),
                channels: Channels::All,
                symbols: [
                    Symbol {
                        frequency: 2400,
                        periods: 1,
                        signal: SignalCondition::Space,
                    },
                    Symbol {
                        frequency: 4800,
                        periods: 2,
                        signal: SignalCondition::Mark,
                    },
                ],
                frequency_tolerance: 10,
            },
        }
    }

    fn validate(&self) -> Option<Self> {
        if self.num_databits >= MIN_NUM_DATABITS
            && self.num_databits <= MAX_NUM_DATABITS
            && self.startbits.0 >= MIN_NUM_STARTBITS
            && self.startbits.0 <= MAX_NUM_STARTBITS
            && self.stopbits.0 >= MIN_NUM_STOPBITS
            && self.stopbits.0 <= MAX_NUM_STOPBITS
        {
            Some(*self)
        } else {
            None
        }
    }
}

impl Default for DecoderConfig {
    fn default() -> Self {
        DecoderConfig::get_preset(&Preset::NASCOM)
    }
}

impl Display for DecoderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\
Channels:  {}
Startbits: {} ({})
Databits:  {}
Parity:    {}
Stopbits:  {} ({})",
            self.channels,
            self.startbits.0,
            self.startbits.1,
            self.num_databits,
            self.parity,
            self.stopbits.0,
            self.stopbits.1
        )
    }
}

/// Decoder transforms from SignalCondition -> DecoderState
#[derive(Debug, Clone)]
pub struct Decoder {
    config: DecoderConfig,
    state: Result<DecoderState, DecoderError>,
}

impl Decoder {
    pub fn process(&mut self, input: SignalCondition) -> Result<u8, Option<DecoderError>> {
        let mut state = self.state.clone()?;
        match &mut state {
            DecoderState::WaitForStartBit(state) => {
                self.state = state.process(input);
                match &self.state {
                    Err(error) => {
                        let error = error.clone();
                        self.reset();
                        Err(Some(error))
                    }
                    _ => Err(None),
                }
            }
            DecoderState::WaitForDataBit(state) => {
                self.state = state.process(input);
                match &self.state {
                    Err(error) => {
                        let error = error.clone();
                        self.reset();
                        Err(Some(error))
                    }
                    _ => Err(None),
                }
            }
            DecoderState::WaitForParity(state) => {
                self.state = state.process(input);
                match &self.state {
                    Err(error) => {
                        let error = error.clone();
                        self.reset();
                        Err(Some(error))
                    }
                    _ => Err(None),
                }
            }
            DecoderState::WaitForStopBit(state) => {
                self.state = state.process(input);
                match &self.state {
                    Ok(DecoderState::DataOut(val)) => {
                        let val = val.clone();
                        self.reset();
                        Ok(val)
                    }
                    Err(error) => {
                        let error = error.clone();
                        self.reset();
                        Err(Some(error))
                    }
                    _ => Err(None),
                }
            }
            DecoderState::DataOut(_) => {
                panic!("The dataout state should never be reached here");
            }
        }
    }

    pub fn new(config: DecoderConfig) -> Option<Self> {
        let mut new = Self {
            config: config.validate()?,
            state: Ok(DecoderState::WaitForStartBit(DecoderStateStartBit::new(
                config,
            ))),
        };
        new.reset();
        Some(new)
    }

    pub fn reset(&mut self) {
        self.state = Ok(DecoderState::WaitForStartBit(DecoderStateStartBit::new(
            self.config,
        )));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zerocrossingdetector_transition_both_ways() {
        let mut zc = ZeroCrossingDetector::new(0.0);
        let data = [0f32, -1.0, -1.0, 1.0, 1.0, 1.0, -1.0, -1.0].into_iter();
        let output = data
            .enumerate()
            .filter_map(|val| zc.process(val))
            .collect::<Vec<_>>();

        assert_eq!(
            output,
            vec![
                (1, ZeroCrossingDirection::Neg),
                (3, ZeroCrossingDirection::Pos),
                (6, ZeroCrossingDirection::Neg)
            ]
        );
    }

    #[test]
    fn zerocrossingdetector_hysteresis() {
        let mut zc = ZeroCrossingDetector::new(0.1);

        // The default state is Pos so with 0.1 hysteresis, transition is not triggered until sample 8 as it is < -0.1
        let data = [0f32, 1.0, 1.0, -0.09, 0.01, 0.0, 1.0, 1.0, -0.11, 0.09].into_iter();
        let output = data
            .enumerate()
            .filter_map(|val| zc.process(val))
            .collect::<Vec<_>>();

        assert_eq!(output, vec![(8, ZeroCrossingDirection::Neg)]);
    }

    #[test]
    fn frequency_identifier_valid_signal() {
        let sample_frequency = 4000.0f32;
        let mut frq_identifier =
            FrequencyIdentifier::new(ZeroCrossingDirection::Pos, sample_frequency as u32);

        // Periods when looking from pos->pos:
        // 4, 5, 100
        // When looking from neg->neg:
        // 4, 80
        let data = [
            (0, ZeroCrossingDirection::Pos),
            (2, ZeroCrossingDirection::Neg),
            (4, ZeroCrossingDirection::Pos),
            (6, ZeroCrossingDirection::Neg),
            (9, ZeroCrossingDirection::Pos),
            (86, ZeroCrossingDirection::Neg),
            (109, ZeroCrossingDirection::Pos),
        ];
        let output = data
            .into_iter()
            .filter_map(|val| frq_identifier.process(val))
            .collect::<Vec<_>>();

        assert_eq!(
            output,
            vec![
                (4, sample_frequency / 4.0),
                (9, sample_frequency / 5.0),
                (109, sample_frequency / 100.0)
            ]
        );

        // Invert the direction but use same input data
        let mut frq_identifier =
            FrequencyIdentifier::new(ZeroCrossingDirection::Neg, sample_frequency as u32);

        let output = data
            .into_iter()
            .filter_map(|val| frq_identifier.process(val))
            .collect::<Vec<_>>();

        assert_eq!(
            output,
            vec![(6, sample_frequency / 4.0), (86, sample_frequency / 80.0)]
        );
    }

    #[test]
    /// It does not matter if transitions are missing, i.e. two of the same directions follow each other.
    /// It will only look for either Pos or Neg depending on the config and ignore the other.
    fn frequency_identifier_missing_transitions() {
        let sample_frequency = 4000.0f32;
        let mut frq_identifier =
            FrequencyIdentifier::new(ZeroCrossingDirection::Pos, sample_frequency as u32);

        let data = [
            (0, ZeroCrossingDirection::Pos),
            (2, ZeroCrossingDirection::Pos),
            (6, ZeroCrossingDirection::Neg),
            (600, ZeroCrossingDirection::Neg),
            (6, ZeroCrossingDirection::Neg),
            (6, ZeroCrossingDirection::Neg),
            (7, ZeroCrossingDirection::Pos),
        ];

        let output = data
            .into_iter()
            .filter_map(|val| frq_identifier.process(val))
            .collect::<Vec<_>>();

        assert_eq!(
            output,
            vec![(2, sample_frequency / 2.0), (7, sample_frequency / 5.0)]
        );
    }

    #[test]
    fn decoder_8n1_byte_success() {
        let mut decoder = Decoder::new(DecoderConfig::default()).unwrap();

        // Two bytes: 0b10000000 (0x80) and 0b01010101 (0x55)
        let data: Vec<(usize, SignalCondition)> = vec![
            (0, SignalCondition::Mark),   // Idle
            (1, SignalCondition::Mark),   // Idle
            (9, SignalCondition::Space),  // Start bit
            (10, SignalCondition::Space), // Data bit 0
            (11, SignalCondition::Space), // Data bit 1
            (12, SignalCondition::Space), // Data bit 2
            (13, SignalCondition::Space), // Data bit 3
            (14, SignalCondition::Space), // Data bit 4
            (15, SignalCondition::Space), // Data bit 5
            (16, SignalCondition::Space), // Data bit 6
            (17, SignalCondition::Mark),  // Data bit 7
            (18, SignalCondition::Mark),  // Stop bit
            (29, SignalCondition::Space), // Start bit
            (30, SignalCondition::Mark),  // Data bit 0
            (31, SignalCondition::Space), // Data bit 1
            (32, SignalCondition::Mark),  // Data bit 2
            (33, SignalCondition::Space), // Data bit 3
            (34, SignalCondition::Mark),  // Data bit 4
            (35, SignalCondition::Space), // Data bit 5
            (36, SignalCondition::Mark),  // Data bit 6
            (37, SignalCondition::Space), // Data bit 7
            (38, SignalCondition::Mark),  // Stop bit
            (40, SignalCondition::Mark),  // Idle
        ];

        let output = data
            .into_iter()
            .map(|(idx, val)| (idx, decoder.process(val)))
            .filter(|(_idx, val)| {
                val.as_ref().is_ok()
                    | (val.as_ref().is_err() && val.as_ref().err().unwrap().is_some())
            })
            .collect::<Vec<_>>();

        assert_eq!(output, vec![(18usize, Ok(0x80)), (38, Ok(0x55)),]);
    }

    #[test]
    fn decoder_8n1_byte_missing_stop_bit() {
        let mut decoder = Decoder::new(DecoderConfig::default()).unwrap();

        // One byte: 0b10000000 (0xF0) but invalid stop bit
        let data: Vec<(usize, SignalCondition)> = vec![
            (0, SignalCondition::Mark),   // Idle
            (1, SignalCondition::Mark),   // Idle
            (9, SignalCondition::Space),  // Start bit
            (10, SignalCondition::Mark),  // Data bit 0
            (11, SignalCondition::Mark),  // Data bit 1
            (12, SignalCondition::Mark),  // Data bit 2
            (13, SignalCondition::Mark),  // Data bit 3
            (14, SignalCondition::Mark),  // Data bit 4
            (15, SignalCondition::Mark),  // Data bit 5
            (16, SignalCondition::Mark),  // Data bit 6
            (17, SignalCondition::Space), // Data bit 7
            (18, SignalCondition::Space), // Stop bit
            (19, SignalCondition::Mark),  // Idle
        ];

        let output = data
            .into_iter()
            .map(|(idx, val)| (idx, decoder.process(val)))
            .filter(|(_idx, val)| {
                val.as_ref().is_ok()
                    | (val.as_ref().is_err() && val.as_ref().err().unwrap().is_some())
            })
            .collect::<Vec<_>>();

        assert_eq!(output, vec![(18, Err(Some(DecoderError::Sync)))]);
    }

    #[test]
    fn decoder_8n1_byte_missing_error_input() {
        let mut decoder = Decoder::new(DecoderConfig::default()).unwrap();

        // One byte: 0b10000000 (0xF0) but invalid stop bit
        let data: Vec<(usize, SignalCondition)> = vec![
            (0, SignalCondition::Mark),   // Idle
            (1, SignalCondition::Mark),   // Idle
            (9, SignalCondition::Space),  // Start bit
            (10, SignalCondition::Mark),  // Data bit 0
            (11, SignalCondition::Mark),  // Data bit 1
            (12, SignalCondition::Mark),  // Data bit 2
            (13, SignalCondition::Mark),  // Data bit 3
            (14, SignalCondition::Mark),  // Data bit 4
            (15, SignalCondition::Error), // Data bit 5
            (16, SignalCondition::Mark),  // Data bit 6
            (17, SignalCondition::Space), // Data bit 7
            (18, SignalCondition::Space), // Stop bit
        ];

        let output = data
            .into_iter()
            .map(|(idx, val)| (idx, decoder.process(val)))
            .filter(|(_idx, val)| {
                val.as_ref().is_ok()
                    | (val.as_ref().is_err() && val.as_ref().err().unwrap().is_some())
            })
            .collect::<Vec<_>>();

        assert_eq!(output, vec![(15, Err(Some(DecoderError::Signal))),]);
    }
}
