use serde::Serialize;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt;

use crate::hev1::HvcCArray;
use crate::mp4a::EsdsBox;
use crate::mp4box::*;
use crate::*;

pub use bytes::Bytes;
pub use num_rational::Ratio;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FixedPointU8(Ratio<u16>);

impl FixedPointU8 {
    pub fn new(val: u8) -> Self {
        Self(Ratio::new_raw(val as u16 * 0x100, 0x100))
    }

    pub fn new_raw(val: u16) -> Self {
        Self(Ratio::new_raw(val, 0x100))
    }

    pub fn value(&self) -> u8 {
        self.0.to_integer() as u8
    }

    pub fn raw_value(&self) -> u16 {
        *self.0.numer()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FixedPointI8(Ratio<i16>);

impl FixedPointI8 {
    pub fn new(val: i8) -> Self {
        Self(Ratio::new_raw(val as i16 * 0x100, 0x100))
    }

    pub fn new_raw(val: i16) -> Self {
        Self(Ratio::new_raw(val, 0x100))
    }

    pub fn value(&self) -> i8 {
        self.0.to_integer() as i8
    }

    pub fn raw_value(&self) -> i16 {
        *self.0.numer()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FixedPointU16(Ratio<u32>);

impl FixedPointU16 {
    pub fn new(val: u16) -> Self {
        Self(Ratio::new_raw(val as u32 * 0x10000, 0x10000))
    }

    pub fn new_raw(val: u32) -> Self {
        Self(Ratio::new_raw(val, 0x10000))
    }

    pub fn value(&self) -> u16 {
        self.0.to_integer() as u16
    }

    pub fn raw_value(&self) -> u32 {
        *self.0.numer()
    }
}

impl fmt::Debug for BoxType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fourcc: FourCC = From::from(*self);
        write!(f, "{fourcc}")
    }
}

impl fmt::Display for BoxType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fourcc: FourCC = From::from(*self);
        write!(f, "{fourcc}")
    }
}

#[derive(Default, PartialEq, Eq, Clone, Copy, Serialize)]
pub struct FourCC {
    pub value: [u8; 4],
}

impl std::str::FromStr for FourCC {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if let [a, b, c, d] = s.as_bytes() {
            Ok(Self {
                value: [*a, *b, *c, *d],
            })
        } else {
            Err(Error::InvalidData("expected exactly four bytes in string"))
        }
    }
}

impl From<u32> for FourCC {
    fn from(number: u32) -> Self {
        FourCC {
            value: number.to_be_bytes(),
        }
    }
}

impl From<FourCC> for u32 {
    fn from(fourcc: FourCC) -> u32 {
        (&fourcc).into()
    }
}

impl From<&FourCC> for u32 {
    fn from(fourcc: &FourCC) -> u32 {
        u32::from_be_bytes(fourcc.value)
    }
}

impl From<[u8; 4]> for FourCC {
    fn from(value: [u8; 4]) -> FourCC {
        FourCC { value }
    }
}

impl From<BoxType> for FourCC {
    fn from(t: BoxType) -> FourCC {
        let box_num: u32 = Into::into(t);
        From::from(box_num)
    }
}

impl fmt::Debug for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code: u32 = self.into();
        let string = String::from_utf8_lossy(&self.value[..]);
        write!(f, "{string} / {code:#010X}")
    }
}

impl fmt::Display for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.value[..]))
    }
}

const DISPLAY_TYPE_VIDEO: &str = "Video";
const DISPLAY_TYPE_AUDIO: &str = "Audio";
const DISPLAY_TYPE_SUBTITLE: &str = "Subtitle";

const HANDLER_TYPE_VIDEO: &str = "vide";
const HANDLER_TYPE_VIDEO_FOURCC: [u8; 4] = [b'v', b'i', b'd', b'e'];

const HANDLER_TYPE_AUDIO: &str = "soun";
const HANDLER_TYPE_AUDIO_FOURCC: [u8; 4] = [b's', b'o', b'u', b'n'];

const HANDLER_TYPE_SUBTITLE: &str = "sbtl";
const HANDLER_TYPE_SUBTITLE_FOURCC: [u8; 4] = [b's', b'b', b't', b'l'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackType {
    Video,
    Audio,
    Subtitle,
}

impl fmt::Display for TrackType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            TrackType::Video => DISPLAY_TYPE_VIDEO,
            TrackType::Audio => DISPLAY_TYPE_AUDIO,
            TrackType::Subtitle => DISPLAY_TYPE_SUBTITLE,
        };
        write!(f, "{s}")
    }
}

impl TryFrom<&str> for TrackType {
    type Error = Error;
    fn try_from(handler: &str) -> Result<TrackType> {
        match handler {
            HANDLER_TYPE_VIDEO => Ok(TrackType::Video),
            HANDLER_TYPE_AUDIO => Ok(TrackType::Audio),
            HANDLER_TYPE_SUBTITLE => Ok(TrackType::Subtitle),
            _ => Err(Error::InvalidData("unsupported handler type")),
        }
    }
}

impl TryFrom<&FourCC> for TrackType {
    type Error = Error;
    fn try_from(fourcc: &FourCC) -> Result<TrackType> {
        match fourcc.value {
            HANDLER_TYPE_VIDEO_FOURCC => Ok(TrackType::Video),
            HANDLER_TYPE_AUDIO_FOURCC => Ok(TrackType::Audio),
            HANDLER_TYPE_SUBTITLE_FOURCC => Ok(TrackType::Subtitle),
            _ => Err(Error::InvalidData("unsupported handler type")),
        }
    }
}

impl From<TrackType> for FourCC {
    fn from(t: TrackType) -> FourCC {
        match t {
            TrackType::Video => HANDLER_TYPE_VIDEO_FOURCC.into(),
            TrackType::Audio => HANDLER_TYPE_AUDIO_FOURCC.into(),
            TrackType::Subtitle => HANDLER_TYPE_SUBTITLE_FOURCC.into(),
        }
    }
}

impl From<TrackType> for String {
    fn from(t: TrackType) -> String {
        match t {
            TrackType::Video => "VideoHandler".to_string(),
            TrackType::Audio => "SoundHandler".to_string(),
            TrackType::Subtitle => "ClosedCaptionHandler".to_string(),
        }
    }
}

const MEDIA_TYPE_H264: &str = "h264";
const MEDIA_TYPE_H265: &str = "h265";
const MEDIA_TYPE_VP9: &str = "vp9";
const MEDIA_TYPE_AAC: &str = "aac";
const MEDIA_TYPE_TTXT: &str = "ttxt";
const MEDIA_TYPE_OPUS: &str = "opus";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    H264,
    H265,
    VP9,
    AAC,
    OPUS,
    TTXT,
}

impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s: &str = self.into();
        write!(f, "{s}")
    }
}

impl TryFrom<&str> for MediaType {
    type Error = Error;
    fn try_from(media: &str) -> Result<MediaType> {
        match media {
            MEDIA_TYPE_H264 => Ok(MediaType::H264),
            MEDIA_TYPE_H265 => Ok(MediaType::H265),
            MEDIA_TYPE_VP9 => Ok(MediaType::VP9),
            MEDIA_TYPE_AAC => Ok(MediaType::AAC),
            MEDIA_TYPE_TTXT => Ok(MediaType::TTXT),
            MEDIA_TYPE_OPUS => Ok(MediaType::OPUS),
            _ => Err(Error::InvalidData("unsupported media type")),
        }
    }
}

impl From<MediaType> for &str {
    fn from(t: MediaType) -> &'static str {
        match t {
            MediaType::H264 => MEDIA_TYPE_H264,
            MediaType::H265 => MEDIA_TYPE_H265,
            MediaType::VP9 => MEDIA_TYPE_VP9,
            MediaType::AAC => MEDIA_TYPE_AAC,
            MediaType::TTXT => MEDIA_TYPE_TTXT,
            MediaType::OPUS => MEDIA_TYPE_OPUS,
        }
    }
}

impl From<&MediaType> for &str {
    fn from(t: &MediaType) -> &'static str {
        match t {
            MediaType::H264 => MEDIA_TYPE_H264,
            MediaType::H265 => MEDIA_TYPE_H265,
            MediaType::VP9 => MEDIA_TYPE_VP9,
            MediaType::AAC => MEDIA_TYPE_AAC,
            MediaType::TTXT => MEDIA_TYPE_TTXT,
            MediaType::OPUS => MEDIA_TYPE_OPUS,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AvcProfile {
    AvcConstrainedBaseline, // 66 with constraint set 1
    AvcBaseline,            // 66,
    AvcMain,                // 77,
    AvcExtended,            // 88,
    AvcHigh,                // 100
                            // TODO Progressive High Profile, Constrained High Profile, ...
}

impl TryFrom<(u8, u8)> for AvcProfile {
    type Error = Error;
    fn try_from(value: (u8, u8)) -> Result<AvcProfile> {
        let profile = value.0;
        let constraint_set1_flag = (value.1 & 0x40) >> 7;
        match (profile, constraint_set1_flag) {
            (66, 1) => Ok(AvcProfile::AvcConstrainedBaseline),
            (66, 0) => Ok(AvcProfile::AvcBaseline),
            (77, _) => Ok(AvcProfile::AvcMain),
            (88, _) => Ok(AvcProfile::AvcExtended),
            (100, _) => Ok(AvcProfile::AvcHigh),
            _ => Err(Error::InvalidData("unsupported avc profile")),
        }
    }
}

impl fmt::Display for AvcProfile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let profile = match self {
            AvcProfile::AvcConstrainedBaseline => "Constrained Baseline",
            AvcProfile::AvcBaseline => "Baseline",
            AvcProfile::AvcMain => "Main",
            AvcProfile::AvcExtended => "Extended",
            AvcProfile::AvcHigh => "High",
        };
        write!(f, "{profile}")
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AudioObjectType {
    AacMain = 1,                                       // AAC Main Profile
    AacLowComplexity = 2,                              // AAC Low Complexity
    AacScalableSampleRate = 3,                         // AAC Scalable Sample Rate
    AacLongTermPrediction = 4,                         // AAC Long Term Predictor
    SpectralBandReplication = 5,                       // Spectral band Replication
    AACScalable = 6,                                   // AAC Scalable
    TwinVQ = 7,                                        // Twin VQ
    CodeExcitedLinearPrediction = 8,                   // CELP
    HarmonicVectorExcitationCoding = 9,                // HVXC
    TextToSpeechtInterface = 12,                       // TTSI
    MainSynthetic = 13,                                // Main Synthetic
    WavetableSynthesis = 14,                           // Wavetable Synthesis
    GeneralMIDI = 15,                                  // General MIDI
    AlgorithmicSynthesis = 16,                         // Algorithmic Synthesis
    ErrorResilientAacLowComplexity = 17,               // ER AAC LC
    ErrorResilientAacLongTermPrediction = 19,          // ER AAC LTP
    ErrorResilientAacScalable = 20,                    // ER AAC Scalable
    ErrorResilientAacTwinVQ = 21,                      // ER AAC TwinVQ
    ErrorResilientAacBitSlicedArithmeticCoding = 22,   // ER Bit Sliced Arithmetic Coding
    ErrorResilientAacLowDelay = 23,                    // ER AAC Low Delay
    ErrorResilientCodeExcitedLinearPrediction = 24,    // ER CELP
    ErrorResilientHarmonicVectorExcitationCoding = 25, // ER HVXC
    ErrorResilientHarmonicIndividualLinesNoise = 26,   // ER HILN
    ErrorResilientParametric = 27,                     // ER Parametric
    SinuSoidalCoding = 28,                             // SSC
    ParametricStereo = 29,                             // PS
    MpegSurround = 30,                                 // MPEG Surround
    MpegLayer1 = 32,                                   // MPEG Layer 1
    MpegLayer2 = 33,                                   // MPEG Layer 2
    MpegLayer3 = 34,                                   // MPEG Layer 3
    DirectStreamTransfer = 35,                         // DST Direct Stream Transfer
    AudioLosslessCoding = 36,                          // ALS Audio Lossless Coding
    ScalableLosslessCoding = 37,                       // SLC Scalable Lossless Coding
    ScalableLosslessCodingNoneCore = 38,               // SLC non-core
    ErrorResilientAacEnhancedLowDelay = 39,            // ER AAC ELD
    SymbolicMusicRepresentationSimple = 40,            // SMR Simple
    SymbolicMusicRepresentationMain = 41,              // SMR Main
    UnifiedSpeechAudioCoding = 42,                     // USAC
    SpatialAudioObjectCoding = 43,                     // SAOC
    LowDelayMpegSurround = 44,                         // LD MPEG Surround
    SpatialAudioObjectCodingDialogueEnhancement = 45,  // SAOC-DE
    AudioSync = 46,                                    // Audio Sync
}

impl TryFrom<u8> for AudioObjectType {
    type Error = Error;
    fn try_from(value: u8) -> Result<AudioObjectType> {
        match value {
            1 => Ok(AudioObjectType::AacMain),
            2 => Ok(AudioObjectType::AacLowComplexity),
            3 => Ok(AudioObjectType::AacScalableSampleRate),
            4 => Ok(AudioObjectType::AacLongTermPrediction),
            5 => Ok(AudioObjectType::SpectralBandReplication),
            6 => Ok(AudioObjectType::AACScalable),
            7 => Ok(AudioObjectType::TwinVQ),
            8 => Ok(AudioObjectType::CodeExcitedLinearPrediction),
            9 => Ok(AudioObjectType::HarmonicVectorExcitationCoding),
            12 => Ok(AudioObjectType::TextToSpeechtInterface),
            13 => Ok(AudioObjectType::MainSynthetic),
            14 => Ok(AudioObjectType::WavetableSynthesis),
            15 => Ok(AudioObjectType::GeneralMIDI),
            16 => Ok(AudioObjectType::AlgorithmicSynthesis),
            17 => Ok(AudioObjectType::ErrorResilientAacLowComplexity),
            19 => Ok(AudioObjectType::ErrorResilientAacLongTermPrediction),
            20 => Ok(AudioObjectType::ErrorResilientAacScalable),
            21 => Ok(AudioObjectType::ErrorResilientAacTwinVQ),
            22 => Ok(AudioObjectType::ErrorResilientAacBitSlicedArithmeticCoding),
            23 => Ok(AudioObjectType::ErrorResilientAacLowDelay),
            24 => Ok(AudioObjectType::ErrorResilientCodeExcitedLinearPrediction),
            25 => Ok(AudioObjectType::ErrorResilientHarmonicVectorExcitationCoding),
            26 => Ok(AudioObjectType::ErrorResilientHarmonicIndividualLinesNoise),
            27 => Ok(AudioObjectType::ErrorResilientParametric),
            28 => Ok(AudioObjectType::SinuSoidalCoding),
            29 => Ok(AudioObjectType::ParametricStereo),
            30 => Ok(AudioObjectType::MpegSurround),
            32 => Ok(AudioObjectType::MpegLayer1),
            33 => Ok(AudioObjectType::MpegLayer2),
            34 => Ok(AudioObjectType::MpegLayer3),
            35 => Ok(AudioObjectType::DirectStreamTransfer),
            36 => Ok(AudioObjectType::AudioLosslessCoding),
            37 => Ok(AudioObjectType::ScalableLosslessCoding),
            38 => Ok(AudioObjectType::ScalableLosslessCodingNoneCore),
            39 => Ok(AudioObjectType::ErrorResilientAacEnhancedLowDelay),
            40 => Ok(AudioObjectType::SymbolicMusicRepresentationSimple),
            41 => Ok(AudioObjectType::SymbolicMusicRepresentationMain),
            42 => Ok(AudioObjectType::UnifiedSpeechAudioCoding),
            43 => Ok(AudioObjectType::SpatialAudioObjectCoding),
            44 => Ok(AudioObjectType::LowDelayMpegSurround),
            45 => Ok(AudioObjectType::SpatialAudioObjectCodingDialogueEnhancement),
            46 => Ok(AudioObjectType::AudioSync),
            _ => Err(Error::InvalidData("invalid audio object type")),
        }
    }
}

impl fmt::Display for AudioObjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let type_str = match self {
            AudioObjectType::AacMain => "AAC Main",
            AudioObjectType::AacLowComplexity => "LC",
            AudioObjectType::AacScalableSampleRate => "SSR",
            AudioObjectType::AacLongTermPrediction => "LTP",
            AudioObjectType::SpectralBandReplication => "SBR",
            AudioObjectType::AACScalable => "Scalable",
            AudioObjectType::TwinVQ => "TwinVQ",
            AudioObjectType::CodeExcitedLinearPrediction => "CELP",
            AudioObjectType::HarmonicVectorExcitationCoding => "HVXC",
            AudioObjectType::TextToSpeechtInterface => "TTSI",
            AudioObjectType::MainSynthetic => "Main Synthetic",
            AudioObjectType::WavetableSynthesis => "Wavetable Synthesis",
            AudioObjectType::GeneralMIDI => "General MIDI",
            AudioObjectType::AlgorithmicSynthesis => "Algorithmic Synthesis",
            AudioObjectType::ErrorResilientAacLowComplexity => "ER AAC LC",
            AudioObjectType::ErrorResilientAacLongTermPrediction => "ER AAC LTP",
            AudioObjectType::ErrorResilientAacScalable => "ER AAC scalable",
            AudioObjectType::ErrorResilientAacTwinVQ => "ER AAC TwinVQ",
            AudioObjectType::ErrorResilientAacBitSlicedArithmeticCoding => "ER AAC BSAC",
            AudioObjectType::ErrorResilientAacLowDelay => "ER AAC LD",
            AudioObjectType::ErrorResilientCodeExcitedLinearPrediction => "ER CELP",
            AudioObjectType::ErrorResilientHarmonicVectorExcitationCoding => "ER HVXC",
            AudioObjectType::ErrorResilientHarmonicIndividualLinesNoise => "ER HILN",
            AudioObjectType::ErrorResilientParametric => "ER Parametric",
            AudioObjectType::SinuSoidalCoding => "SSC",
            AudioObjectType::ParametricStereo => "Parametric Stereo",
            AudioObjectType::MpegSurround => "MPEG surround",
            AudioObjectType::MpegLayer1 => "MPEG Layer 1",
            AudioObjectType::MpegLayer2 => "MPEG Layer 2",
            AudioObjectType::MpegLayer3 => "MPEG Layer 3",
            AudioObjectType::DirectStreamTransfer => "DST",
            AudioObjectType::AudioLosslessCoding => "ALS",
            AudioObjectType::ScalableLosslessCoding => "SLS",
            AudioObjectType::ScalableLosslessCodingNoneCore => "SLS Non-core",
            AudioObjectType::ErrorResilientAacEnhancedLowDelay => "ER AAC ELD",
            AudioObjectType::SymbolicMusicRepresentationSimple => "SMR Simple",
            AudioObjectType::SymbolicMusicRepresentationMain => "SMR Main",
            AudioObjectType::UnifiedSpeechAudioCoding => "USAC",
            AudioObjectType::SpatialAudioObjectCoding => "SAOC",
            AudioObjectType::LowDelayMpegSurround => "LD MPEG Surround",
            AudioObjectType::SpatialAudioObjectCodingDialogueEnhancement => "SAOC-DE",
            AudioObjectType::AudioSync => "Audio Sync",
        };
        write!(f, "{type_str}")
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SampleFreqIndex {
    Freq96000 = 0x0,
    Freq88200 = 0x1,
    Freq64000 = 0x2,
    Freq48000 = 0x3,
    Freq44100 = 0x4,
    Freq32000 = 0x5,
    Freq24000 = 0x6,
    Freq22050 = 0x7,
    Freq16000 = 0x8,
    Freq12000 = 0x9,
    Freq11025 = 0xa,
    Freq8000 = 0xb,
    Freq7350 = 0xc,
}

impl TryFrom<u8> for SampleFreqIndex {
    type Error = Error;
    fn try_from(value: u8) -> Result<SampleFreqIndex> {
        match value {
            0x0 => Ok(SampleFreqIndex::Freq96000),
            0x1 => Ok(SampleFreqIndex::Freq88200),
            0x2 => Ok(SampleFreqIndex::Freq64000),
            0x3 => Ok(SampleFreqIndex::Freq48000),
            0x4 => Ok(SampleFreqIndex::Freq44100),
            0x5 => Ok(SampleFreqIndex::Freq32000),
            0x6 => Ok(SampleFreqIndex::Freq24000),
            0x7 => Ok(SampleFreqIndex::Freq22050),
            0x8 => Ok(SampleFreqIndex::Freq16000),
            0x9 => Ok(SampleFreqIndex::Freq12000),
            0xa => Ok(SampleFreqIndex::Freq11025),
            0xb => Ok(SampleFreqIndex::Freq8000),
            0xc => Ok(SampleFreqIndex::Freq7350),
            _ => Err(Error::InvalidData("invalid sampling frequency index")),
        }
    }
}

impl TryFrom<u32> for SampleFreqIndex {
    type Error = Error;
    fn try_from(value: u32) -> Result<SampleFreqIndex> {
        match value {
            9600 => Ok(SampleFreqIndex::Freq96000),
            88200 => Ok(SampleFreqIndex::Freq88200),
            64000 => Ok(SampleFreqIndex::Freq64000),
            48000 => Ok(SampleFreqIndex::Freq48000),
            44100 => Ok(SampleFreqIndex::Freq44100),
            32000 => Ok(SampleFreqIndex::Freq32000),
            24000 => Ok(SampleFreqIndex::Freq24000),
            22050 => Ok(SampleFreqIndex::Freq22050),
            16000 => Ok(SampleFreqIndex::Freq16000),
            12000 => Ok(SampleFreqIndex::Freq12000),
            11025 => Ok(SampleFreqIndex::Freq11025),
            8000 => Ok(SampleFreqIndex::Freq8000),
            7350 => Ok(SampleFreqIndex::Freq7350),
            _ => Err(Error::InvalidData("invalid sampling frequency index")),
        }
    }
}

impl SampleFreqIndex {
    pub fn freq(&self) -> u32 {
        match *self {
            SampleFreqIndex::Freq96000 => 96000,
            SampleFreqIndex::Freq88200 => 88200,
            SampleFreqIndex::Freq64000 => 64000,
            SampleFreqIndex::Freq48000 => 48000,
            SampleFreqIndex::Freq44100 => 44100,
            SampleFreqIndex::Freq32000 => 32000,
            SampleFreqIndex::Freq24000 => 24000,
            SampleFreqIndex::Freq22050 => 22050,
            SampleFreqIndex::Freq16000 => 16000,
            SampleFreqIndex::Freq12000 => 12000,
            SampleFreqIndex::Freq11025 => 11025,
            SampleFreqIndex::Freq8000 => 8000,
            SampleFreqIndex::Freq7350 => 7350,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ChannelConfig {
    Mono = 0x1,
    Stereo = 0x2,
    Three = 0x3,
    Four = 0x4,
    Five = 0x5,
    FiveOne = 0x6,
    SevenOne = 0x7,
}

impl TryFrom<u8> for ChannelConfig {
    type Error = Error;
    fn try_from(value: u8) -> Result<ChannelConfig> {
        match value {
            0x1 => Ok(ChannelConfig::Mono),
            0x2 => Ok(ChannelConfig::Stereo),
            0x3 => Ok(ChannelConfig::Three),
            0x4 => Ok(ChannelConfig::Four),
            0x5 => Ok(ChannelConfig::Five),
            0x6 => Ok(ChannelConfig::FiveOne),
            0x7 => Ok(ChannelConfig::SevenOne),
            _ => Err(Error::InvalidData("invalid channel configuration")),
        }
    }
}

impl fmt::Display for ChannelConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            ChannelConfig::Mono => "mono",
            ChannelConfig::Stereo => "stereo",
            ChannelConfig::Three => "three",
            ChannelConfig::Four => "four",
            ChannelConfig::Five => "five",
            ChannelConfig::FiveOne => "five.one",
            ChannelConfig::SevenOne => "seven.one",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct AvcConfig {
    pub width: u16,
    pub height: u16,
    pub seq_param_set: Vec<u8>,
    pub pic_param_set: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HevcBoxType {
    Hev1,
    Hvc1,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HevcConfig {
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub configuration_version: Option<u8>,
    pub general_profile_space: Option<u8>,
    pub general_tier_flag: Option<bool>,
    pub general_profile_idc: Option<u8>,
    pub general_profile_compatibility_flags: Option<u32>,
    pub general_constraint_indicator_flag: Option<u64>,
    pub general_level_idc: Option<u8>,
    pub min_spatial_segmentation_idc: Option<u16>,
    pub parallelism_type: Option<u8>,
    pub chroma_format_idc: Option<u8>,
    pub bit_depth_luma_minus8: Option<u8>,
    pub bit_depth_chroma_minus8: Option<u8>,
    pub avg_frame_rate: Option<u16>,
    pub constant_frame_rate: Option<u8>,
    pub num_temporal_layers: Option<u8>,
    pub temporal_id_nested: Option<bool>,
    pub length_size_minus_one: Option<u8>,
    pub arrays: Option<Vec<HvcCArray>>,
    pub use_hvc1: bool,
}

impl Default for HevcConfig {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            configuration_version: None,
            general_profile_space: None,
            general_tier_flag: None,
            general_profile_idc: None,
            general_profile_compatibility_flags: None,
            general_constraint_indicator_flag: None,
            general_level_idc: None,
            min_spatial_segmentation_idc: None,
            parallelism_type: None,
            chroma_format_idc: None,
            bit_depth_luma_minus8: None,
            bit_depth_chroma_minus8: None,
            avg_frame_rate: None,
            constant_frame_rate: None,
            num_temporal_layers: None,
            temporal_id_nested: None,
            length_size_minus_one: None,
            arrays: None,
            use_hvc1: false,
        }
    }
}

impl HevcConfig {
    pub fn new() -> Self {
        Self::default()
    }

    // Builder methods for each field
    pub fn with_width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: u16) -> Self {
        self.height = Some(height);
        self
    }

    pub fn with_configuration_version(mut self, version: u8) -> Self {
        self.configuration_version = Some(version);
        self
    }

    pub fn with_use_hvc1(mut self, use_hvc1: bool) -> Self {
        self.use_hvc1 = use_hvc1;
        self
    }

    // ... add similar methods for all other fields ...
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Vp9Config {
    pub width: u16,
    pub height: u16,
    pub profile: u8,
    pub level: u8,
    pub bit_depth: u8,
    pub chroma_subsampling: u8,
    pub video_full_range_flag: bool,
    pub color_primaries: u8,
    pub transfer_characteristics: u8,
    pub matrix_coefficients: u8,
    pub codec_initialization_data_size: u16,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AacConfig {
    pub bitrate: u32,
    pub profile: AudioObjectType,
    pub freq_index: SampleFreqIndex,
    pub chan_conf: ChannelConfig,

    // New fields from Mp4aBox and child structs
    pub data_reference_index: u16,
    pub sound_version: u16,
    pub samplesize: u16,

    // From EsdsBox
    pub esds_version: Option<u8>,
    pub esds_flags: Option<u32>,

    // From ESDescriptor
    pub es_id: Option<u16>,

    // From DecoderConfigDescriptor
    pub object_type_indication: Option<u8>,
    pub stream_type: Option<u8>,
    pub up_stream: Option<u8>,
    pub buffer_size_db: Option<u32>,
    pub max_bitrate: Option<u32>,
    pub avg_bitrate: Option<u32>,

    pub qt_bytes: Option<Vec<u8>>,
}

impl AacConfig {
    pub fn from_mp4a_box(mp4a: &Mp4aBox) -> Self {
        let mut config = AacConfig {
            // Set your existing required fields with appropriate defaults or mappings
            bitrate: mp4a
                .esds
                .as_ref()
                .map(|esds| esds.es_desc.dec_config.avg_bitrate)
                .unwrap_or(0),

            profile: mp4a
                .esds
                .as_ref()
                .map(|esds| {
                    AudioObjectType::try_from(esds.es_desc.dec_config.dec_specific.profile)
                        .unwrap_or(AudioObjectType::AacLowComplexity)
                })
                .unwrap_or(AudioObjectType::AacLowComplexity),

            freq_index: mp4a
                .esds
                .as_ref()
                .map(|esds| {
                    SampleFreqIndex::try_from(esds.es_desc.dec_config.dec_specific.freq_index)
                        .unwrap_or(SampleFreqIndex::Freq48000)
                })
                .unwrap_or(SampleFreqIndex::Freq48000),

            chan_conf: mp4a
                .esds
                .as_ref()
                .map(|esds| {
                    ChannelConfig::try_from(esds.es_desc.dec_config.dec_specific.chan_conf)
                        .unwrap_or(ChannelConfig::Stereo)
                })
                .unwrap_or(ChannelConfig::Stereo),

            // Set new fields from mp4a
            data_reference_index: mp4a.data_reference_index,
            sound_version: 0,

            samplesize: mp4a.samplesize,
            qt_bytes: mp4a.qt_bytes.clone(),

            // Set remaining fields to None initially
            esds_version: mp4a.esds.as_ref().map(|esds| esds.version),
            esds_flags: mp4a.esds.as_ref().map(|esds| esds.flags),
            es_id: mp4a.esds.as_ref().map(|esds| esds.es_desc.es_id),
            object_type_indication: mp4a
                .esds
                .as_ref()
                .map(|esds| esds.es_desc.dec_config.object_type_indication),
            stream_type: mp4a
                .esds
                .as_ref()
                .map(|esds| esds.es_desc.dec_config.stream_type),
            up_stream: mp4a
                .esds
                .as_ref()
                .map(|esds| esds.es_desc.dec_config.up_stream),
            buffer_size_db: mp4a
                .esds
                .as_ref()
                .map(|esds| esds.es_desc.dec_config.buffer_size_db),
            max_bitrate: mp4a
                .esds
                .as_ref()
                .map(|esds| esds.es_desc.dec_config.max_bitrate),
            avg_bitrate: mp4a
                .esds
                .as_ref()
                .map(|esds| esds.es_desc.dec_config.avg_bitrate),
        };

        // Fill in ESDS-related fields if present
        if let Some(esds) = &mp4a.esds {
            config.esds_version = Some(esds.version);
            config.esds_flags = Some(esds.flags);
            config.es_id = Some(esds.es_desc.es_id);

            let dec_config = &esds.es_desc.dec_config;
            config.object_type_indication = Some(dec_config.object_type_indication);
            config.stream_type = Some(dec_config.stream_type);
            config.up_stream = Some(dec_config.up_stream);
            config.buffer_size_db = Some(dec_config.buffer_size_db);
            config.max_bitrate = Some(dec_config.max_bitrate);
            config.avg_bitrate = Some(dec_config.avg_bitrate);
        }
        config
    }
}

impl Default for AacConfig {
    fn default() -> Self {
        let esds = EsdsBox::default();
        Self {
            bitrate: 0,
            profile: AudioObjectType::AacLowComplexity,
            freq_index: SampleFreqIndex::Freq48000,
            chan_conf: ChannelConfig::Stereo,
            samplesize: 16,
            data_reference_index: 1,
            sound_version: 0,
            esds_version: Some(esds.version),
            esds_flags: Some(esds.flags),
            es_id: Some(esds.es_desc.es_id),
            object_type_indication: Some(esds.es_desc.dec_config.object_type_indication),
            stream_type: Some(esds.es_desc.dec_config.stream_type),
            up_stream: Some(esds.es_desc.dec_config.up_stream),
            buffer_size_db: Some(esds.es_desc.dec_config.buffer_size_db),
            max_bitrate: Some(esds.es_desc.dec_config.max_bitrate),
            avg_bitrate: Some(esds.es_desc.dec_config.avg_bitrate),
            qt_bytes: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TtxtConfig {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OpusConfig {
    pub bitrate: u32,
    pub freq_index: SampleFreqIndex,
    pub chan_conf: ChannelConfig,
    pub pre_skip: u16,
}

impl Default for OpusConfig {
    fn default() -> Self {
        Self {
            bitrate: 0,
            freq_index: SampleFreqIndex::Freq48000,
            chan_conf: ChannelConfig::Stereo,
            pre_skip: 0,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MediaConfig {
    AvcConfig(AvcConfig),
    HevcConfig(HevcConfig),
    Vp9Config(Vp9Config),
    AacConfig(AacConfig),
    TtxtConfig(TtxtConfig),
    OpusConfig(OpusConfig),
}

#[derive(Debug)]
pub struct Mp4Sample {
    pub start_time: u64,
    pub duration: u32,
    pub rendering_offset: i32,
    pub is_sync: bool,
    pub bytes: Bytes,
}

#[derive(Debug)]
pub struct Mp4SampleMetadata {
    pub start_time: u64,
    pub duration: u32,
    pub rendering_offset: i32,
    pub is_sync: bool,
}

impl PartialEq for Mp4Sample {
    fn eq(&self, other: &Self) -> bool {
        self.start_time == other.start_time
            && self.duration == other.duration
            && self.rendering_offset == other.rendering_offset
            && self.is_sync == other.is_sync
            && self.bytes.len() == other.bytes.len() // XXX for easy check
    }
}

impl fmt::Display for Mp4Sample {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "start_time {}, duration {}, rendering_offset {}, is_sync {}, length {}",
            self.start_time,
            self.duration,
            self.rendering_offset,
            self.is_sync,
            self.bytes.len()
        )
    }
}

pub fn creation_time(creation_time: u64) -> u64 {
    // convert from MP4 epoch (1904-01-01) to Unix epoch (1970-01-01)
    if creation_time >= 2082844800 {
        creation_time - 2082844800
    } else {
        creation_time
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum DataType {
    Binary = 0x000000,
    Text = 0x000001,
    Image = 0x00000D,
    TempoCpil = 0x000015,
}

#[allow(clippy::derivable_impls)]
impl std::default::Default for DataType {
    fn default() -> Self {
        DataType::Binary
    }
}

impl TryFrom<u32> for DataType {
    type Error = Error;
    fn try_from(value: u32) -> Result<DataType> {
        match value {
            0x000000 => Ok(DataType::Binary),
            0x000001 => Ok(DataType::Text),
            0x00000D => Ok(DataType::Image),
            0x000015 => Ok(DataType::TempoCpil),
            _ => Err(Error::InvalidData("invalid data type")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum MetadataKey {
    Title,
    Year,
    Poster,
    Summary,
}

pub trait Metadata<'a> {
    /// The video's title
    fn title(&self) -> Option<Cow<str>>;
    /// The video's release year
    fn year(&self) -> Option<u32>;
    /// The video's poster (cover art)
    fn poster(&self) -> Option<&[u8]>;
    /// The video's summary
    fn summary(&self) -> Option<Cow<str>>;
}

impl<'a, T: Metadata<'a>> Metadata<'a> for &'a T {
    fn title(&self) -> Option<Cow<str>> {
        (**self).title()
    }

    fn year(&self) -> Option<u32> {
        (**self).year()
    }

    fn poster(&self) -> Option<&[u8]> {
        (**self).poster()
    }

    fn summary(&self) -> Option<Cow<str>> {
        (**self).summary()
    }
}

impl<'a, T: Metadata<'a>> Metadata<'a> for Option<T> {
    fn title(&self) -> Option<Cow<str>> {
        self.as_ref().and_then(|t| t.title())
    }

    fn year(&self) -> Option<u32> {
        self.as_ref().and_then(|t| t.year())
    }

    fn poster(&self) -> Option<&[u8]> {
        self.as_ref().and_then(|t| t.poster())
    }

    fn summary(&self) -> Option<Cow<str>> {
        self.as_ref().and_then(|t| t.summary())
    }
}
